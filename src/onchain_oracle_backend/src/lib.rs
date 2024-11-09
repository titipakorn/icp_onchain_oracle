//1. IMPORT IC MANAGEMENT CANISTER
//This includes all methods and types needed
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk::query;
use std::cell::RefCell;
// use ic_cdk_macros::{self, query, update};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use ic_cdk::api::time;

#[derive(Serialize, Deserialize, Clone, Debug, candid::CandidType)]
struct DataPoint {
    timestamp: u64,
    low: f64,
    high: f64,
    open: f64,
    close: f64,
    volume: f64,
}

thread_local! {
static TOKEN_STORE: RefCell<Vec<DataPoint>> = const { RefCell::new(Vec::new()) };
}


// This struct is legacy code and is not really used in the code.
#[derive(Serialize, Deserialize)]
struct Context {
    bucket_start_time_index: usize,
    closing_price_index: usize,
}

#[query(name = "get_price_list")]
fn get_price_list() -> Vec<DataPoint> {
    let submitted_name = TOKEN_STORE.with(|token_store| {
        token_store
            .borrow()
            .to_vec()
    });
    submitted_name
}


fn get_current_unix_time() -> u64 {
    time() / 1_000_000_000
}

#[ic_cdk::init]
fn init(timer_interval_secs: u64) {
    let interval = std::time::Duration::from_secs(timer_interval_secs);
    ic_cdk::println!("Starting a periodic task with interval {interval:?}");
    ic_cdk_timers::set_timer_interval(interval, move || {
        let timer_interval_secs_clone = timer_interval_secs;
        let start_time = get_current_unix_time();
        let end_time = start_time - timer_interval_secs_clone;
        ic_cdk::spawn(async move {
            get_icp_usd_exchange( end_time,start_time).await;
        });
    });
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade(timer_interval_secs: u64) {
    let interval = std::time::Duration::from_secs(timer_interval_secs);
    ic_cdk::println!("Starting a periodic task with interval {interval:?}");
    ic_cdk_timers::set_timer_interval(interval, move || {
        let timer_interval_secs_clone = timer_interval_secs;
        let start_time = get_current_unix_time();
        let end_time = start_time - timer_interval_secs_clone;
        ic_cdk::spawn(async move {
            get_icp_usd_exchange( end_time,start_time).await;
        });
    });
}

//Update method using the HTTPS outcalls feature
#[ic_cdk::update]
async fn get_icp_usd_exchange(start_timestamp: u64, end_timestamp: u64) {
    //2. SETUP ARGUMENTS FOR HTTP GET request
    // 2.1 Setup the URL and its query parameters
    // type Timestamp = u64;
    // let start_timestamp: Timestamp = 1682978460; //May 1, 2023 22:01:00 GMT
    let seconds_of_time: u64 = 60; //we start with 60 seconds
    let host = "api.exchange.coinbase.com";
    let url = format!(
        "https://{}/products/ICP-USD/candles?start={}&end={}&granularity={}",
        host,
        start_timestamp,
        end_timestamp,
        seconds_of_time
    );

    // 2.2 prepare headers for the system http_request call
    //Note that `HttpHeader` is declared in line 4
    let request_headers = vec![
        HttpHeader {
            name: "Host".to_string(),
            value: format!("{host}:443"),
        },
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "exchange_rate_canister".to_string(),
        },
    ];


    // This struct is legacy code and is not really used in the code. Need to be removed in the future
    // The "TransformContext" function does need a CONTEXT parameter, but this implementation is not necessary
    // the TransformContext(transform, context) below accepts this "context", but it does nothing with it in this implementation.
    // bucket_start_time_index and closing_price_index are meaninglesss
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };

    //note "CanisterHttpRequestArgument" and "HttpMethod" are declared in line 4
    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,               //optional for request
        max_response_bytes: None, //optional for request
        // transform: None,          //optional for request
        transform: Some(TransformContext::new(transform, serde_json::to_vec(&context).unwrap())),
        headers: request_headers,
    };

    //3. MAKE HTTPS REQUEST AND WAIT FOR RESPONSE

    //Note: in Rust, `http_request()` already sends the cycles needed
    //so no need for explicit Cycles.add() as in Motoko
    match http_request(request).await {
        //4. DECODE AND RETURN THE RESPONSE

        //See:https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/struct.HttpResponse.html
        Ok((response,)) => {

            // ic_cdk::api::print(format!("Body Response = {:?}", &response.body));
            // let str_body = String::from_utf8(response.body)
            //     .expect("Transformed response is not UTF-8 encoded.");
            let parsed: Value = serde_json::from_slice(&response.body)
            .expect("Failed to parse JSON response");
        
            // ic_cdk::api::print(format!("Body Json = {:?}", &parsed));
            // println!("Response body: {:?}", str_body);
            //Return the body as a string and end the method
            // let parsed: Vec<Vec<f64>> = serde_json::from_str(&str_body).unwrap();

            // Loop through the parsed data and create DataPoint instances
            // Assuming the JSON structure is an array of arrays of floats
        if let Value::Array(entries) = parsed {
            for entry in entries {
                if let Value::Array(values) = entry {
                    if values.len() == 6 {
                        let data_point = DataPoint {
                            timestamp: values[0].as_u64().expect("Expected u64"),
                            low: values[1].as_f64().expect("Expected f64"),
                            high: values[2].as_f64().expect("Expected f64"),
                            open: values[3].as_f64().expect("Expected f64"),
                            close: values[4].as_f64().expect("Expected f64"),
                            volume: values[5].as_f64().expect("Expected f64"),
                        };
                        TOKEN_STORE.with(|token_store| {
                            let mut token_store = token_store.borrow_mut();
                            token_store.push(data_point);
                        });
                    }
                }
            }
        }

        }
        Err((r, m)) => {
            let _message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", _message));
        }
    }
}


// Strips all data that is not needed from the original response.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {

    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];
    

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
    };

    if res.status == 200 {

        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from coinbase: err = {:?}", raw));
    }
    res
}
import React, { useState, useEffect } from "react";
import { onchain_oracle_backend } from "declarations/onchain_oracle_backend";
const styles = `
  .data-table-container {
    width: 100%;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    font-family: system-ui, -apple-system, sans-serif;
  }

  .table-header {
    padding: 16px;
    background-color: #f8fafc;
    border-bottom: 1px solid #e2e8f0;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .title {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0;
  }

  .update-info {
    font-size: 0.875rem;
    color: #64748b;
  }

  .updating {
    margin-left: 8px;
    color: #3b82f6;
    animation: pulse 2s infinite;
  }

  .table-wrapper {
    overflow-x: auto;
    padding: 16px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    background-color: #f8fafc;
    padding: 12px;
    text-align: left;
    font-weight: 600;
    border-bottom: 1px solid #e2e8f0;
  }

  th:not(:first-child) {
    text-align: right;
  }

  td {
    padding: 12px;
    border-bottom: 1px solid #e2e8f0;
    font-family: 'Courier New', monospace;
  }

  td:not(:first-child) {
    text-align: right;
  }

  tr:hover {
    background-color: #f8fafc;
    transition: background-color 0.15s;
  }

  .increase {
    color: #22c55e;
  }

  .decrease {
    color: #ef4444;
  }

  .neutral {
    color: #64748b;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .pulse {
    animation: pulse 2s infinite;
  }
`;

const convertDataPoints = (dataPoints) => {
  return dataPoints.map((dataPoint) => [
    Number(dataPoint.timestamp),
    dataPoint.low,
    dataPoint.high,
    dataPoint.open,
    dataPoint.close,
    dataPoint.volume,
  ]);
};

const DataTable = () => {
  const [data, setData] = useState([]);
  const [lastUpdate, setLastUpdate] = useState(new Date());
  const [isUpdating, setIsUpdating] = useState(false);

  useEffect(() => {
    const updateData = () => {
      setIsUpdating(true);
      onchain_oracle_backend.get_price_list().then((result) => {
        let new_result = convertDataPoints(result);
        setData(new_result);
        setLastUpdate(new Date());
        setTimeout(() => setIsUpdating(false), 60000);
      });
    };
    onchain_oracle_backend.get_price_list().then((result) => {
      let new_result = convertDataPoints(result);
      setData(new_result);
    });
    const interval = setInterval(updateData, 300000); // 5 minutes
    return () => clearInterval(interval);
  }, [data]);

  const formatTimestamp = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString();
  };

  const formatNumber = (num) => num.toFixed(4);

  const getPriceChangeClass = (current, previous) => {
    if (current > previous) return "increase";
    if (current < previous) return "decrease";
    return "neutral";
  };

  return (
    <>
      <style>{styles}</style>
      <div className="data-table-container">
        <div className="table-header">
          <div className="header-content">
            <h2 className="title">Price of ICP/USD</h2>
            <div className="update-info">
              Last update: {lastUpdate.toLocaleTimeString()}
              {isUpdating && <span className="updating">Updating...</span>}
            </div>
          </div>
        </div>
        <div className="table-wrapper">
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>Open</th>
                <th>High</th>
                <th>Low</th>
                <th>Close</th>
                <th>Volume</th>
              </tr>
            </thead>
            <tbody>
              {data.map((row, index) => {
                const prevRow = data[index + 1];
                return (
                  <tr key={row[0]} className={isUpdating ? "pulse" : ""}>
                    <td>{formatTimestamp(row[0])}</td>
                    <td className={getPriceChangeClass(row[1], prevRow?.[1])}>
                      {formatNumber(row[1])}
                    </td>
                    <td className={getPriceChangeClass(row[2], prevRow?.[2])}>
                      {formatNumber(row[2])}
                    </td>
                    <td className={getPriceChangeClass(row[3], prevRow?.[3])}>
                      {formatNumber(row[3])}
                    </td>
                    <td className={getPriceChangeClass(row[4], prevRow?.[4])}>
                      {formatNumber(row[4])}
                    </td>
                    <td>{formatNumber(row[5])}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </>
  );
};

export default DataTable;

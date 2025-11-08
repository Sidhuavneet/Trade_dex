// Transaction logger for API requests and responses

interface TransactionLog {
  timestamp: string;
  type: 'request' | 'response' | 'error';
  endpoint: string;
  method: string;
  data?: any;
  error?: string;
}

class TransactionLogger {
  private logs: TransactionLog[] = [];
  private maxLogs = 100;

  log(type: 'request' | 'response' | 'error', endpoint: string, method: string, data?: any, error?: string) {
    const log: TransactionLog = {
      timestamp: new Date().toISOString(),
      type,
      endpoint,
      method,
      data,
      error,
    };

    this.logs.push(log);

    // Keep only last N logs
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    // Console output for debugging
    const emoji = type === 'request' ? '📤' : type === 'response' ? '📥' : '❌';
    console.log(`${emoji} [${type.toUpperCase()}] ${method} ${endpoint}`, data || error || '');

    // Also log to console table for better readability
    if (type === 'response' && data) {
      console.table({
        endpoint,
        method,
        timestamp: log.timestamp,
        data: JSON.stringify(data, null, 2),
      });
    }
  }

  getLogs(): TransactionLog[] {
    return [...this.logs];
  }

  clearLogs() {
    this.logs = [];
  }

  getLogsByEndpoint(endpoint: string): TransactionLog[] {
    return this.logs.filter(log => log.endpoint === endpoint);
  }
}

export const transactionLogger = new TransactionLogger();


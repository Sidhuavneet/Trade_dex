// API service layer for backend communication

import { transactionLogger } from './transactionLogger';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

export interface Trade {
  id: string;
  timestamp: string;
  base_symbol: string;
  quote_symbol: string;
  base_mint: string;
  quote_mint: string;
  price: number;
  amount: number;
  side: 'buy' | 'sell' | 'price';
  total_value: number;
  dex_program: string;
  slot: number;
}

export interface OHLCVData {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface AuthNonceResponse {
  nonce: string;
}

export interface AuthVerifyRequest {
  publicKey: string;
  signature: string;
  nonce: string;
}

export interface AuthVerifyResponse {
  token: string;
  expiresAt: string;
}

// Auth endpoints
export const authApi = {
  async getNonce(): Promise<AuthNonceResponse> {
    const endpoint = '/auth/nonce';
    const method = 'GET';
    
    try {
      // Log request
      transactionLogger.log('request', endpoint, method);
      const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        method,
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        transactionLogger.log('error', endpoint, method, undefined, `HTTP ${response.status}: ${errorText}`);
        throw new Error(`Failed to fetch nonce: ${response.status} ${errorText}`);
      }
      
      const data = await response.json();
      
      // Log response
      transactionLogger.log('response', endpoint, method, data);
      
      return data;
    } catch (error) {
      transactionLogger.log('error', endpoint, method, undefined, error instanceof Error ? error.message : String(error));
      throw error;
    }
  },

  async verifySignature(data: AuthVerifyRequest): Promise<AuthVerifyResponse> {
    const endpoint = '/auth/verify';
    const method = 'POST';
    
    try {
      // Log request with masked signature for security
      const logData = { 
        ...data, 
        signature: data.signature.length > 20 
          ? data.signature.substring(0, 20) + '...' 
          : '***' 
      };
      transactionLogger.log('request', endpoint, method, logData);
      
      const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        method,
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        transactionLogger.log('error', endpoint, method, undefined, `HTTP ${response.status}: ${errorText}`);
        throw new Error(`Failed to verify signature: ${response.status} ${errorText}`);
      }
      
      const responseData = await response.json();
      
      // Log response (mask token for security)
      const maskedResponse = { 
        ...responseData, 
        token: responseData.token 
          ? (responseData.token.length > 20 
              ? responseData.token.substring(0, 20) + '...' 
              : '***')
          : undefined
      };
      transactionLogger.log('response', endpoint, method, maskedResponse);
      
      return responseData;
    } catch (error) {
      transactionLogger.log('error', endpoint, method, undefined, error instanceof Error ? error.message : String(error));
      throw error;
    }
  },
};

// Trade endpoints
export const tradeApi = {
  async getTrades(pair: string, limit: number = 100): Promise<Trade[]> {
    const response = await fetch(
      `${API_BASE_URL}/api/trades?pair=${encodeURIComponent(pair)}&limit=${limit}`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      }
    );
    
    const data = await response.json();
    
    if (!response.ok) {
      const errorMessage = data?.message || data?.error || 'Failed to fetch trades';
      throw new Error(errorMessage);
    }
    
    // Check if response is an error object
    if (data && typeof data === 'object' && 'error' in data) {
      const errorMessage = data.message || data.error || 'Failed to fetch trades';
      throw new Error(errorMessage);
    }
    
    // Ensure it's an array
    if (!Array.isArray(data)) {
      throw new Error('Invalid response format: expected an array');
    }
    
    return data;
  },

  async getOHLCV(pair: string, interval: string = '1m'): Promise<OHLCVData[]> {
    const response = await fetch(
      `${API_BASE_URL}/api/ohlcv?pair=${encodeURIComponent(pair)}&interval=${interval}`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      }
    );
    
    if (!response.ok) {
      throw new Error('Failed to fetch OHLCV data');
    }
    
    return response.json();
  },
};

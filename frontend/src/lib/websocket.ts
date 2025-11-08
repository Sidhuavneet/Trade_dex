// WebSocket client for live trade updates

import { Trade } from './api';

const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:3000';

export type TradeCallback = (trade: Trade) => void;
export type ConnectionCallback = (connected: boolean) => void;

export class TradeWebSocket {
  private ws: WebSocket | null = null;
  private callbacks: TradeCallback[] = [];
  private connectionCallbacks: ConnectionCallback[] = [];
  private reconnectTimeout: number | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 3000;
  private shouldReconnect = true;
  private pendingPairSelection: string | null = null; // Queue pair selection if WebSocket not ready

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.log('WebSocket already connected');
      return;
    }

    if (this.ws?.readyState === WebSocket.CONNECTING) {
      console.log('WebSocket connection already in progress');
      return;
    }

    try {
      this.shouldReconnect = true;
      this.ws = new WebSocket(`${WS_BASE_URL}/ws/trades`);

      this.ws.onopen = () => {
        console.log('✅ WebSocket connected');
        this.reconnectAttempts = 0;
        this.notifyConnection(true);
        // Send pending pair selection if any
        if (this.pendingPairSelection) {
          const pair = this.pendingPairSelection;
          this.pendingPairSelection = null;
          setTimeout(() => {
            this.sendPairSelection(pair);
            console.log('📤 [WebSocket] Sent queued pair selection:', pair);
          }, 100);
        }
      };

      this.ws.onmessage = (event) => {
        try {
          const trade: Trade = JSON.parse(event.data);
          
          // Log received trades and price updates
          if (trade.side === 'price') {
            console.log('💰 [RECEIVE] Price update received:', {
              pair: `${trade.base_symbol}/${trade.quote_symbol}`,
              price: trade.price,
              timestamp: trade.timestamp,
            });
          } else if (trade.amount > 0) {
            console.log('📥 [RECEIVE] Trade received:', {
              trade,
            });
          }
          
          this.callbacks.forEach((callback) => callback(trade));
        } catch (error) {
          console.error('❌ [RECEIVE] Failed to parse trade message:', error, event.data);
        }
      };

      this.ws.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
        this.notifyConnection(false);
      };

      this.ws.onclose = () => {
        console.log('🔌 WebSocket disconnected');
        this.notifyConnection(false);
        
        if (this.shouldReconnect) {
          this.attemptReconnect();
        }
      };
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
      this.notifyConnection(false);
      
      if (this.shouldReconnect) {
        this.attemptReconnect();
      }
    }
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
    }

    this.reconnectAttempts++;
    console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);

    this.reconnectTimeout = window.setTimeout(() => {
      this.connect();
    }, this.reconnectDelay);
  }

  subscribe(callback: TradeCallback): () => void {
    this.callbacks.push(callback);
    
    // Return unsubscribe function
    return () => {
      this.callbacks = this.callbacks.filter((cb) => cb !== callback);
    };
  }

  onConnection(callback: ConnectionCallback): () => void {
    this.connectionCallbacks.push(callback);
    
    // Return unsubscribe function
    return () => {
      this.connectionCallbacks = this.connectionCallbacks.filter((cb) => cb !== callback);
    };
  }

  private notifyConnection(connected: boolean): void {
    this.connectionCallbacks.forEach((callback) => callback(connected));
  }

  disconnect(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.callbacks = [];
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  sendPairSelection(pair: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      const message = JSON.stringify({
        type: 'select_pair',
        pair: pair,
      });
      this.ws.send(message);
      console.log('📤 [WebSocket] Sent pair selection:', pair);
      this.pendingPairSelection = null; // Clear pending if sent successfully
    } else {
      // Queue the pair selection to send when WebSocket connects
      this.pendingPairSelection = pair;
      console.warn('⚠️ [WebSocket] Not connected, queued pair selection:', pair);
      
      // If WebSocket is connecting, wait a bit and try again
      if (this.ws?.readyState === WebSocket.CONNECTING) {
        setTimeout(() => {
          if (this.ws?.readyState === WebSocket.OPEN && this.pendingPairSelection === pair) {
            this.sendPairSelection(pair);
          }
        }, 500);
      }
    }
  }
}

export const tradeWebSocket = new TradeWebSocket();

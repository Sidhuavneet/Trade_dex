// OHLC aggregation utility for converting trades into candlesticks

import { Trade } from './api';
import { Time } from 'lightweight-charts';

export interface OHLC {
  time: Time;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  tradeCount: number;
}

/**
 * Convert interval string to milliseconds
 * @param interval - Interval string (e.g., '1m', '5m', '15m', '1h')
 * @returns Interval in milliseconds
 */
export function intervalToMs(interval: string): number {
  const match = interval.match(/^(\d+)([mhd])$/);
  if (!match) {
    throw new Error(`Invalid interval format: ${interval}`);
  }

  const value = parseInt(match[1], 10);
  const unit = match[2];

  switch (unit) {
    case 'm':
      return value * 60 * 1000; // minutes to milliseconds
    case 'h':
      return value * 60 * 60 * 1000; // hours to milliseconds
    case 'd':
      return value * 24 * 60 * 60 * 1000; // days to milliseconds
    default:
      throw new Error(`Unknown interval unit: ${unit}`);
  }
}

/**
 * Get candle start time for a given timestamp and interval
 * @param timestamp - Trade timestamp (milliseconds)
 * @param intervalMs - Interval in milliseconds
 * @returns Candle start time (Unix timestamp in seconds)
 */
export function getCandleStartTime(timestamp: number, intervalMs: number): number {
  return Math.floor(timestamp / intervalMs) * intervalMs;
}

/**
 * Convert Unix timestamp (milliseconds) to TradingView time format
 * @param timestamp - Unix timestamp in milliseconds
 * @returns TradingView time (Unix timestamp in seconds)
 */
export function toTradingViewTime(timestamp: number): Time {
  return Math.floor(timestamp / 1000) as Time;
}

/**
 * Aggregate trades into OHLC candles
 * @param trades - Array of trades
 * @param interval - Interval string (e.g., '1m', '5m', '15m', '1h')
 * @returns Array of OHLC candles
 */
export function aggregateTradesToOHLC(trades: Trade[], interval: string): OHLC[] {
  if (trades.length === 0) {
    return [];
  }

  const intervalMs = intervalToMs(interval);
  const candlesMap = new Map<number, OHLC>();

  // Sort trades by timestamp
  const sortedTrades = [...trades].sort(
    (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  );

  for (const trade of sortedTrades) {
    const tradeTime = new Date(trade.timestamp).getTime();
    const candleStartTime = getCandleStartTime(tradeTime, intervalMs);
    const candleTime = toTradingViewTime(candleStartTime);

    // Get or create candle
    let candle = candlesMap.get(candleStartTime);
    if (!candle) {
      candle = {
        time: candleTime,
        open: trade.price,
        high: trade.price,
        low: trade.price,
        close: trade.price,
        volume: 0,
        tradeCount: 0,
      };
      candlesMap.set(candleStartTime, candle);
    }

    // Update candle
    candle.high = Math.max(candle.high, trade.price);
    candle.low = Math.min(candle.low, trade.price);
    candle.close = trade.price; // Close is the last price in the candle
    candle.volume += trade.amount * trade.price; // Volume = amount * price
    candle.tradeCount += 1;
  }

  // Convert map to sorted array
  return Array.from(candlesMap.values()).sort((a, b) => a.time - b.time);
}

/**
 * Update or create a candle with a new trade
 * @param candles - Map of candles keyed by start time
 * @param trade - New trade
 * @param intervalMs - Interval in milliseconds
 * @returns Updated candle or null if trade doesn't belong to current candle set
 */
export function updateCandleWithTrade(
  candles: Map<number, OHLC>,
  trade: Trade,
  intervalMs: number
): OHLC | null {
  const tradeTime = new Date(trade.timestamp).getTime();
  const candleStartTime = getCandleStartTime(tradeTime, intervalMs);
  const candleTime = toTradingViewTime(candleStartTime);

  // Get or create candle
  let candle = candles.get(candleStartTime);
  if (!candle) {
    // Create new candle - open is the first trade's price
    candle = {
      time: candleTime,
      open: trade.price, // First trade's price is the open
      high: trade.price,
      low: trade.price,
      close: trade.price,
      volume: 0,
      tradeCount: 0,
    };
    candles.set(candleStartTime, candle);
  }

  // Update candle (open stays the same - it's the first trade's price)
  candle.high = Math.max(candle.high, trade.price);
  candle.low = Math.min(candle.low, trade.price);
  candle.close = trade.price; // Close is always the last trade's price
  candle.volume += trade.amount * trade.price; // Volume = amount * price
  candle.tradeCount += 1;

  return candle;
}


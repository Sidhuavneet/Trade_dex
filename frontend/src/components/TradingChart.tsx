import { useEffect, useRef, useState } from 'react';
import { createChart, IChartApi, ISeriesApi, CandlestickData, Time, CandlestickSeries } from 'lightweight-charts';
import { Trade, tradeApi } from '@/lib/api';
import { tradeWebSocket } from '@/lib/websocket';
import { intervalToMs, updateCandleWithTrade, OHLC, toTradingViewTime } from '@/lib/ohlc';

interface TradingChartProps {
  pair: string;
  interval?: string;
}

export const TradingChart = ({ pair, interval = '1m' }: TradingChartProps) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const candlesMapRef = useRef<Map<number, OHLC>>(new Map());
  const tradesBufferRef = useRef<Trade[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!chartContainerRef.current) return;

    // Create chart
    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { color: '#0a0e13' },
        textColor: '#d1d5db',
      },
      grid: {
        vertLines: { color: '#1a1e25' },
        horzLines: { color: '#1a1e25' },
      },
      width: chartContainerRef.current.clientWidth,
      height: chartContainerRef.current.clientHeight || 500,
      autoSize: false, // Disable autoSize to respect fixed parent height
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        borderColor: '#1a1e25',
      },
      rightPriceScale: {
        borderColor: '#1a1e25',
        // Automatically adjust scale to fit price range
        autoScale: true,
        // Scale margins for better visibility
        scaleMargins: {
          top: 0.1,
          bottom: 0.1,
        },
        // Enable precision for very small prices
        entireTextOnly: false,
      },
    });

    const candlestickSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderUpColor: '#22c55e',
      borderDownColor: '#ef4444',
      wickUpColor: '#22c55e',
      wickDownColor: '#ef4444',
      priceFormat: {
        type: 'price',
        precision: 8, // Allow up to 8 decimal places for very small prices
        minMove: 0.00000001, // Minimum price movement for very small prices
      },
      // Enable price line visibility for small values
      priceLineVisible: true,
      lastValueVisible: true,
    });

    chartRef.current = chart;
    candlestickSeriesRef.current = candlestickSeries;

    // Handle resize
    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
          height: chartContainerRef.current.clientHeight,
        });
      }
    };

    window.addEventListener('resize', handleResize);

    // Fetch initial data (optional - can start empty if backend doesn't have historical data)
    const fetchChartData = async () => {
      try {
        setIsLoading(true);
        const data = await tradeApi.getOHLCV(pair, interval);
        
        if (data && data.length > 0) {
          const chartData: CandlestickData[] = data.map((d: any) => ({
            time: d.time as Time,
            open: d.open,
            high: d.high,
            low: d.low,
            close: d.close,
          }));

          candlestickSeries.setData(chartData);
          
          // Populate candles map from initial data
          const intervalMs = intervalToMs(interval);
          candlesMapRef.current.clear();
          for (const candle of chartData) {
            const timeMs = (candle.time as number) * 1000; // Convert to milliseconds
            const candleStartTime = Math.floor(timeMs / intervalMs) * intervalMs;
            candlesMapRef.current.set(candleStartTime, {
              time: candle.time,
              open: candle.open,
              high: candle.high,
              low: candle.low,
              close: candle.close,
              volume: 0,
              tradeCount: 0,
            });
          }
          
          // Fit content to ensure proper scaling for both time and price
          chart.timeScale().fitContent();
          
          // Force price scale to auto-scale based on data
          // This ensures small values (like 0.0002) are properly visible
          if (chartData.length > 0) {
            const prices = chartData.flatMap(c => [c.high, c.low, c.open, c.close]);
            const minPrice = Math.min(...prices);
            const maxPrice = Math.max(...prices);
            const priceRange = maxPrice - minPrice;
            
            // If price range is very small, ensure the scale adjusts properly
            if (priceRange > 0 && maxPrice > 0) {
              // The autoScale: true should handle this, but we can ensure it by resetting
              chart.priceScale('right').applyOptions({
                autoScale: true,
              });
            }
          }
        }
      } catch (error) {
        console.error('Failed to fetch initial chart data:', error);
        // Start with empty chart if backend doesn't have historical data yet
        // WebSocket will populate it in real-time
      } finally {
        setIsLoading(false);
      }
    };

    fetchChartData();

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.remove();
    };
  }, [pair, interval]);

  // Subscribe to WebSocket trades and update chart in real-time
  useEffect(() => {
    if (!candlestickSeriesRef.current) return;

    const candlestickSeries = candlestickSeriesRef.current;
    const intervalMs = intervalToMs(interval);
    const [baseSymbol, quoteSymbol] = pair.split('/');

    // Subscribe to WebSocket trades
    const unsubscribe = tradeWebSocket.subscribe((trade: Trade) => {
      // Filter trades by pair
      if (trade.base_symbol === baseSymbol && trade.quote_symbol === quoteSymbol) {
        // Add to buffer for potential bulk processing
        tradesBufferRef.current.push(trade);

        // Update candle with new trade
        const updatedCandle = updateCandleWithTrade(
          candlesMapRef.current,
          trade,
          intervalMs
        );

        if (updatedCandle) {
          // Convert OHLC to CandlestickData
          const candleData: CandlestickData = {
            time: updatedCandle.time,
            open: updatedCandle.open,
            high: updatedCandle.high,
            low: updatedCandle.low,
            close: updatedCandle.close,
          };

          // Update chart with new/updated candle
          // TradingView will automatically update if the candle exists, or create a new one
          candlestickSeries.update(candleData);
          
          // Ensure price scale auto-adjusts for small values
          // This is especially important when switching from high-value pairs (SOL/USDC ~160)
          // to low-value pairs (BONK/SOL ~0.0002)
          if (chartRef.current) {
            chartRef.current.priceScale('right').applyOptions({
              autoScale: true,
            });
          }
        }
      }
    });

    return () => {
      unsubscribe();
    };
  }, [pair, interval]);

  return (
    <div className="relative w-full h-full flex flex-col overflow-hidden">
      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center bg-card/50 z-10">
          <div className="text-muted-foreground">Loading chart data...</div>
        </div>
      )}
      <div ref={chartContainerRef} className="w-full h-full" />
    </div>
  );
};

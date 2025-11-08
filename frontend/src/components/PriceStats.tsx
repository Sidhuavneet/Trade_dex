import { useEffect, useState, useRef } from 'react';
import { Trade } from '@/lib/api';
import { tradeWebSocket } from '@/lib/websocket';
import { cn } from '@/lib/utils';
import { TrendingUp, TrendingDown } from 'lucide-react';

interface PriceStatsProps {
  pair: string;
}

interface Stats {
  currentPrice: number;
  change24h: number;
  changePercent24h: number;
  volume24h: number;
  high24h: number;
  low24h: number;
}

export const PriceStats = ({ pair }: PriceStatsProps) => {
  const [stats, setStats] = useState<Stats>({
    currentPrice: 0,
    change24h: 0,
    changePercent24h: 0,
    volume24h: 0,
    high24h: 0,
    low24h: 0,
  });

  const tradesRef = useRef<Trade[]>([]);

  useEffect(() => {
    // Reset trades and stats when pair changes
    tradesRef.current = [];
    setStats({
      currentPrice: 0,
      change24h: 0,
      changePercent24h: 0,
      volume24h: 0,
      high24h: 0,
      low24h: 0,
    });

    const unsubscribe = tradeWebSocket.subscribe((trade: Trade) => {
      const [base, quote] = pair.split('/');
      if (trade.base_symbol === base && trade.quote_symbol === quote) {
        // Handle price-only updates (from Jupiter) - update current price only
        if (trade.side === 'price' || trade.amount === 0) {
          console.log('💰 [PriceStats] Updating price:', {
            pair: `${trade.base_symbol}/${trade.quote_symbol}`,
            price: trade.price,
          });
          setStats((prev) => ({
            ...prev,
            currentPrice: trade.price,
            // Keep other stats unchanged
            change24h: prev.change24h,
            changePercent24h: prev.changePercent24h,
            volume24h: prev.volume24h,
            high24h: prev.high24h > 0 ? Math.max(prev.high24h, trade.price) : trade.price,
            low24h: prev.low24h > 0 ? Math.min(prev.low24h, trade.price) : trade.price,
          }));
          return;
        }

        // Handle actual trades - exclude price-only updates
        if ((trade.side === 'buy' || trade.side === 'sell') && trade.amount > 0) {
          tradesRef.current.push(trade);

          // Keep only last 24h trades
          const now = Date.now();
          const recentTrades = tradesRef.current.filter(
            (t) => t.side !== 'price' && t.amount > 0 && new Date(t.timestamp).getTime() > (now - 24 * 60 * 60 * 1000)
          );

          // Update trades ref to only keep recent trades
          tradesRef.current = recentTrades;

          if (recentTrades.length > 0) {
            const prices = recentTrades.map((t) => t.price);
            const currentPrice = prices[prices.length - 1];
            const firstPrice = prices[0];
            const high24h = Math.max(...prices);
            const low24h = Math.min(...prices);
            const volume24h = recentTrades.reduce((sum, t) => sum + t.amount * t.price, 0); // Volume = amount * price
            const change24h = currentPrice - firstPrice;
            const changePercent24h = firstPrice > 0 ? (change24h / firstPrice) * 100 : 0;

            setStats({
              currentPrice,
              change24h,
              changePercent24h,
              volume24h,
              high24h,
              low24h,
            });
          }
        }
      }
    });

    return () => {
      unsubscribe();
    };
  }, [pair]);

  const formatPrice = (price: number) => {
    if (price === 0 || !price || isNaN(price)) return '$0.000000';
    // Use scientific notation for very small numbers
    if (Math.abs(price) < 0.0001 && price !== 0) {
      return `$${price.toExponential(2)}`;
    }
    // For larger numbers, show up to 6 decimal places, removing trailing zeros
    const formatted = price.toFixed(6).replace(/\.?0+$/, '');
    return `$${formatted}`;
  };

  const formatVolume = (volume: number) => {
    if (volume > 1000000) return `$${(volume / 1000000).toFixed(2)}M`;
    if (volume > 1000) return `$${(volume / 1000).toFixed(2)}K`;
    return `$${volume.toFixed(2)}`;
  };

  const isPositive = stats.change24h >= 0;

  return (
    <div className="grid grid-cols-2 md:grid-cols-6 gap-4 p-4 border-b border-border bg-card">
      <div className="md:col-span-2">
        <div className="text-xs text-muted-foreground mb-1">Price</div>
        <div className="text-2xl font-bold font-mono">
          {formatPrice(stats.currentPrice)}
        </div>
      </div>

      <div>
        <div className="text-xs text-muted-foreground mb-1">24h Change</div>
        <div
          className={cn(
            'text-lg font-bold flex items-center gap-1',
            isPositive ? 'text-success' : 'text-destructive'
          )}
        >
          {isPositive ? (
            <TrendingUp className="w-4 h-4" />
          ) : (
            <TrendingDown className="w-4 h-4" />
          )}
          {isPositive ? '+' : ''}
          {stats.changePercent24h.toFixed(2)}%
        </div>
      </div>

      <div>
        <div className="text-xs text-muted-foreground mb-1">24h High</div>
        <div className="text-lg font-bold font-mono">
          {formatPrice(stats.high24h)}
        </div>
      </div>

      <div>
        <div className="text-xs text-muted-foreground mb-1">24h Low</div>
        <div className="text-lg font-bold font-mono">
          {formatPrice(stats.low24h)}
        </div>
      </div>

      <div>
        <div className="text-xs text-muted-foreground mb-1">24h Volume</div>
        <div className="text-lg font-bold">
          {formatVolume(stats.volume24h)}
        </div>
      </div>

      <div>
        <div className="text-xs text-muted-foreground mb-1">Pair</div>
        <div className="text-lg font-bold text-primary">
          {pair}
        </div>
      </div>
    </div>
  );
};

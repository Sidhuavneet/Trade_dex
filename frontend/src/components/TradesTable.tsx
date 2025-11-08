import { useEffect, useState, useRef, useCallback } from 'react';
import { Trade, tradeApi } from '@/lib/api';
import { tradeWebSocket } from '@/lib/websocket';
import { cn } from '@/lib/utils';

interface TradesTableProps {
  pair: string;
}

export const TradesTable = ({ pair }: TradesTableProps) => {
  const [trades, setTrades] = useState<Trade[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Parse pair for filtering
  const [baseSymbol, quoteSymbol] = pair.split('/');

  // Helper function to check if a trade matches the selected pair
  const matchesPair = useCallback((trade: Trade): boolean => {
    // Check if trade matches the selected pair (both directions)
    const matchesForward = trade.base_symbol === baseSymbol && trade.quote_symbol === quoteSymbol;
    const matchesReverse = trade.base_symbol === quoteSymbol && trade.quote_symbol === baseSymbol;
    return matchesForward || matchesReverse;
  }, [baseSymbol, quoteSymbol]);

  // Don't load initial trades - only show real-time WebSocket trades

  useEffect(() => {
    // Reset trades when pair changes
    setTrades([]);
    
    // Subscribe to WebSocket trade updates
    const unsubscribe = tradeWebSocket.subscribe((trade: Trade) => {
      // Filter out price updates and only show actual trades from valid DEX programs
      if (trade.side === 'price' || trade.amount === 0) {
        return; // Skip price updates
      }
      
      // Only show trades from valid DEX programs
      const validDexPrograms = ['Jupiter v6', 'Jupiter v4', 'Raydium', 'Orca', 'Meteora', 'Phoenix'];
      if (!validDexPrograms.includes(trade.dex_program)) {
        return; // Skip trades from unknown DEX programs
      }
      
      // Filter by selected pair
      if (!matchesPair(trade)) {
        return; // Skip trades that don't match the selected pair
      }
      
      setTrades((prevTrades) => {
        // Avoid duplicates by checking if trade with same ID already exists
        const exists = prevTrades.some(t => t.id === trade.id);
        if (exists) {
          return prevTrades;
        }
        const newTrades = [trade, ...prevTrades].slice(0, 100);
        return newTrades;
      });
    });

    return () => {
      unsubscribe();
    };
  }, [pair, matchesPair]);

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  const formatPrice = (price: number) => {
    return price.toFixed(6);
  };

  const formatAmount = (amount: number) => {
    if (amount === 0) return '0';
    // Use scientific notation for very small numbers
    if (Math.abs(amount) < 0.0001 && amount !== 0) {
      return amount.toExponential(2);
    }
    // For larger numbers, show up to 8 decimal places
    return amount.toFixed(8).replace(/\.?0+$/, '');
  };

  const formatValue = (value: number | undefined) => {
    if (value === undefined || value === null || isNaN(value)) {
      return '0.00';
    }
    if (value === 0) return '0.00';
    // Use scientific notation for very small numbers
    if (Math.abs(value) < 0.01 && value !== 0) {
      return value.toExponential(2);
    }
    // For larger numbers, show 2 decimal places
    return value.toFixed(2);
  };

  const formatSignature = (sig: string) => {
    if (!sig || sig.length < 8) return 'N/A';
    return `${sig.substring(0, 4)}...${sig.substring(sig.length - 4)}`;
  };

  const copySignature = async (signature: string) => {
    try {
      await navigator.clipboard.writeText(signature);
      // You could add a toast notification here if you have a toast system
    } catch (err) {
      console.error('Failed to copy signature:', err);
    }
  };


  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="overflow-y-auto overflow-x-auto flex-1">
        <table className="w-full text-sm">
          <thead className="border-b border-border bg-muted/20 sticky top-0">
            <tr className="text-muted-foreground">
              <th className="text-left py-3 px-4 font-medium">Time</th>
              <th className="text-right py-3 px-4 font-medium">Price</th>
              <th className="text-right py-3 px-4 font-medium">Amount</th>
              <th className="text-right py-3 px-4 font-medium">Value</th>
              <th className="text-right py-3 px-4 font-medium">Side</th>
              <th className="text-right py-3 px-4 font-medium">DEX</th>
              <th className="text-left py-3 px-4 font-medium">Signature</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr>
                <td colSpan={7} className="text-center py-8 text-muted-foreground">
                  Loading trades...
                </td>
              </tr>
            ) : trades.length === 0 ? (
              <tr>
                <td colSpan={7} className="text-center py-8 text-muted-foreground">
                  No trades yet. Waiting for live data...
                </td>
              </tr>
            ) : (
              <>
                {trades.map((trade) => {
                  return (
                    <tr
                      key={trade.id}
                      className="border-b border-border/50 hover:bg-secondary/50 transition-colors"
                    >
                      <td className="py-2 px-4 text-muted-foreground">
                        {formatTime(trade.timestamp)}
                      </td>
                      <td
                        className={cn(
                          'py-2 px-4 text-right font-mono',
                          trade.side === 'buy' ? 'text-success' : 'text-destructive'
                        )}
                      >
                        {formatPrice(trade.price)}
                      </td>
                      <td className="py-2 px-4 text-right font-mono text-foreground">
                        {formatAmount(trade.amount)}
                      </td>
                      <td className="py-2 px-4 text-right font-mono text-foreground">
                        ${formatValue(trade.total_value)}
                      </td>
                      <td className="py-2 px-4 text-right">
                        <span
                          className={cn(
                            'px-2 py-1 rounded text-xs font-medium',
                            trade.side === 'buy'
                              ? 'bg-success/20 text-success'
                              : 'bg-destructive/20 text-destructive'
                          )}
                        >
                          {trade.side?.toUpperCase() || 'N/A'}
                        </span>
                      </td>
                      <td className="py-2 px-4 text-right text-muted-foreground text-xs">
                        {trade.dex_program || 'Unknown'}
                      </td>
                      <td 
                        className="py-2 px-4 text-left font-mono text-xs text-muted-foreground cursor-pointer hover:text-primary transition-colors"
                        onClick={() => copySignature(trade.id)}
                        title="Click to copy signature"
                      >
                        {formatSignature(trade.id)}
                      </td>
                    </tr>
                  );
                })}
              </>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};

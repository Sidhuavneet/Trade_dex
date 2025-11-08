import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { tradeApi, Trade } from '@/lib/api';
import { Loader2, Copy, Check } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

interface TradesModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  pair: string;
}

export const TradesModal = ({ open, onOpenChange, pair }: TradesModalProps) => {
  const [trades, setTrades] = useState<Trade[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copiedSignature, setCopiedSignature] = useState<string | null>(null);
  const { toast } = useToast();

  const fetchTrades = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const fetchedTrades = await tradeApi.getTrades(pair, 100);
      // Filter out price updates and only show actual trades
      // Note: ClickHouse trades have empty dex_program (not stored per assignment)
      // WebSocket trades have valid dex_program, so we allow both
      const validDexPrograms = ['Jupiter v6', 'Jupiter v4', 'Raydium', 'Orca', 'Meteora', 'Phoenix'];
      const filteredTrades = fetchedTrades.filter(trade => 
        trade.side !== 'price' && 
        trade.amount > 0 &&
        // Allow empty dex_program (from ClickHouse) OR valid DEX programs (from WebSocket)
        (trade.dex_program === '' || validDexPrograms.includes(trade.dex_program))
      );
      setTrades(filteredTrades);
      console.log(`✅ TradesModal: Fetched ${fetchedTrades.length} trades, filtered to ${filteredTrades.length} valid trades`);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch trades';
      setError(errorMessage);
      setTrades([]);
      console.error('Error fetching trades:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (open) {
      fetchTrades();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, pair]);

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
    if (Math.abs(amount) < 0.0001 && amount !== 0) {
      return amount.toExponential(2);
    }
    return amount.toFixed(8).replace(/\.?0+$/, '');
  };

  const formatValue = (value: number | undefined) => {
    if (value === undefined || value === null || isNaN(value)) {
      return '0.00';
    }
    if (value === 0) return '0.00';
    if (Math.abs(value) < 0.01 && value !== 0) {
      return value.toExponential(2);
    }
    return value.toFixed(2);
  };

  const formatSignature = (signature: string) => {
    if (!signature) return 'N/A';
    return `${signature.slice(0, 4)}...${signature.slice(-4)}`;
  };

  const copySignature = async (signature: string, e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent row click if any
    try {
      await navigator.clipboard.writeText(signature);
      setCopiedSignature(signature);
      toast({
        title: 'Signature copied!',
        description: 'Transaction signature copied to clipboard',
      });
      // Reset copied state after 2 seconds
      setTimeout(() => setCopiedSignature(null), 2000);
    } catch (err) {
      console.error('Failed to copy signature:', err);
      toast({
        title: 'Failed to copy',
        description: 'Could not copy signature to clipboard',
        variant: 'destructive',
      });
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-6xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>Last 100 Trades - {pair}</DialogTitle>
          <DialogDescription>
            Historical trades from ClickHouse database
          </DialogDescription>
        </DialogHeader>
        
        <div className="flex-1 overflow-hidden flex flex-col">
          {isLoading ? (
            <div className="flex items-center justify-center h-64">
              <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
            </div>
          ) : error ? (
            <div className="flex flex-col items-center justify-center h-64 gap-4">
              <p className="text-destructive">{error}</p>
              <Button onClick={fetchTrades} variant="outline">
                Retry
              </Button>
            </div>
          ) : trades.length === 0 ? (
            <div className="flex items-center justify-center h-64">
              <p className="text-muted-foreground">No trades found</p>
            </div>
          ) : (
            <div className="overflow-y-auto flex-1">
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
                  {trades.map((trade) => (
                    <tr
                      key={trade.id}
                      className="border-b border-border hover:bg-muted/50 transition-colors"
                    >
                      <td className="py-3 px-4 text-muted-foreground">
                        {formatTime(trade.timestamp)}
                      </td>
                      <td className="py-3 px-4 text-right font-medium">
                        ${formatPrice(trade.price)}
                      </td>
                      <td className="py-3 px-4 text-right">
                        {formatAmount(trade.amount)}
                      </td>
                      <td className="py-3 px-4 text-right">
                        ${formatValue(trade.total_value)}
                      </td>
                      <td className="py-3 px-4 text-right">
                        <span
                          className={trade.side === 'buy' ? 'text-green-500' : 'text-red-500'}
                        >
                          {trade.side?.toUpperCase() || 'N/A'}
                        </span>
                      </td>
                      <td className="py-3 px-4 text-right text-muted-foreground">
                        {trade.dex_program || 'ClickHouse'}
                      </td>
                      <td className="py-3 px-4">
                        <div className="flex items-center gap-2 group">
                          <span className="text-muted-foreground font-mono text-xs">
                            {formatSignature(trade.id)}
                          </span>
                          <button
                            onClick={(e) => copySignature(trade.id, e)}
                            className="opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-muted rounded"
                            title="Click to copy signature"
                          >
                            {copiedSignature === trade.id ? (
                              <Check className="w-3 h-3 text-green-500" />
                            ) : (
                              <Copy className="w-3 h-3 text-muted-foreground hover:text-foreground" />
                            )}
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};


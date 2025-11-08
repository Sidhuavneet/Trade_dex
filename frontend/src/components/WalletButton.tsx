import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { Button } from '@/components/ui/button';
import { usePhantomAuth } from '@/hooks/usePhantomAuth';
import { Wallet, LogOut, Check } from 'lucide-react';
import { cn } from '@/lib/utils';

export const WalletButton = () => {
  const { connected, publicKey } = useWallet();
  const { isAuthenticated, authenticate, logout, isLoading } = usePhantomAuth();

  const formatAddress = (address: string) => {
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
  };

  if (!connected) {
    return (
      <WalletMultiButton className="!bg-primary !text-primary-foreground hover:!bg-primary/90 !rounded-md !h-10 !px-4 !font-medium" />
    );
  }

  return (
    <div className="flex items-center gap-2">
      {isAuthenticated ? (
        <>
          <div className="flex items-center gap-2 px-4 py-2 bg-secondary rounded-md border border-border">
            <Check className="w-4 h-4 text-success" />
            <span className="text-sm font-medium">
              {publicKey && formatAddress(publicKey.toBase58())}
            </span>
          </div>
          <Button
            variant="outline"
            onClick={logout}
            className="border-border"
          >
            <LogOut className="w-4 h-4 mr-2" />
            Sign Out
          </Button>
        </>
      ) : (
        <Button
          onClick={authenticate}
          disabled={isLoading}
          className="bg-primary text-primary-foreground hover:bg-primary/90"
        >
          <Wallet className="w-4 h-4 mr-2" />
          {isLoading ? 'Authenticating...' : 'Authenticate'}
        </Button>
      )}
    </div>
  );
};

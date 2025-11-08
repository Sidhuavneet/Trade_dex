import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { Wallet, ArrowRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { usePhantomAuth } from '@/hooks/usePhantomAuth';
import { cn } from '@/lib/utils';

export const LandingPage = () => {
  const { connected } = useWallet();
  const { isAuthenticated, authenticate, isLoading } = usePhantomAuth();

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="max-w-2xl w-full text-center space-y-8">
        {/* Logo/Brand */}
        <div className="space-y-4">
          <div className="flex items-center justify-center gap-3">
            <div className="w-12 h-12 bg-primary rounded-lg flex items-center justify-center">
              <div className="flex flex-col gap-1">
                <div className="w-6 h-0.5 bg-primary-foreground" />
                <div className="w-6 h-0.5 bg-primary-foreground" />
                <div className="w-6 h-0.5 bg-primary-foreground" />
              </div>
            </div>
            <h1 className="text-4xl font-bold">TRADE</h1>
          </div>
          <p className="text-xl text-muted-foreground">
            Real-time Solana Trading Terminal
          </p>
        </div>

        {/* Main Content */}
        <div className="space-y-6">
          <div className="space-y-4">
            <h2 className="text-2xl font-semibold">
              Connect your wallet to get started
            </h2>
            <p className="text-muted-foreground max-w-md mx-auto">
              Connect your Phantom wallet to access live trading data, charts, and real-time market updates.
            </p>
          </div>

          {/* Wallet Connection */}
          <div className="flex flex-col items-center gap-4 pt-4">
            {!connected ? (
              <WalletMultiButton className="!bg-primary !text-primary-foreground hover:!bg-primary/90 !rounded-md !h-12 !px-6 !font-medium !text-base" />
            ) : !isAuthenticated ? (
              <div className="space-y-4 w-full max-w-sm">
                <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
                  <Wallet className="w-4 h-4" />
                  <span>Wallet connected</span>
                </div>
                <Button
                  onClick={authenticate}
                  disabled={isLoading}
                  size="lg"
                  className="w-full bg-primary text-primary-foreground hover:bg-primary/90 h-12 text-base"
                >
                  {isLoading ? (
                    <>
                      <span className="animate-spin mr-2">⏳</span>
                      Authenticating...
                    </>
                  ) : (
                    <>
                      Authenticate
                      <ArrowRight className="ml-2 w-5 h-5" />
                    </>
                  )}
                </Button>
                <p className="text-xs text-muted-foreground">
                  You'll need to sign a message to verify your wallet
                </p>
              </div>
            ) : null}
          </div>

          {/* Features Preview (when not authenticated) */}
          {!isAuthenticated && (
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-12 pt-8 border-t border-border">
            <div className="space-y-2">
              <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center mx-auto">
                <svg
                  className="w-5 h-5 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
              </div>
              <h3 className="font-semibold">Live Trading</h3>
              <p className="text-sm text-muted-foreground">
                Real-time trade updates
              </p>
            </div>
            <div className="space-y-2">
              <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center mx-auto">
                <svg
                  className="w-5 h-5 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                  />
                </svg>
              </div>
              <h3 className="font-semibold">TradingView Charts</h3>
              <p className="text-sm text-muted-foreground">
                Professional charting tools
              </p>
            </div>
            <div className="space-y-2">
              <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center mx-auto">
                <svg
                  className="w-5 h-5 text-primary"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                  />
                </svg>
              </div>
              <h3 className="font-semibold">Secure Auth</h3>
              <p className="text-sm text-muted-foreground">
                Phantom wallet integration
              </p>
            </div>
          </div>
          )}
        </div>
      </div>
    </div>
  );
};


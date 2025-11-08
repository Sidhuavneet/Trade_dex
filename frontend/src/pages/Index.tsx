import { useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { usePhantomAuth } from '@/hooks/usePhantomAuth';
import { LandingPage } from '@/components/LandingPage';
import { Header } from '@/components/Header';
import { PriceStats } from '@/components/PriceStats';
import { TradingChart } from '@/components/TradingChart';
import { TabsPanel } from '@/components/TabsPanel';
import { tradeWebSocket } from '@/lib/websocket';
import { useToast } from '@/hooks/use-toast';

const Index = () => {
  const { connected } = useWallet();
  const { isAuthenticated } = usePhantomAuth();
  const [selectedPair, setSelectedPair] = useState('SOL/USDC');
  const { toast } = useToast();

  // Check localStorage as fallback to ensure we have the latest auth state
  const [authChecked, setAuthChecked] = useState(false);
  
  useEffect(() => {
    // Double-check localStorage for authentication
    const token = localStorage.getItem('auth_token');
    const expiresAt = localStorage.getItem('auth_expires_at');
    const hasValidToken = token && expiresAt && new Date(expiresAt) > new Date();
    
    console.log('🔍 Index.tsx - Auth check:', {
      isAuthenticated,
      connected,
      hasValidToken,
      token: token ? 'exists' : 'missing',
    });
    
    setAuthChecked(true);
  }, [isAuthenticated, connected]);

  // Connect WebSocket when authenticated and connected
  useEffect(() => {
    if (isAuthenticated && connected) {
      // Connect WebSocket if not already connected
      if (!tradeWebSocket.isConnected()) {
        tradeWebSocket.connect();
      }
    } else {
      // Disconnect if not authenticated or wallet disconnected
      tradeWebSocket.disconnect();
    }

    return () => {
      // Only disconnect on unmount or when auth/wallet state changes
      // Don't disconnect when pair changes
      if (!isAuthenticated || !connected) {
        tradeWebSocket.disconnect();
      }
    };
  }, [isAuthenticated, connected]); // Removed selectedPair from dependencies

  // WebSocket connection status monitoring and pair selection
  useEffect(() => {
    if (!isAuthenticated || !connected) {
      return;
    }

    // Subscribe to WebSocket connection status
    const unsubscribe = tradeWebSocket.onConnection((wsConnected) => {
      if (wsConnected) {
        toast({
          title: 'WebSocket Connected',
          description: 'Real-time trade data is now streaming',
        });
        // Send current pair selection immediately when WebSocket connects
        // Use a small delay to ensure WebSocket is fully ready
        setTimeout(() => {
          if (tradeWebSocket.isConnected()) {
            tradeWebSocket.sendPairSelection(selectedPair);
            console.log('📤 [Index] Sent initial pair selection on connect:', selectedPair);
          }
        }, 100);
      } else {
        toast({
          title: 'WebSocket Disconnected',
          description: 'Reconnecting to trade stream...',
          variant: 'destructive',
        });
      }
    });

    // Also send pair selection if WebSocket is already connected
    if (tradeWebSocket.isConnected()) {
      setTimeout(() => {
        if (tradeWebSocket.isConnected()) {
          tradeWebSocket.sendPairSelection(selectedPair);
          console.log('📤 [Index] Sent pair selection (already connected):', selectedPair);
        }
      }, 100);
    }

    return () => {
      unsubscribe();
    };
  }, [isAuthenticated, connected, toast, selectedPair]);

  // Send pair selection to backend when pair changes
  useEffect(() => {
    if (isAuthenticated && connected) {
      // Always try to send pair selection, even if WebSocket is not connected yet
      // The WebSocket class will queue it if not ready
      tradeWebSocket.sendPairSelection(selectedPair);
      console.log('📤 [Index] Attempting to send pair selection on change:', selectedPair);
    }
  }, [selectedPair, isAuthenticated, connected]);

  // Show landing page if not authenticated or wallet not connected
  // Use both hook state and localStorage check for reliability
  const token = localStorage.getItem('auth_token');
  const expiresAt = localStorage.getItem('auth_expires_at');
  const hasValidToken = token && expiresAt && new Date(expiresAt) > new Date();
  const shouldShowDashboard = (isAuthenticated || hasValidToken) && connected;

  if (!shouldShowDashboard) {
    console.log('🚫 Showing LandingPage - isAuthenticated:', isAuthenticated, 'connected:', connected, 'hasValidToken:', hasValidToken);
    return <LandingPage />;
  }

  // Show full trading interface when authenticated and connected
  return (
    <div className="min-h-screen bg-background flex flex-col">
      <Header selectedPair={selectedPair} onPairChange={setSelectedPair} />
      
      <PriceStats pair={selectedPair} />

      <div className="flex-1 flex flex-col">
        {/* Chart Area - Fixed Height */}
        <div className="h-[500px] border-b border-border bg-card">
          <TradingChart pair={selectedPair} interval="1m" />
        </div>

        {/* Trades Panel - Takes remaining space */}
        <div className="flex-1 bg-card overflow-hidden">
          <TabsPanel pair={selectedPair} />
        </div>
      </div>
    </div>
  );
};

export default Index;

import { useCallback, useEffect, useState } from 'react';
import { flushSync } from 'react-dom';
import { useNavigate } from 'react-router-dom';
import { useWallet } from '@solana/wallet-adapter-react';
import bs58 from 'bs58';
import { authApi } from '@/lib/api';
import { toast } from '@/hooks/use-toast';
import { tradeWebSocket } from '@/lib/websocket';

interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  isLoading: boolean;
  nonce: string | null;
}

export const usePhantomAuth = () => {
  const { publicKey, signMessage, connected } = useWallet();
  const navigate = useNavigate();
  const [authState, setAuthState] = useState<AuthState>({
    isAuthenticated: false,
    token: null,
    isLoading: false,
    nonce: null,
  });

  // Track if this is the initial mount (page load)
  const [isInitialMount, setIsInitialMount] = useState(true);

  // Check if token exists in localStorage on mount
  useEffect(() => {
    const token = localStorage.getItem('auth_token');
    const expiresAt = localStorage.getItem('auth_expires_at');
    
    if (token && expiresAt) {
      const isExpired = new Date(expiresAt) < new Date();
      if (!isExpired) {
        setAuthState((prev) => ({
          ...prev,
          isAuthenticated: true,
          token,
          isLoading: false,
        }));
      } else {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('auth_expires_at');
      }
    }
    
    // Mark that initial mount is complete after a short delay
    // This gives Phantom wallet time to reconnect
    const timer = setTimeout(() => {
      setIsInitialMount(false);
    }, 2000); // Wait 2 seconds for wallet to reconnect
    
    return () => clearTimeout(timer);
  }, []);

  // Automatically fetch nonce when wallet connects
  useEffect(() => {
    if (connected && publicKey && !authState.nonce && !authState.isAuthenticated && !authState.isLoading) {
      const fetchNonce = async () => {
        try {
          setAuthState((prev) => ({ ...prev, isLoading: true }));
          const { nonce } = await authApi.getNonce();
          setAuthState((prev) => ({ ...prev, nonce, isLoading: false }));
          console.log('✅ Nonce received:', nonce);
        } catch (error) {
          console.error('❌ Failed to fetch nonce:', error);
          setAuthState((prev) => ({ ...prev, isLoading: false }));
          toast({
            title: 'Failed to fetch nonce',
            description: error instanceof Error ? error.message : 'Please try again',
            variant: 'destructive',
          });
        }
      };

      fetchNonce();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connected, publicKey]);

  const authenticate = useCallback(async () => {
    if (!publicKey || !signMessage || !connected) {
      toast({
        title: 'Wallet not connected',
        description: 'Please connect your Phantom wallet first',
        variant: 'destructive',
      });
      return;
    }

    setAuthState((prev) => ({ ...prev, isLoading: true }));

    try {
      // Step 1: Get nonce (use existing if available, otherwise fetch new)
      let nonce = authState.nonce;
      if (!nonce) {
        const nonceResponse = await authApi.getNonce();
        nonce = nonceResponse.nonce;
        setAuthState((prev) => ({ ...prev, nonce }));
      }

      console.log('📝 Using nonce for signing:', nonce);

      // Step 2: Create message to sign
      const message = new TextEncoder().encode(
        `Sign this message to authenticate with Trade: ${nonce}`
      );

      console.log('✍️ Requesting signature from Phantom wallet...');

      // Step 3: Sign message with Phantom wallet
      const signature = await signMessage(message);

      console.log('✅ Signature received:', {
        publicKey: publicKey.toBase58(),
        signatureLength: signature.length,
        signaturePreview: bs58.encode(signature).substring(0, 20) + '...',
      });

      // Step 4: Verify signature with backend
      const response = await authApi.verifySignature({
        publicKey: publicKey.toBase58(),
        signature: bs58.encode(signature),
        nonce,
      });

      console.log('📥 Backend response:', response);

      // Extract token and expiresAt
      const token = response.token;
      const expiresAt = response.expiresAt;

      if (!token || !expiresAt) {
        throw new Error('Invalid response from server: missing token or expiresAt');
      }

      console.log('🎉 Authentication successful!', {
        tokenPreview: token.substring(0, 20) + '...',
        expiresAt,
      });

      // Step 5: Store token
      localStorage.setItem('auth_token', token);
      localStorage.setItem('auth_expires_at', expiresAt);

      // Update state immediately and force synchronous render
      flushSync(() => {
        setAuthState({
          isAuthenticated: true,
          token,
          isLoading: false,
          nonce: null, // Clear nonce after successful auth
        });
      });

      console.log('✅ Auth state updated - isAuthenticated: true');
      console.log('🔍 Current auth state after update:', {
        isAuthenticated: true,
        token: token ? 'exists' : 'missing',
      });

      toast({
        title: 'Authentication successful',
        description: 'You are now connected to Trade',
      });

      // Step 6: Automatically establish WebSocket connection
      console.log('🔌 Establishing WebSocket connection...');
      tradeWebSocket.connect();
      console.log('WebSocket connected');
    } catch (error) {
      console.error('❌ Authentication failed:', error);
      setAuthState({
        isAuthenticated: false,
        token: null,
        isLoading: false,
        nonce: null, // Clear nonce on error
      });
      
      toast({
        title: 'Authentication failed',
        description: error instanceof Error ? error.message : 'Please try again',
        variant: 'destructive',
      });
    }
  }, [publicKey, signMessage, connected, authState.nonce]);

  const logout = useCallback(() => {
    // Disconnect WebSocket first
    tradeWebSocket.disconnect();
    
    // Clear authentication state
    localStorage.removeItem('auth_token');
    localStorage.removeItem('auth_expires_at');
    
    // Update state immediately and force synchronous render
    flushSync(() => {
      setAuthState({
        isAuthenticated: false,
        token: null,
        isLoading: false,
        nonce: null,
      });
    });
    
    toast({
      title: 'Signed out',
      description: 'You have been successfully signed out',
    });
    
    // Force navigation to home page to trigger landing page display
    navigate('/', { replace: true });
  }, [navigate]);

  // Clear authentication if wallet disconnects
  // Don't clear auth during initial mount (page reload) - only clear on explicit disconnect
  useEffect(() => {
    // Only clear auth if wallet disconnects AFTER initial mount is complete
    // This prevents clearing auth during page reload when wallet is still reconnecting
    if (!connected && authState.isAuthenticated && !isInitialMount) {
      setAuthState({
        isAuthenticated: false,
        token: null,
        isLoading: false,
        nonce: null,
      });
      localStorage.removeItem('auth_token');
      localStorage.removeItem('auth_expires_at');
    }
    
    // Clear nonce when wallet disconnects
    if (!connected && authState.nonce) {
      setAuthState((prev) => ({ ...prev, nonce: null }));
    }
  }, [connected, authState.isAuthenticated, authState.nonce, isInitialMount]);

  // Debug: Log what we're returning
  useEffect(() => {
    console.log('🔍 usePhantomAuth - Returning state:', {
      isAuthenticated: authState.isAuthenticated,
      token: authState.token ? 'exists' : 'null',
      isLoading: authState.isLoading,
    });
  }, [authState.isAuthenticated, authState.token, authState.isLoading]);

  return {
    isAuthenticated: authState.isAuthenticated,
    token: authState.token,
    isLoading: authState.isLoading,
    authenticate,
    logout,
  };
};

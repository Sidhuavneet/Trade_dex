# Architecture Documentation

## System Overview

This frontend application is a Solana trading terminal that provides real-time trade tracking, interactive charts, and secure wallet authentication. It's designed to integrate with a Rust backend that uses ClickHouse for data persistence.

## Component Architecture

### 1. Application Entry Point

```
App.tsx (Root)
├── WalletContextProvider (Solana Wallet Integration)
├── QueryClientProvider (React Query)
├── TooltipProvider (Radix UI)
└── BrowserRouter (React Router)
    └── Index.tsx (Main Trading Page)
```

### 2. Main Trading Page Structure

```
Index.tsx
├── Header
│   ├── Logo
│   ├── PairSelector
│   └── WalletButton
├── PriceStats
│   └── Real-time statistics (price, 24h change, volume, high, low)
├── TradingChart (2/3 width)
│   └── TradingView Lightweight Charts
└── TabsPanel (1/3 width)
    ├── Trades Tab → TradesTable
    ├── Open Orders Tab
    └── Order History Tab
```

## Core Modules

### Authentication Module (`usePhantomAuth`)

**Purpose**: Manages Phantom wallet authentication lifecycle

**Flow**:
```typescript
1. User connects wallet → Solana Wallet Adapter handles connection
2. User clicks "Authenticate" → usePhantomAuth.authenticate()
3. Hook requests nonce → GET /auth/nonce
4. Hook creates message → `Sign this message to authenticate: ${nonce}`
5. User signs via Phantom → signMessage(encoded message)
6. Hook sends to backend → POST /auth/verify { publicKey, signature, nonce }
7. Backend verifies signature → Returns JWT token
8. Hook stores token → localStorage + state update
```

**State Management**:
- `isAuthenticated`: Boolean flag for auth status
- `token`: JWT token string
- `isLoading`: Loading state during authentication

**Storage**:
- Token stored in localStorage as `auth_token`
- Expiry stored as `auth_expires_at`
- Auto-validates on app load

### WebSocket Module (`lib/websocket.ts`)

**Purpose**: Real-time trade streaming with auto-reconnect

**Features**:
- Singleton pattern for single connection
- Multiple subscribers support
- Automatic reconnection with exponential backoff
- Graceful disconnect handling

**Usage**:
```typescript
// Connect (done in Index.tsx on mount)
tradeWebSocket.connect();

// Subscribe to trades
const unsubscribe = tradeWebSocket.subscribe((trade) => {
  console.log('New trade:', trade);
});

// Cleanup
unsubscribe();
tradeWebSocket.disconnect();
```

**Reconnection Logic**:
- Max attempts: 5
- Delay: 3000ms
- Resets counter on successful connection

### API Module (`lib/api.ts`)

**Purpose**: Centralized REST API communication

**Endpoints**:

1. **Authentication**
```typescript
authApi.getNonce() → { nonce: string }
authApi.verifySignature({ publicKey, signature, nonce }) → { token, expiresAt }
```

2. **Trading Data**
```typescript
tradeApi.getTrades(pair, limit) → Trade[]
tradeApi.getOHLCV(pair, interval) → OHLCVData[]
```

**Configuration**:
- Base URL from `VITE_API_BASE_URL`
- All endpoints use JSON content-type
- Error handling with meaningful messages

## Data Flow Diagrams

### Authentication Flow

```
┌──────────┐     ┌─────────────┐     ┌─────────┐     ┌─────────┐
│  User    │────▶│   Phantom   │────▶│Frontend │────▶│ Backend │
│          │     │   Wallet    │     │         │     │         │
└──────────┘     └─────────────┘     └─────────┘     └─────────┘
     │                  │                   │              │
     │  1. Connect      │                   │              │
     │─────────────────▶│                   │              │
     │                  │                   │              │
     │  2. Authenticate │                   │              │
     │──────────────────┼──────────────────▶│              │
     │                  │                   │ 3. Get Nonce │
     │                  │                   │─────────────▶│
     │                  │                   │◀─────────────│
     │                  │  4. Sign Message  │              │
     │                  │◀──────────────────│              │
     │                  │──────────────────▶│              │
     │                  │                   │ 5. Verify    │
     │                  │                   │─────────────▶│
     │                  │                   │◀─────────────│
     │                  │  6. Store Token   │  JWT Token   │
     │◀─────────────────┼───────────────────│              │
```

### Trade Streaming Flow

```
┌─────────┐         ┌──────────┐         ┌────────────┐
│ Backend │────────▶│WebSocket │────────▶│ Components │
└─────────┘         └──────────┘         └────────────┘
     │                    │                      │
     │ New Trade          │                      │
     │───────────────────▶│                      │
     │                    │ Broadcast            │
     │                    │─────────────────────▶│
     │                    │                      │
     │                    │                      │ Update UI:
     │                    │                      │ - TradesTable
     │                    │                      │ - PriceStats
     │                    │                      │ - Chart (future)
```

### Chart Data Flow

```
┌──────────┐      ┌─────────┐      ┌───────────┐      ┌────────────┐
│   User   │─────▶│  Index  │─────▶│TradingChart│─────▶│ API Client │
└──────────┘      └─────────┘      └───────────┘      └────────────┘
     │                 │                  │                    │
     │ Select Pair     │                  │                    │
     │────────────────▶│                  │                    │
     │                 │  Pass Pair       │                    │
     │                 │─────────────────▶│                    │
     │                 │                  │  GET /api/ohlcv    │
     │                 │                  │───────────────────▶│
     │                 │                  │◀───────────────────│
     │                 │                  │  OHLCV Data        │
     │                 │                  │                    │
     │                 │                  │  Render Chart      │
     │                 │◀─────────────────│                    │
```

## State Management

### Global State
- **Wallet Context**: Manages Solana wallet connection state
- **React Query**: Caches API responses (ready for use)

### Component State
- **Index.tsx**: Selected trading pair
- **TradesTable**: Array of recent trades
- **PriceStats**: Calculated 24h statistics
- **TradingChart**: Chart instance and loading state
- **usePhantomAuth**: Authentication state and token

### Local Storage
- `auth_token`: JWT for authenticated requests
- `auth_expires_at`: Token expiration timestamp

## Design System

### Color Tokens (HSL)
```css
--background: 240 10% 3.9%     /* Dark background */
--foreground: 0 0% 98%         /* Light text */
--primary: 158 64% 52%         /* Green accent */
--secondary: 240 6% 10%        /* Dark secondary */
--success: 142 76% 36%         /* Green for buy */
--destructive: 0 72% 51%       /* Red for sell */
--border: 240 6% 12%           /* Subtle borders */
```

### Typography
- **Font Family**: System font stack (default)
- **Monospace**: Used for prices and amounts
- **Font Sizes**: Tailwind's default scale

### Spacing
- **Layout**: Grid system with Tailwind utilities
- **Padding**: Consistent 4px increments
- **Gaps**: 2-4 units between elements

## Performance Considerations

### Optimization Strategies

1. **Component Memoization**
   - Charts re-render only on pair/interval change
   - Tables use virtual scrolling (can be added)
   - Stats recalculate only on new trades

2. **Data Management**
   - Trades list limited to 100 most recent
   - WebSocket reconnect with backoff prevents spam
   - Chart data fetched once per pair/interval

3. **Bundle Optimization**
   - Vite's code splitting
   - Tree-shaking for unused code
   - Lazy loading for routes (can be added)

### Potential Improvements

1. **Virtual Scrolling**: For large trade lists
2. **Worker Threads**: For heavy calculations
3. **Debounced Updates**: Batch WebSocket messages
4. **Chart Data Aggregation**: Client-side OHLC calculation

## Security Considerations

### Authentication Security

1. **Nonce-based Auth**
   - Random nonce prevents replay attacks
   - Single-use nonce enforced by backend
   - Message includes context string

2. **Token Management**
   - JWT stored in localStorage (XSS risk mitigated by CSP)
   - Token expiry checked on app load
   - Automatic logout on expiry

3. **Wallet Security**
   - User always in control of private keys
   - Phantom handles all signing
   - No private key exposure to app

### Best Practices

- HTTPS only in production
- CSP headers for XSS protection
- No sensitive data in console logs (production)
- Input validation on user inputs
- CORS configuration on backend

## Error Handling

### Strategy
1. **API Errors**: Try-catch with user-friendly toast messages
2. **WebSocket Errors**: Auto-reconnect with user notification
3. **Auth Errors**: Clear state and prompt re-authentication
4. **Chart Errors**: Display loading state with error message

### User Feedback
- Toast notifications for errors
- Loading states for async operations
- Empty states with helpful messages
- Connection status indicators

## Testing Strategy

### Unit Tests (To be added)
- API client functions
- WebSocket reconnect logic
- Authentication flow
- Price calculation utilities

### Integration Tests (To be added)
- Component interactions
- WebSocket message handling
- Chart data rendering
- Auth flow end-to-end

### E2E Tests (To be added)
- Complete trading workflow
- Wallet connection and auth
- Live trade updates
- Chart interactions

## Deployment

### Environment Configuration
```bash
# Development
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_BASE_URL=ws://localhost:8080

# Production
VITE_API_BASE_URL=https://api.yourdomain.com
VITE_WS_BASE_URL=wss://api.yourdomain.com
```

### Build Process
```bash
npm run build
# Outputs to dist/
# Static files ready for CDN
```

### Hosting Options
- **Vercel**: Zero-config deployment
- **Netlify**: Automatic CI/CD
- **AWS S3 + CloudFront**: Scalable static hosting
- **Cloudflare Pages**: Edge network deployment

## Browser Compatibility

### Requirements
- Modern browsers with ES2020 support
- WebSocket support
- LocalStorage support
- Phantom Wallet extension (Chrome, Firefox, Brave, Edge)

### Tested Browsers
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Brave 1.24+

## Monitoring & Logging

### Client-Side Logging
- Console errors for debugging
- WebSocket connection status
- API request failures
- Auth state changes

### Future Enhancements
- Sentry for error tracking
- Analytics for user behavior
- Performance monitoring
- Real-time alerts

## API Versioning

### Current Version: v1
- Base path: `/api/`
- Auth path: `/auth/`
- WebSocket path: `/ws/`

### Future Considerations
- API version in URL: `/api/v2/`
- Backward compatibility layer
- Deprecation warnings
- Migration guides

## Scalability

### Current Capacity
- WebSocket: Single connection for all trades
- Trades buffer: 100 most recent
- Chart data: Full history per pair

### Scaling Strategies
1. **WebSocket**: Room-based subscriptions per pair
2. **Caching**: Redis for OHLCV data
3. **CDN**: Static assets on edge network
4. **API**: Rate limiting and pagination

## Dependencies

### Core Dependencies
```json
{
  "@solana/wallet-adapter-react": "^0.15.x",
  "@solana/web3.js": "^1.x",
  "lightweight-charts": "^5.0.x",
  "react": "^18.3.x",
  "bs58": "^5.x"
}
```

### Development Dependencies
```json
{
  "typescript": "^5.x",
  "vite": "^5.x",
  "tailwindcss": "^3.x"
}
```

### Upgrade Path
- Regular updates for security patches
- Major version updates with testing
- Dependency audit monthly

---

**Document Version**: 1.0  
**Last Updated**: 2025-11-04  
**Maintained By**: Development Team

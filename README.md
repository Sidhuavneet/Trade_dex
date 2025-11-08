# Solana Trade Tracker

A real-time Solana trading terminal clone built with Rust backend and React frontend, featuring live trade data streaming via WebSocket, ClickHouse persistence, TradingView charts, and Phantom wallet authentication.

## 🎯 Overview

This project is a full-stack Solana trade tracking application that:

- **Streams live Solana trade data** via QuickNode WebSocket subscriptions
- **Persists trades** to ClickHouse for historical analysis
- **Displays real-time charts** using TradingView Lightweight Charts
- **Authenticates users** via Phantom wallet with signature verification
- **Monitors multiple DEXs** including Jupiter, Raydium, Orca, Meteora, and Phoenix
- **Supports multiple trading pairs** including SOL/USDC, BONK/SOL, JUP/SOL, WIF/SOL, RAY/SOL

## 🏗️ Architecture

### System Architecture

```
┌─────────────────┐
│   Frontend      │
│   (React)       │
│                 │
│  - TradingView  │
│  - WebSocket    │
│  - Phantom Auth │
└────────┬────────┘
         │
         │ HTTP/WS
         │
┌────────▼─────────────────┐
│   Backend (Rust)        │
│                         │
│  ┌──────────────────┐   │
│  │ QuickNode WS     │   │
│  │ Subscription     │   │
│  └────────┬─────────┘   │
│           │             │
│  ┌────────▼─────────┐   │
│  │ Trade Stream      │   │
│  │ Service           │   │
│  └────────┬─────────┘   │
│           │             │
│  ┌────────▼─────────┐   │
│  │ WebSocket        │   │
│  │ Manager          │   │
│  └────────┬─────────┘   │
│           │             │
│  ┌────────▼─────────┐     │
│  │ ClickHouse      │     │
│  │ Service         │     │
│  └────────┬───────┘     │
└───────────┼──────────────┘
            │
            │
    ┌───────▼────────┐
    │  ClickHouse    │
    │  Database      │
    └────────────────┘
```

### Data Flow

1. **Trade Ingestion**: QuickNode WebSocket → Backend → ClickHouse + WebSocket Broadcast
2. **Price Updates**: Jupiter API → Backend → WebSocket Broadcast
3. **Chart Data**: ClickHouse → Backend API → Frontend Chart
4. **Authentication**: Phantom Wallet → Backend → JWT Token → ClickHouse Sessions

## ✨ Key Features

### Frontend Features
- ✅ **Real-time Trade Streaming** - Live trade updates via WebSocket
- ✅ **TradingView Charts** - Interactive candlestick charts with OHLC data
- ✅ **Phantom Wallet Authentication** - Secure nonce-based signature verification
- ✅ **Live Price Statistics** - 24h change, volume, high/low tracking
- ✅ **Multi-Pair Support** - Switch between SOL/USDC, BONK/SOL, JUP/SOL, WIF/SOL, RAY/SOL
- ✅ **Dark Theme UI** - Modern, professional interface matching Trade design
- ✅ **Responsive Design** - Works on desktop and mobile devices
- ✅ **Auto-scaling Charts** - Handles both high-value (SOL/USDC ~$160) and low-value (BONK/SOL ~$0.0002) pairs

### Backend Features
- ✅ **QuickNode WebSocket Integration** - Real-time Solana transaction monitoring
- ✅ **Multi-DEX Support** - Monitors Jupiter v6/v4, Raydium, Orca, Meteora, Phoenix
- ✅ **Jupiter API Integration** - Real-time price fetching for all supported pairs
- ✅ **ClickHouse Persistence** - Stores all trades for historical analysis
- ✅ **OHLC Aggregation** - Supports 1m, 5m, 15m, 1h, 4h, 1d intervals
- ✅ **REST API Endpoints** - `/api/trades` for historical data
- ✅ **WebSocket Broadcasting** - Real-time trade and price updates to connected clients
- ✅ **Phantom Authentication** - ed25519 signature verification with JWT tokens
- ✅ **Session Management** - ClickHouse-backed session storage

## 🛠️ Tech Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **TailwindCSS** - Styling
- **Lightweight Charts 5.0** - TradingView-style charts
- **Solana Wallet Adapter** - Phantom wallet integration
- **@solana/web3.js** - Solana blockchain interaction
- **Radix UI** - Accessible UI components

### Backend
- **Rust** - System programming language
- **Tokio** - Async runtime
- **Axum** - Web framework
- **ClickHouse** - Columnar database for time-series data
- **ed25519-dalek** - Cryptography for signature verification
- **jsonwebtoken** - JWT token generation
- **tokio-tungstenite** - WebSocket client/server
- **reqwest** - HTTP client for Jupiter API
- **chrono** - Date/time handling

### Infrastructure
- **ClickHouse Cloud** - Managed ClickHouse database
- **QuickNode** - Solana RPC provider
- **Jupiter API** - Token price data
- **Docker Compose** - Local development environment

## 📁 Project Structure

```
.
├── backend/
│   ├── src/
│   │   ├── main.rs              # Application entry point
│   │   ├── routes/              # API route handlers
│   │   │   ├── auth.rs          # Authentication endpoints
│   │   │   └── trades.rs        # Trade data endpoints
│   │   ├── services/            # Business logic
│   │   │   ├── clickhouse.rs    # ClickHouse database operations
│   │   │   ├── jupiter.rs       # Jupiter API integration
│   │   │   ├── quicknode_ws.rs  # QuickNode WebSocket subscription
│   │   │   ├── solana.rs        # Solana RPC client
│   │   │   ├── trade_stream.rs  # Trade stream orchestration
│   │   │   └── pair_mapping.rs  # Pair symbol/mint mapping
│   │   ├── models/              # Data models
│   │   │   ├── trade.rs         # Trade struct
│   │   │   └── auth.rs          # Auth models
│   │   ├── websocket/           # WebSocket handlers
│   │   │   ├── manager.rs       # Connection management
│   │   │   └── handler.rs       # WebSocket handler
│   │   ├── middleware/          # HTTP middleware
│   │   │   ├── auth.rs          # JWT authentication
│   │   │   └── cors.rs          # CORS configuration
│   │   └── utils/               # Utility functions
│   │       └── jwt.rs           # JWT token utilities
│   ├── Cargo.toml               # Rust dependencies
│   └── Dockerfile               # Docker image for backend
│
├── frontend/
│   ├── src/
│   │   ├── components/          # React components
│   │   │   ├── Header.tsx       # Top navigation
│   │   │   ├── PriceStats.tsx   # 24h price statistics
│   │   │   ├── TradingChart.tsx # TradingView chart
│   │   │   ├── TradesTable.tsx  # Live trades list
│   │   │   ├── PairSelector.tsx # Trading pair selector
│   │   │   ├── TradesModal.tsx  # Historical trades modal
│   │   │   └── WalletButton.tsx # Phantom wallet button
│   │   ├── pages/               # Page components
│   │   │   ├── Index.tsx        # Main trading page
│   │   │   └── LandingPage.tsx # Landing page
│   │   ├── lib/                 # Utilities
│   │   │   ├── api.ts           # REST API client
│   │   │   ├── websocket.ts     # WebSocket client
│   │   │   └── ohlc.ts          # OHLC aggregation
│   │   ├── hooks/               # React hooks
│   │   │   └── usePhantomAuth.ts # Phantom auth hook
│   │   └── contexts/            # React contexts
│   │       └── WalletContext.tsx # Wallet provider
│   ├── package.json             # Node dependencies
│   ├── Dockerfile               # Docker image for frontend
│   └── nginx.conf               # Nginx configuration
│
├── docker-compose.yml           # Docker Compose configuration
└── README.md                    # This file
```

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.70+ (for backend)
- **Node.js** 18+ and npm (for frontend)
- **Docker** and Docker Compose (optional, for local development)
- **QuickNode Account** - For Solana RPC access
- **ClickHouse Cloud Account** - For database (or use local ClickHouse)
- **Phantom Wallet** - Browser extension for authentication

### Environment Variables

#### Backend (.env)

```env
# QuickNode RPC URL (WebSocket endpoint)
QUICKNODE_RPC_URL=wss://your-endpoint.solana-mainnet.quiknode.pro/your-api-key/

# ClickHouse Configuration
CLICKHOUSE_URL=https://your-instance.clickhouse.cloud:8443
CLICKHOUSE_USERNAME=default
CLICKHOUSE_PASSWORD=your-password

# JWT Secret (change in production)
JWT_SECRET=your-secret-key-change-in-production
```

#### Frontend (.env)

```env
# Backend API URL
VITE_API_BASE_URL=http://localhost:3000
VITE_WS_BASE_URL=ws://localhost:3000

# Solana Network
VITE_SOLANA_NETWORK=mainnet-beta
```

### Installation

#### Option 1: Docker Compose (Recommended)

1. **Clone the repository:**
```bash
git clone <repository-url>
cd solana-trade-tracker
```

2. **Create `.env` file:**
```bash
cp .env.example .env
# Edit .env with your QuickNode and ClickHouse credentials
# For frontend URLs, use:
#   - localhost:3000 if accessing from the same machine
#   - your-machine-ip:3000 if accessing from a different machine
```

3. **Start all services:**
```bash
docker-compose up -d
```

4. **Access the application:**
- Frontend: http://localhost:8080
- Backend API: http://localhost:3000
- ClickHouse: http://localhost:8123

#### Option 2: Manual Setup

**Backend:**

1. **Navigate to backend directory:**
```bash
cd backend
```

2. **Create `.env` file:**
```bash
cp .env.example .env
# Edit .env with your credentials
```

3. **Build and run:**
```bash
cargo build --release
cargo run
```

**Frontend:**

1. **Navigate to frontend directory:**
```bash
cd frontend
```

2. **Install dependencies:**
```bash
npm install
```

3. **Create `.env` file:**
```bash
cp .env.example .env
# Edit .env with backend URL
```

4. **Start development server:**
```bash
npm run dev
```

5. **Open browser:**
```
http://localhost:5173
```

## 🔐 Authentication Flow

The application implements a secure Phantom wallet authentication flow:

### Flow Diagram

```
┌──────────┐         ┌──────────┐         ┌──────────┐
│  Client  │         │ Backend  │         │ Phantom  │
└────┬─────┘         └────┬─────┘         └────┬─────┘
     │                    │                    │
     │ 1. GET /auth/nonce │                    │
     │───────────────────>│                    │
     │                    │                    │
     │ 2. nonce           │                    │
     │<───────────────────│                    │
     │                    │                    │
     │ 3. Sign nonce      │                    │
     │────────────────────────────────────────>│
     │                    │                    │
     │ 4. signature       │                    │
     │<────────────────────────────────────────│
     │                    │                    │
     │ 5. POST /auth/verify                    │
     │    {publicKey, signature, nonce}       │
     │───────────────────>│                    │
     │                    │                    │
     │                    │ 6. Verify ed25519  │
     │                    │    signature       │
     │                    │                    │
     │                    │ 7. Store session   │
     │                    │    in ClickHouse   │
     │                    │                    │
     │ 8. JWT token       │                    │
     │<───────────────────│                    │
     │                    │                    │
```

### Implementation Details

1. **Client requests nonce** from backend (`GET /auth/nonce`)
2. **Backend generates random nonce** and returns it
3. **User signs nonce** using Phantom wallet
4. **Client sends signature + public key** to backend (`POST /auth/verify`)
5. **Backend verifies signature** using ed25519 cryptography
6. **Backend stores session** in ClickHouse with expiration
7. **Backend returns JWT token** for authenticated requests
8. **Client stores JWT** in localStorage for subsequent requests

### API Endpoints

**GET /auth/nonce**
```json
Response:
{
  "nonce": "random-string-here"
}
```

**POST /auth/verify**
```json
Request:
{
  "publicKey": "Base58-encoded-public-key",
  "signature": "Base58-encoded-signature",
  "nonce": "nonce-from-step-1"
}

Response:
{
  "token": "JWT-token-here",
  "expiresAt": "2024-01-01T00:00:00Z"
}
```

## 📡 API Endpoints

### Trade Endpoints

**GET /api/trades**
- Get recent trades filtered by pair
- Query parameters:
  - `pair` (required): Trading pair (e.g., "SOL/USDC")
  - `limit` (optional): Number of trades to return (default: 100)
- Example: `GET /api/trades?pair=SOL/USDC&limit=100`

**GET /api/ohlcv**
- Get OHLCV (Open, High, Low, Close, Volume) data for charts
- Query parameters:
  - `pair` (required): Trading pair (e.g., "SOL/USDC")
  - `interval` (optional): Time interval (1m, 5m, 15m, 1h, 4h, 1d) (default: 1m)
- Example: `GET /api/ohlcv?pair=SOL/USDC&interval=1m`

### WebSocket Endpoint

**WS /ws/trades**
- Real-time trade and price updates
- Message format:
```json
{
  "id": "transaction-signature",
  "timestamp": "2024-01-01T00:00:00Z",
  "base_symbol": "SOL",
  "quote_symbol": "USDC",
  "price": 160.50,
  "amount": 1.5,
  "side": "buy",
  "base_mint": "So11111111111111111111111111111111111111112",
  "quote_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "total_value": 240.75,
  "dex_program": "Jupiter v6",
  "slot": 123456789
}
```

**Client Messages:**
```json
{
  "type": "select_pair",
  "pair": "SOL/USDC"
}
```

## 💾 ClickHouse Schema

### Trades Table

```sql
CREATE TABLE trades (
    id String,
    timestamp DateTime,
    base_symbol String,
    quote_symbol String,
    price Float64,
    amount Float64,
    side String
) ENGINE = MergeTree()
ORDER BY (timestamp);
```

### Sessions Table

```sql
CREATE TABLE sessions (
    user_pubkey String,
    token String,
    created_at DateTime,
    expires_at DateTime
) ENGINE = MergeTree()
ORDER BY (user_pubkey, expires_at);
```

## 🔄 Data Flow

### Trade Ingestion Flow

1. **QuickNode WebSocket** subscribes to DEX program logs
2. **Backend detects** swap transactions from logs
3. **Backend fetches** full transaction details via RPC
4. **Backend parses** trade data (amount, price, side, pair)
5. **Backend stores** trade in ClickHouse
6. **Backend broadcasts** trade to connected WebSocket clients
7. **Frontend receives** trade and updates UI in real-time

### Price Update Flow

1. **Backend periodically** fetches prices from Jupiter API
2. **Backend broadcasts** price updates via WebSocket
3. **Frontend receives** price update and updates chart/statistics

### Chart Data Flow

1. **Frontend requests** OHLCV data from `/api/ohlcv`
2. **Backend queries** ClickHouse for aggregated candles
3. **Backend returns** OHLCV data to frontend
4. **Frontend renders** chart with historical data
5. **Frontend updates** chart in real-time via WebSocket trades

## 🎨 Supported Trading Pairs

- **SOL/USDC** - Solana / USD Coin
- **SOL/USDT** - Solana / Tether
- **BONK/SOL** - Bonk / Solana
- **JUP/SOL** - Jupiter / Solana
- **WIF/SOL** - dogwifhat / Solana
- **RAY/SOL** - Raydium / Solana

## 🏭 Supported DEX Programs

- **Jupiter v6** - `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
- **Jupiter v4** - `JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB`
- **Raydium** - `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`
- **Orca** - `9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP`
- **Meteora** - `9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp`
- **Phoenix** - `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLRJi5i4Z2j3Yc`

## 🚢 Deployment

### Frontend Deployment (Netlify/Vercel)

1. **Build the frontend:**
```bash
cd frontend
npm run build
```

2. **Deploy to Netlify:**
   - Connect GitHub repository
   - Set build command: `npm run build`
   - Set publish directory: `dist`
   - Add environment variables in Netlify dashboard

3. **Deploy to Vercel:**
```bash
npm install -g vercel
vercel
```

### Backend Deployment (Railway/Render/Fly.io)

1. **Build Docker image:**
```bash
cd backend
docker build -t solana-trade-backend .
```

2. **Deploy to Railway:**
   - Connect GitHub repository
   - Select backend directory
   - Add environment variables
   - Deploy

3. **Deploy to Render:**
   - Create new Web Service
   - Connect GitHub repository
   - Set build command: `cargo build --release`
   - Set start command: `./target/release/backend`
   - Add environment variables

4. **Deploy to Fly.io:**
```bash
fly launch
fly secrets set QUICKNODE_RPC_URL=...
fly secrets set CLICKHOUSE_URL=...
fly deploy
```

### ClickHouse Deployment

- **ClickHouse Cloud** (Recommended):
  1. Sign up at https://clickhouse.cloud
  2. Create new service
  3. Get connection URL and credentials
  4. Update `CLICKHOUSE_URL` in `.env` file

- **Docker** (Local):
  - Already included in `docker-compose.yml`
  - Or run standalone:
```bash
docker run -d -p 8123:8123 -p 9000:9000 clickhouse/clickhouse-server
```

### Docker Compose Notes

**Frontend Environment Variables:**
- `VITE_API_BASE_URL` and `VITE_WS_BASE_URL` are set at **build time**
- If accessing from the same machine, use `http://localhost:3000`
- If accessing from a different machine, use `http://your-machine-ip:3000`
- To rebuild with new URLs: `docker-compose build frontend && docker-compose up -d`

## 🧪 Testing

### Test WebSocket Connection

```bash
# Connect to WebSocket
wscat -c ws://localhost:3000/ws/trades

# Send pair selection
{"type": "select_pair", "pair": "SOL/USDC"}
```

### Test API Endpoints

```bash
# Get trades
curl "http://localhost:3000/api/trades?pair=SOL/USDC&limit=10"

# Get OHLCV data
curl "http://localhost:3000/api/ohlcv?pair=SOL/USDC&interval=1m"

# Get nonce
curl "http://localhost:3000/auth/nonce"
```

## 📊 Performance Considerations

- **WebSocket Reconnection**: Automatic reconnection with exponential backoff
- **ClickHouse Optimization**: Efficient time-series queries with proper indexing
- **Chart Rendering**: Optimized for large datasets with TradingView Lightweight Charts
- **Price Updates**: Throttled to prevent excessive API calls
- **Trade Filtering**: Backend filters trades by allowed tokens before processing

## 🔒 Security

- **JWT Tokens**: Secure token-based authentication
- **ed25519 Signatures**: Cryptographically secure signature verification
- **CORS**: Configured for production domains
- **Session Expiration**: Automatic session cleanup
- **Input Validation**: All API inputs are validated
- **Rate Limiting**: (To be implemented) API rate limiting

## 🐛 Troubleshooting

### Backend Issues

**QuickNode Connection Failed:**
- Check `QUICKNODE_RPC_URL` is set correctly
- Verify WebSocket URL format (wss://)
- Check QuickNode account credits

**ClickHouse Connection Failed:**
- Verify `CLICKHOUSE_URL`, `CLICKHOUSE_USERNAME`, `CLICKHOUSE_PASSWORD`
- Check ClickHouse service is running
- Verify network connectivity

**No Trades Appearing:**
- Check QuickNode WebSocket subscription is active
- Verify DEX programs are being monitored
- Check trade filtering logic

### Frontend Issues

**WebSocket Not Connecting:**
- Verify `VITE_WS_BASE_URL` is correct
- Check backend is running
- Verify CORS is configured

**Chart Not Displaying:**
- Check OHLCV endpoint is returning data
- Verify TradingView chart initialization
- Check browser console for errors

**Phantom Authentication Failing:**
- Ensure Phantom wallet is installed
- Check network is set to mainnet-beta
- Verify signature verification logic

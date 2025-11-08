# Setup Guide - Trade Frontend

Complete setup guide to get the Solana Trade Tracker frontend running.

## Prerequisites

Before you begin, ensure you have:

- ✅ **Node.js 18+** installed ([Download here](https://nodejs.org/))
- ✅ **npm or yarn** package manager
- ✅ **Phantom Wallet** browser extension ([Install here](https://phantom.app/))
- ✅ **Git** for version control
- ✅ **Code Editor** (VS Code recommended)

## Quick Start (5 minutes)

### 1. Clone and Install

```bash
# Clone the repository
git clone <your-repo-url>
cd solana-trade-stream

# Install dependencies
npm install

# This will install:
# - React & TypeScript
# - Solana Wallet Adapter
# - TradingView Lightweight Charts
# - TailwindCSS
# - And all other dependencies
```

### 2. Environment Configuration

```bash
# Create environment file
cp .env.example .env

# Edit .env with your settings
nano .env  # or use your preferred editor
```

**Required Environment Variables:**

```env
# Backend API base URL
VITE_API_BASE_URL=http://localhost:8080

# WebSocket URL
VITE_WS_BASE_URL=ws://localhost:8080

# Solana network (mainnet-beta, testnet, devnet)
VITE_SOLANA_NETWORK=mainnet-beta
```

### 3. Start Development Server

```bash
npm run dev

# Server will start at:
# http://localhost:8080

# You should see:
# ✓ ready in Xms
# ➜  Local:   http://localhost:8080/
# ➜  Network: use --host to expose
```

### 4. Open in Browser

1. Navigate to `http://localhost:8080`
2. You should see the Trade interface
3. Click "Connect Wallet" to connect Phantom

## Backend Setup (Required for Full Functionality)

The frontend requires a running backend. Follow these steps:

### Option A: Connect to Existing Backend

If you already have a backend running:

```env
VITE_API_BASE_URL=https://your-backend-url.com
VITE_WS_BASE_URL=wss://your-backend-url.com
```

### Option B: Run Backend Locally

1. **Start your Rust backend** (from backend directory):
```bash
cd ../backend
cargo run
```

2. **Ensure ClickHouse is running**:
```bash
# Using Docker
docker run -d -p 9000:9000 --name clickhouse-server clickhouse/clickhouse-server

# Or using native installation
clickhouse-server
```

3. **Verify backend is running**:
```bash
curl http://localhost:8080/health
# Should return: {"status": "ok"}
```

## Testing the Application

### 1. Test Wallet Connection

1. Ensure Phantom is installed and set up
2. Click "Connect Wallet" button
3. Approve connection in Phantom popup
4. Your wallet address should appear

### 2. Test Authentication

1. After connecting wallet, click "Authenticate"
2. Phantom will prompt you to sign a message
3. Approve the signature
4. You should see a success toast and checkmark

### 3. Test Live Data (Requires Backend)

1. Select a trading pair from dropdown (e.g., SOL/USDC)
2. Trades should appear in the table in real-time
3. Chart should display candlestick data
4. Price stats should update automatically

## Common Issues & Solutions

### Issue: "Failed to fetch nonce"

**Solution:**
- Check backend is running: `curl http://localhost:8080/auth/nonce`
- Verify `VITE_API_BASE_URL` in `.env`
- Check CORS is enabled on backend

### Issue: WebSocket not connecting

**Solution:**
- Check backend WebSocket server is running
- Verify `VITE_WS_BASE_URL` in `.env`
- Look for errors in browser console (F12)
- Ensure no firewall blocking WebSocket connections

### Issue: Phantom wallet not detected

**Solution:**
- Install Phantom extension
- Refresh the page
- Check extension is enabled
- Try in different browser (Chrome/Firefox/Brave)

### Issue: Chart not rendering

**Solution:**
- Check browser console for errors
- Ensure backend OHLCV endpoint is working
- Verify trading pair has data
- Clear browser cache and reload

### Issue: Build errors during `npm install`

**Solution:**
```bash
# Clear npm cache
npm cache clean --force

# Remove node_modules and package-lock
rm -rf node_modules package-lock.json

# Reinstall
npm install
```

## Development Workflow

### Hot Reload

The dev server supports hot module replacement (HMR):
- Edit any component file
- Changes appear instantly
- No need to refresh browser
- State is preserved when possible

### Code Structure

```
src/
├── components/       # React components
├── contexts/        # React contexts
├── hooks/           # Custom React hooks
├── lib/             # Utility libraries
│   ├── api.ts      # REST API client
│   ├── websocket.ts # WebSocket client
│   └── utils.ts    # Helper functions
└── pages/          # Page components
```

### Making Changes

1. **Add a new component:**
```bash
# Create file
touch src/components/MyComponent.tsx

# Import in parent
import { MyComponent } from '@/components/MyComponent';
```

2. **Add a new API endpoint:**
```typescript
// In src/lib/api.ts
export const myApi = {
  async getData(): Promise<Data[]> {
    const response = await fetch(`${API_BASE_URL}/api/data`);
    return response.json();
  }
};
```

3. **Add a new hook:**
```bash
# Create file
touch src/hooks/useMyHook.ts

# Use in component
import { useMyHook } from '@/hooks/useMyHook';
```

## Testing

### Manual Testing Checklist

- [ ] Wallet connection works
- [ ] Authentication flow completes
- [ ] Trades appear in table
- [ ] Chart renders correctly
- [ ] Pair selector works
- [ ] Price stats update
- [ ] WebSocket reconnects after disconnect
- [ ] Token persists after refresh
- [ ] Logout clears token

### Browser Testing

Test in multiple browsers:
- [ ] Chrome/Chromium
- [ ] Firefox
- [ ] Safari (Mac only)
- [ ] Brave
- [ ] Edge

### Responsive Testing

Test at different screen sizes:
- [ ] Desktop (1920x1080)
- [ ] Laptop (1366x768)
- [ ] Tablet (768x1024)
- [ ] Mobile (375x667)

## Building for Production

### 1. Build the Application

```bash
# Create production build
npm run build

# Output will be in dist/
# dist/
# ├── assets/
# │   ├── index-[hash].js
# │   └── index-[hash].css
# └── index.html
```

### 2. Test Production Build Locally

```bash
# Preview production build
npm run preview

# Opens at http://localhost:4173
```

### 3. Deploy

#### Option A: Vercel

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel

# Follow prompts
# Set environment variables in Vercel dashboard
```

#### Option B: Netlify

```bash
# Install Netlify CLI
npm i -g netlify-cli

# Deploy
netlify deploy --prod --dir=dist

# Or use Netlify UI to connect Git repo
```

#### Option C: AWS S3 + CloudFront

```bash
# Build
npm run build

# Upload to S3
aws s3 sync dist/ s3://your-bucket-name

# Configure CloudFront distribution
# Set environment variables in S3/CloudFront
```

## Environment Variables for Production

Create environment variables in your hosting platform:

**Vercel:**
- Project Settings → Environment Variables

**Netlify:**
- Site Settings → Build & Deploy → Environment

**AWS:**
- CloudFront → Distributions → Behaviors → Environment Variables

**Variables to set:**
```
VITE_API_BASE_URL=https://api.yourproduction.com
VITE_WS_BASE_URL=wss://api.yourproduction.com
VITE_SOLANA_NETWORK=mainnet-beta
```

## Monitoring & Debugging

### Browser Console

Open DevTools (F12) to see:
- Component render cycles
- API requests and responses
- WebSocket messages
- Error stack traces
- Performance metrics

### Network Tab

Monitor network activity:
- REST API calls
- WebSocket connection
- Response times
- Payload sizes

### React DevTools

Install React DevTools extension to:
- Inspect component tree
- View props and state
- Profile performance
- Track re-renders

## IDE Setup (VS Code)

### Recommended Extensions

```json
{
  "recommendations": [
    "dbaeumer.vscode-eslint",
    "esbenp.prettier-vscode",
    "bradlc.vscode-tailwindcss",
    "ms-vscode.vscode-typescript-next"
  ]
}
```

### Settings

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "typescript.tsdk": "node_modules/typescript/lib",
  "tailwindCSS.experimental.classRegex": [
    ["cn\\(([^)]*)\\)", "\"([^\"]*)\""]
  ]
}
```

## Performance Tips

1. **Keep trades list to 100 items max** (already implemented)
2. **Use React DevTools Profiler** to identify slow renders
3. **Lazy load heavy components** for faster initial load
4. **Optimize images** if you add any custom graphics
5. **Use production build** for testing real performance

## Security Checklist

- [ ] Use HTTPS in production
- [ ] Set proper CORS headers on backend
- [ ] Never commit `.env` file
- [ ] Rotate JWT secrets regularly
- [ ] Use CSP headers
- [ ] Keep dependencies updated
- [ ] Audit dependencies: `npm audit`

## Getting Help

### Documentation
- [React Docs](https://react.dev/)
- [Solana Wallet Adapter](https://github.com/solana-labs/wallet-adapter)
- [TradingView Lightweight Charts](https://tradingview.github.io/lightweight-charts/)
- [TailwindCSS](https://tailwindcss.com/docs)

### Debug Commands

```bash
# Check Node version
node --version

# Check npm version
npm --version

# List installed packages
npm list

# Check for outdated packages
npm outdated

# Audit dependencies
npm audit

# Fix audit issues
npm audit fix
```

## Next Steps

After successful setup:

1. ✅ Verify all features work
2. ✅ Test with real backend
3. ✅ Customize branding/colors if needed
4. ✅ Add any custom features
5. ✅ Write tests
6. ✅ Deploy to production

## Support

For issues related to:
- **Frontend**: Check this README and ARCHITECTURE.md
- **Backend**: Refer to backend documentation
- **Solana/Phantom**: Check official Solana docs
- **Deployment**: Refer to hosting platform docs

---

**Happy Trading! 🚀**

# SniperBot 2.0 Frontend Integration Guide

## Overview

The SniperBot 2.0 frontend has been successfully integrated with the existing bot system, providing a modern React/TypeScript interface for monitoring and controlling the trading bot.

## Architecture

### Frontend Stack
- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite 5.4
- **Styling**: Tailwind CSS 3.4
- **Icons**: Lucide React
- **State Management**: React Context + useReducer

### Backend Integration
- **API Server**: Axum-based REST API on port 8084
- **WebSocket**: Real-time updates via WebSocket connection
- **Static Files**: Frontend served directly from API server in production

## Key Features

### 🎯 **Real-time Dashboard**
- **Bot Status**: Live monitoring of bot state, portfolio value, and strategy performance
- **Signal Feed**: Real-time trading signals with strength indicators and filtering
- **Trade History**: Complete trade execution history with status tracking
- **WebSocket Integration**: Live updates without page refresh

### 🔧 **Bot Control**
- **Start/Stop Bot**: Direct control over bot execution
- **Strategy Management**: Enable/disable individual strategies
- **Emergency Controls**: Quick access to emergency stop functionality

### 📊 **Analytics & Monitoring**
- **Performance Metrics**: Strategy-specific P&L, win rates, and signal counts
- **Portfolio Tracking**: Real-time portfolio value and balance updates
- **Connection Status**: Visual indicators for API and WebSocket connectivity

## File Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── dashboard/
│   │   │   ├── BotStatusDashboard.tsx    # Main bot status and controls
│   │   │   ├── SignalsDashboard.tsx      # Real-time signals feed
│   │   │   └── TradesDashboard.tsx       # Trade history and analytics
│   │   └── layout/
│   │       ├── DashboardLayout.tsx       # Main layout component
│   │       ├── Header.tsx                # Top navigation
│   │       └── Sidebar.tsx               # Side navigation
│   ├── context/
│   │   ├── SniperBotContext.tsx          # Bot state management
│   │   └── ThemeContext.tsx              # Theme management
│   ├── utils/
│   │   └── api.ts                        # API client and types
│   ├── App.tsx                           # Root component
│   └── main.tsx                          # Entry point
├── .env                                  # Environment configuration
├── vite.config.ts                        # Vite configuration with proxy
└── package.json                          # Dependencies and scripts
```

## API Integration

### REST Endpoints
- `GET /api/bot-status` - Bot status and strategy performance
- `GET /api/signals` - Recent trading signals
- `GET /api/trades` - Trade history
- `POST /api/bot/start` - Start the bot
- `POST /api/bot/stop` - Stop the bot
- `POST /api/strategy/{name}/toggle` - Toggle strategy

### WebSocket Events
- `SignalGenerated` - New trading signal
- `TradeExecuted` - Trade completion
- `connected` - Connection established

### Data Types
```typescript
interface BotStatus {
  success: boolean;
  data: {
    engine_status: {
      is_running: boolean;
      portfolio_value: number;
    };
    active_strategies: string[];
    strategy_performance: Record<string, StrategyPerformance>;
  };
}

interface Signal {
  id: string;
  strategy: string;
  signal_type: 'Buy' | 'Sell';
  symbol: string;
  strength: number;
  timestamp: string;
  price?: number;
  reason?: string;
}
```

## Development Workflow

### Development Mode
```bash
# Start both API server and frontend dev server
./scripts/start_dev.sh

# Access points:
# Frontend: http://localhost:3000 (or 3001 if 3000 is busy)
# API: http://localhost:8084
# WebSocket: ws://localhost:8084/ws
```

### Production Mode
```bash
# Build frontend and start integrated server
./scripts/start_with_frontend.sh

# Access point:
# Complete app: http://localhost:8084
```

## Configuration

### Environment Variables
```bash
# Frontend (.env)
VITE_API_BASE_URL=http://localhost:8084
VITE_WS_BASE_URL=ws://localhost:8084
VITE_DEV_MODE=true
VITE_LOG_LEVEL=debug
```

### Vite Proxy Configuration
```typescript
// vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8084',
      changeOrigin: true,
    },
    '/ws': {
      target: 'ws://localhost:8084',
      ws: true,
    },
  },
}
```

## Integration Benefits

### ✅ **Improved User Experience**
- Modern, responsive interface
- Real-time updates without manual refresh
- Intuitive navigation and controls
- Visual feedback for all actions

### ✅ **Enhanced Monitoring**
- Live strategy performance tracking
- Real-time signal visualization
- Comprehensive trade history
- Connection status indicators

### ✅ **Better Control**
- Direct bot management from UI
- Individual strategy control
- Emergency stop functionality
- Configuration management

### ✅ **Developer Experience**
- TypeScript for type safety
- Hot reload in development
- Modular component architecture
- Easy to extend and maintain

## Troubleshooting

### Common Issues

1. **Port Conflicts**
   ```bash
   # Kill processes on ports 8084 and 3000
   lsof -ti:8084 | xargs kill -9
   lsof -ti:3000 | xargs kill -9
   ```

2. **API Connection Issues**
   - Check if API server is running on port 8084
   - Verify CORS settings in API server
   - Check network connectivity

3. **WebSocket Connection Issues**
   - Ensure WebSocket endpoint is accessible
   - Check browser console for connection errors
   - Verify WebSocket handler in API server

4. **Build Issues**
   ```bash
   # Clean and rebuild
   cd frontend
   rm -rf node_modules dist
   npm install
   npm run build
   ```

## Future Enhancements

- [ ] Advanced charting and analytics
- [ ] Strategy configuration UI
- [ ] Risk management controls
- [ ] Performance optimization
- [ ] Mobile responsiveness
- [ ] Dark/light theme toggle
- [ ] Export functionality
- [ ] Alert notifications

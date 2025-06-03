# 🌐 SniperBot 2.0 - API Documentation

This document provides comprehensive API documentation for SniperBot 2.0's REST API and WebSocket endpoints.

## 📋 Table of Contents

- [Authentication](#authentication)
- [REST API Endpoints](#rest-api-endpoints)
- [WebSocket API](#websocket-api)
- [Data Models](#data-models)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)

## 🔐 Authentication

### API Key Authentication

Include your API key in the request headers:

```bash
curl -H "X-API-Key: your-api-key" \
     -H "Content-Type: application/json" \
     http://localhost:8080/api/v1/status
```

### Configuration

Enable authentication in `config.toml`:

```toml
[api]
auth_enabled = true
api_key = "your-secure-api-key"
```

## 🌐 REST API Endpoints

### Base URL
```
http://localhost:8080/api/v1
```

### Health & Status

#### GET /health
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "2.0.0"
}
```

#### GET /status
Get bot status and statistics.

**Response:**
```json
{
  "bot_status": "running",
  "uptime_seconds": 3600,
  "strategies_active": 2,
  "data_sources_connected": 6,
  "orders_today": 15,
  "pnl_today": 125.50,
  "last_signal": "2024-01-15T10:25:00Z"
}
```

### Portfolio Management

#### GET /portfolio
Get current portfolio state.

**Response:**
```json
{
  "total_value": 10000.0,
  "available_balance": 5000.0,
  "unrealized_pnl": 250.0,
  "realized_pnl": 150.0,
  "daily_pnl": 75.0,
  "max_drawdown": 0.05,
  "positions": [
    {
      "symbol": "TOKEN/SOL",
      "size": 1000.0,
      "entry_price": 0.001,
      "current_price": 0.0012,
      "pnl": 200.0,
      "pnl_percentage": 20.0
    }
  ],
  "updated_at": "2024-01-15T10:30:00Z"
}
```

### Order Management

#### GET /orders
Get order history with optional filters.

**Query Parameters:**
- `limit` (optional): Number of orders to return (default: 50)
- `status` (optional): Filter by order status
- `symbol` (optional): Filter by trading pair

**Response:**
```json
{
  "orders": [
    {
      "id": "order_123",
      "symbol": "TOKEN/SOL",
      "side": "buy",
      "size": 1000.0,
      "price": 0.001,
      "status": "filled",
      "strategy": "pumpfun_sniping",
      "created_at": "2024-01-15T10:20:00Z",
      "filled_at": "2024-01-15T10:20:05Z"
    }
  ],
  "total": 1,
  "page": 1
}
```

#### POST /orders
Create a new order (manual trading).

**Request Body:**
```json
{
  "symbol": "TOKEN/SOL",
  "side": "buy",
  "size": 1000.0,
  "order_type": "market",
  "price": null,
  "stop_loss": 0.0009,
  "take_profit": 0.0015
}
```

**Response:**
```json
{
  "order_id": "order_124",
  "status": "pending",
  "message": "Order created successfully"
}
```

### Strategy Management

#### GET /strategies
Get all available strategies and their status.

**Response:**
```json
{
  "strategies": [
    {
      "name": "pumpfun_sniping",
      "type": "Sniping",
      "enabled": true,
      "confidence": 0.85,
      "signals_generated": 25,
      "successful_trades": 20,
      "win_rate": 0.8,
      "total_pnl": 500.0
    },
    {
      "name": "liquidity_sniping",
      "type": "Liquidity",
      "enabled": true,
      "confidence": 0.78,
      "signals_generated": 15,
      "successful_trades": 12,
      "win_rate": 0.8,
      "total_pnl": 300.0
    }
  ]
}
```

#### POST /strategies/{strategy_name}/parameters
Update strategy parameters dynamically.

**Request Body:**
```json
{
  "confidence_threshold": 0.8,
  "max_position_size": 750.0,
  "min_market_cap": 15000.0
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Strategy parameters updated",
  "updated_parameters": {
    "confidence_threshold": 0.8,
    "max_position_size": 750.0,
    "min_market_cap": 15000.0
  }
}
```

#### POST /strategies/{strategy_name}/enable
Enable a strategy.

#### POST /strategies/{strategy_name}/disable
Disable a strategy.

### Market Data

#### GET /market-data/{symbol}
Get current market data for a symbol.

**Response:**
```json
{
  "symbol": "TOKEN/SOL",
  "price": 0.0012,
  "volume": 50000.0,
  "bid": 0.00119,
  "ask": 0.00121,
  "sources_count": 3,
  "confidence_score": 0.9,
  "latency_ms": 150,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### GET /market-data/{symbol}/history
Get historical market data.

**Query Parameters:**
- `interval`: Time interval (1m, 5m, 15m, 1h, 4h, 1d)
- `limit`: Number of data points (default: 100)

### Data Sources

#### GET /data-sources
Get status of all data sources.

**Response:**
```json
{
  "sources": [
    {
      "name": "raydium",
      "status": "connected",
      "latency_ms": 120,
      "last_update": "2024-01-15T10:30:00Z"
    },
    {
      "name": "pumpfun",
      "status": "connected",
      "latency_ms": 200,
      "last_update": "2024-01-15T10:29:55Z"
    }
  ]
}
```

### Risk Management

#### GET /risk/status
Get current risk management status.

**Response:**
```json
{
  "global_exposure": 7500.0,
  "max_exposure": 10000.0,
  "daily_loss": 150.0,
  "max_daily_loss": 1000.0,
  "current_drawdown": 0.03,
  "max_drawdown": 0.2,
  "emergency_stop_active": false,
  "circuit_breaker_triggered": false
}
```

#### POST /risk/emergency-stop
Trigger emergency stop (close all positions).

#### POST /risk/reset-daily-limits
Reset daily loss limits (admin only).

## 🔌 WebSocket API

### Connection
```
ws://localhost:8080/ws
```

### Authentication
Send API key in connection headers or as first message:

```json
{
  "type": "auth",
  "api_key": "your-api-key"
}
```

### Subscription Topics

#### Market Data Updates
```json
{
  "type": "subscribe",
  "topic": "market_data",
  "symbols": ["TOKEN/SOL", "MEME/SOL"]
}
```

#### Strategy Signals
```json
{
  "type": "subscribe",
  "topic": "signals"
}
```

#### Order Updates
```json
{
  "type": "subscribe",
  "topic": "orders"
}
```

#### Portfolio Updates
```json
{
  "type": "subscribe",
  "topic": "portfolio"
}
```

### Message Format

#### Market Data Update
```json
{
  "type": "market_data",
  "symbol": "TOKEN/SOL",
  "price": 0.0012,
  "volume": 50000.0,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Strategy Signal
```json
{
  "type": "signal",
  "strategy": "pumpfun_sniping",
  "symbol": "TOKEN/SOL",
  "signal_type": "buy",
  "strength": 0.85,
  "price": 0.001,
  "size": 1000.0,
  "metadata": {
    "market_cap": 100000.0,
    "age_hours": 6.0,
    "confidence": 0.85
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Order Update
```json
{
  "type": "order_update",
  "order_id": "order_123",
  "status": "filled",
  "filled_size": 1000.0,
  "filled_price": 0.001,
  "timestamp": "2024-01-15T10:30:05Z"
}
```

## 📊 Data Models

### StrategySignal
```json
{
  "strategy": "string",
  "symbol": "string",
  "signal_type": "buy" | "sell",
  "strength": "number (0-1)",
  "price": "number",
  "size": "number",
  "metadata": "object",
  "timestamp": "ISO 8601 string"
}
```

### MarketData
```json
{
  "symbol": "string",
  "price": "number",
  "volume": "number",
  "bid": "number | null",
  "ask": "number | null",
  "timestamp": "ISO 8601 string",
  "source": "string"
}
```

### Order
```json
{
  "id": "string",
  "symbol": "string",
  "side": "buy" | "sell",
  "size": "number",
  "price": "number | null",
  "order_type": "market" | "limit",
  "status": "pending" | "filled" | "cancelled" | "failed",
  "strategy": "string | null",
  "created_at": "ISO 8601 string",
  "filled_at": "ISO 8601 string | null"
}
```

## ❌ Error Handling

### Error Response Format
```json
{
  "error": {
    "code": "INVALID_SYMBOL",
    "message": "Symbol TOKEN/INVALID not found",
    "details": {
      "symbol": "TOKEN/INVALID",
      "available_symbols": ["TOKEN/SOL", "MEME/SOL"]
    }
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| `UNAUTHORIZED` | Invalid or missing API key |
| `INVALID_SYMBOL` | Trading pair not supported |
| `INSUFFICIENT_BALANCE` | Not enough funds for order |
| `STRATEGY_NOT_FOUND` | Strategy name not recognized |
| `RATE_LIMIT_EXCEEDED` | Too many requests |
| `EMERGENCY_STOP_ACTIVE` | Trading halted by emergency stop |

## 🚦 Rate Limiting

### Limits
- **REST API**: 100 requests per minute per API key
- **WebSocket**: 10 subscriptions per connection
- **Order Creation**: 20 orders per minute

### Headers
Rate limit information is included in response headers:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642248600
```

### Rate Limit Exceeded Response
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again in 60 seconds.",
    "retry_after": 60
  }
}
```

---

**For more information, see the [Strategy Guide](strategies.md) and [Configuration Guide](configuration.md).**

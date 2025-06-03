# ⚙️ SniperBot 2.0 - Configuration Guide

This document provides comprehensive configuration options for SniperBot 2.0.

## 📋 Table of Contents

- [Configuration File Structure](#configuration-file-structure)
- [Bot Settings](#bot-settings)
- [Exchange Configuration](#exchange-configuration)
- [Strategy Configuration](#strategy-configuration)
- [Risk Management](#risk-management)
- [Database Configuration](#database-configuration)
- [Logging Configuration](#logging-configuration)
- [API Configuration](#api-configuration)
- [Environment Variables](#environment-variables)

## 📁 Configuration File Structure

SniperBot 2.0 uses TOML format for configuration. The main configuration file is `config.toml`:

```toml
[bot]
# Core bot settings

[exchanges]
# Exchange API configurations

[strategies]
# Trading strategy configurations

[risk_management]
# Risk control parameters

[database]
# Database connection settings

[logging]
# Logging configuration

[api]
# REST API settings
```

## 🤖 Bot Settings

### Core Configuration

```toml
[bot]
name = "SniperBot-2.0"
version = "2.0.0"
environment = "production"  # "development", "testing", "production"
update_interval_ms = 100    # Data update frequency
max_concurrent_orders = 5   # Maximum simultaneous orders
dry_run = false            # Enable dry run mode
paper_trading = false      # Enable paper trading mode
```

### Performance Settings

```toml
[bot.performance]
worker_threads = 4         # Number of async worker threads
max_connections = 100      # Maximum HTTP connections
connection_timeout_ms = 5000
request_timeout_ms = 10000
retry_attempts = 3
retry_delay_ms = 1000
```

## 🏦 Exchange Configuration

### Helius Solana RPC

```toml
[exchanges.helius]
name = "Helius Solana RPC"
api_key = "your-helius-api-key"
enabled = true
sandbox = false
rate_limit_per_second = 10
supported_pairs = ["SOL/USDC", "TOKEN/SOL"]

[exchanges.helius.endpoints]
rpc_url = "https://mainnet.helius-rpc.com"
enhanced_api_url = "https://api.helius.xyz/v0"
websocket_url = "wss://mainnet.helius-rpc.com"
```

### Binance

```toml
[exchanges.binance]
name = "Binance"
api_key = "your-binance-api-key"
api_secret = "your-binance-secret"
enabled = true
sandbox = false
rate_limit_per_second = 10
supported_pairs = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]

[exchanges.binance.endpoints]
rest_url = "https://api.binance.com"
websocket_url = "wss://stream.binance.com:9443"
```

### Exchange Template

```toml
[exchanges.custom_exchange]
name = "Custom Exchange"
api_key = "your-api-key"
api_secret = "your-api-secret"
enabled = false
sandbox = true
rate_limit_per_second = 5
supported_pairs = []

[exchanges.custom_exchange.endpoints]
rest_url = "https://api.example.com"
websocket_url = "wss://ws.example.com"
```

## 🎯 Strategy Configuration

### PumpFun Sniping Strategy

```toml
[strategies.pumpfun_sniping]
enabled = true
confidence_threshold = 0.75
max_position_size = 500.0
stop_loss_percentage = 15.0
take_profit_percentage = 50.0
cooldown_seconds = 300
required_sources = ["pumpfun"]

# Strategy-specific parameters
min_market_cap = 10000.0
max_market_cap = 1000000.0
min_volume_24h = 5000.0
max_age_hours = 24.0
min_holder_count = 10
graduation_threshold = 0.8
bonding_curve_progress_min = 0.1
bonding_curve_progress_max = 0.9
max_slippage_bps = 500
min_liquidity = 1000.0
blacklisted_creators = []
```

### Liquidity Pool Sniping Strategy

```toml
[strategies.liquidity_sniping]
enabled = true
confidence_threshold = 0.8
max_position_size = 1000.0
stop_loss_percentage = 10.0
take_profit_percentage = 25.0
cooldown_seconds = 180
required_sources = ["raydium", "meteora"]

# Strategy-specific parameters
min_initial_liquidity = 5000.0
max_initial_liquidity = 100000.0
min_pool_age_minutes = 5.0
max_pool_age_hours = 12.0
min_apr = 50.0
max_price_impact = 3.0
min_token_holders = 20
max_token_supply = 1000000000.0
max_slippage_bps = 300
min_volume_ratio = 0.1
preferred_quote_tokens = [
    "So11111111111111111111111111111111111111112",  # SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"   # USDC
]
```

### Strategy Template

```toml
[strategies.custom_strategy]
enabled = false
confidence_threshold = 0.7
max_position_size = 1000.0
stop_loss_percentage = 5.0
take_profit_percentage = 15.0
cooldown_seconds = 60
required_sources = ["binance"]

# Add custom parameters here
custom_parameter = "value"
```

## 🛡️ Risk Management

### Global Risk Settings

```toml
[risk_management]
global_max_exposure = 10000.0      # Maximum total position value
max_daily_loss = 1000.0            # Maximum daily loss limit
max_drawdown = 0.2                 # Maximum portfolio drawdown (20%)
position_sizing_method = "percentage" # "fixed", "percentage", "volatility"
emergency_stop_enabled = true      # Enable emergency stop functionality
circuit_breaker_threshold = 0.05   # 5% loss triggers circuit breaker
```

### Position Sizing

```toml
[risk_management.position_sizing]
default_percentage = 2.0           # 2% of portfolio per trade
max_percentage = 5.0               # Maximum 5% per trade
min_position_size = 10.0           # Minimum position size in USD
max_position_size = 2000.0         # Maximum position size in USD
volatility_adjustment = true       # Adjust size based on volatility
```

### Stop Loss & Take Profit

```toml
[risk_management.stops]
default_stop_loss = 5.0            # Default 5% stop loss
default_take_profit = 15.0         # Default 15% take profit
trailing_stop_enabled = true       # Enable trailing stops
trailing_stop_distance = 3.0       # 3% trailing distance
max_stop_loss = 20.0               # Maximum allowed stop loss
min_take_profit = 5.0              # Minimum take profit
```

### Circuit Breakers

```toml
[risk_management.circuit_breakers]
daily_loss_limit = 1000.0          # Daily loss limit
portfolio_drawdown_limit = 0.15    # 15% portfolio drawdown limit
consecutive_losses_limit = 5       # Stop after 5 consecutive losses
volatility_threshold = 0.5         # Stop if volatility > 50%
```

## 🗄️ Database Configuration

### SQLite (Default)

```toml
[database]
sqlite_path = "data/sniperbot.db"
connection_pool_size = 10
max_connections = 20
connection_timeout_seconds = 30
```

### Redis (Optional)

```toml
[database]
redis_url = "redis://localhost:6379"
redis_password = "your-redis-password"
redis_db = 0
redis_pool_size = 10
```

### QuestDB (Time Series)

```toml
[database]
questdb_url = "http://localhost:9000"
questdb_username = "admin"
questdb_password = "quest"
```

### Neo4j (Graph Database)

```toml
[database]
neo4j_url = "bolt://localhost:7687"
neo4j_username = "neo4j"
neo4j_password = "your-neo4j-password"
```

## 📝 Logging Configuration

### Basic Logging

```toml
[logging]
level = "info"                     # "trace", "debug", "info", "warn", "error"
file_path = "logs/sniperbot.log"
max_file_size_mb = 100
max_files = 10
structured = true                  # Enable structured JSON logging
```

### Advanced Logging

```toml
[logging.modules]
"sniperbot::data_fetcher" = "debug"
"sniperbot::strategy" = "info"
"sniperbot::risk_management" = "warn"
"sniperbot::execution" = "debug"

[logging.filters]
exclude_patterns = ["heartbeat", "ping"]
include_only = []                  # Empty = include all

[logging.outputs]
console = true
file = true
syslog = false
```

## 🌐 API Configuration

### REST API

```toml
[api]
host = "127.0.0.1"
port = 8080
cors_enabled = true
auth_enabled = true
api_key = "your-secure-api-key"
rate_limit_per_minute = 100
max_request_size_mb = 10
```

### WebSocket

```toml
[api.websocket]
enabled = true
max_connections = 100
heartbeat_interval_seconds = 30
max_message_size_kb = 1024
compression_enabled = true
```

### TLS/SSL

```toml
[api.tls]
enabled = false
cert_file = "certs/server.crt"
key_file = "certs/server.key"
ca_file = "certs/ca.crt"
```

## 🌍 Environment Variables

### Required Variables

```bash
# Helius API Key
HELIUS_API_KEY=your-helius-api-key

# Binance API Credentials
BINANCE_API_KEY=your-binance-api-key
BINANCE_API_SECRET=your-binance-secret

# Database URLs
DATABASE_URL=sqlite:data/sniperbot.db
REDIS_URL=redis://localhost:6379

# API Security
API_KEY=your-secure-api-key
JWT_SECRET=your-jwt-secret
```

### Optional Variables

```bash
# Environment
ENVIRONMENT=production
LOG_LEVEL=info
DRY_RUN=false
PAPER_TRADING=false

# Performance
WORKER_THREADS=4
MAX_CONNECTIONS=100

# Monitoring
PROMETHEUS_ENABLED=true
GRAFANA_ENABLED=true
```

### Environment File (.env)

Create a `.env` file in the project root:

```bash
# Copy from template
cp .env.template .env

# Edit with your values
nano .env
```

## 🔧 Configuration Validation

### Validation Rules

The bot validates configuration on startup:

- **Required fields**: Must be present and non-empty
- **Numeric ranges**: Values must be within acceptable ranges
- **API keys**: Must be valid format (length, characters)
- **URLs**: Must be valid and reachable
- **File paths**: Must exist and be accessible

### Configuration Test

Test your configuration:

```bash
# Validate configuration
cargo run -- --validate-config

# Test with dry run
cargo run -- --dry-run --config config.toml
```

## 📊 Configuration Examples

### Development Environment

```toml
[bot]
environment = "development"
dry_run = true
update_interval_ms = 1000

[logging]
level = "debug"
console = true

[api]
auth_enabled = false
```

### Production Environment

```toml
[bot]
environment = "production"
dry_run = false
update_interval_ms = 100

[logging]
level = "info"
structured = true

[api]
auth_enabled = true
tls_enabled = true
```

### High-Frequency Trading

```toml
[bot]
update_interval_ms = 50
max_concurrent_orders = 10

[bot.performance]
worker_threads = 8
max_connections = 200

[strategies.pumpfun_sniping]
cooldown_seconds = 60
```

## 🔄 Dynamic Configuration

### Runtime Updates

Some configuration can be updated at runtime via API:

```bash
# Update strategy parameters
curl -X POST http://localhost:8080/api/v1/config/strategies/pumpfun_sniping \
  -H "Content-Type: application/json" \
  -d '{"confidence_threshold": 0.8}'

# Update risk management
curl -X POST http://localhost:8080/api/v1/config/risk_management \
  -H "Content-Type: application/json" \
  -d '{"max_daily_loss": 1500.0}'
```

### Configuration Reload

Reload configuration without restart:

```bash
# Send SIGHUP signal
kill -HUP $(pgrep sniperbot)

# Or via API
curl -X POST http://localhost:8080/api/v1/config/reload
```

---

**For more information, see the [API Documentation](api.md) and [Strategy Guide](strategies.md).**

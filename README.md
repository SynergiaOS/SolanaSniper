# 🚀 SniperBot 2.0 - Revolutionary AI Trading Bot

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![Graphiti](https://img.shields.io/badge/Graphiti-Knowledge%20Graph-purple.svg)](https://github.com/getzep/graphiti)
[![DragonflyDB](https://img.shields.io/badge/DragonflyDB-25x%20Faster-red.svg)](https://dragonflydb.io)

**The most intelligent Solana trading bot powered by Graphiti Knowledge Graph and DragonflyDB ultra-fast cache.**

## 🧠 Revolutionary Architecture

SniperBot 2.0 features a cutting-edge technology stack that sets it apart from all other trading bots:

### 🎯 **Core Technologies:**
- **🧠 Hub-and-Spoke Architecture** - Persistent state management with DragonflyDB brain
- **🐉 DragonflyDB** - Ultra-fast cache (25x faster than Redis, 80% cost savings)
- **🤖 AI Decision Engine** - Mistral AI integration for intelligent trading decisions
- **🐍 Python-Rust Bridge** - Seamless integration with PyInstaller executables
- **⚡ Real-time Data Pipeline** - Soul Meteor Scanner + Crawl4AI validation
- **🦀 Rust Core** - High-performance, memory-safe trading engine

### 🏆 **Competitive Advantages:**
1. **Hub-and-Spoke Architecture** - Persistent state management with enterprise scalability
2. **Ultra-Performance Caching** - DragonflyDB with sub-millisecond data access
3. **AI-Powered Decisions** - Mistral AI integration with real-time analysis
4. **Python-Rust Bridge** - Seamless integration between high-level AI and low-level execution
5. **Real-time Data Pipeline** - Soul Meteor Scanner + Crawl4AI validation pipeline
6. **Persistent Intelligence** - Bot remembers everything between restarts

## 🚀 Features

### 🎯 **Specialized Sniping Strategies**
- **🔥 PumpFun Sniping**: Early meme token detection with bonding curve analysis
- **💧 Liquidity Pool Sniping**: New Raydium/Meteora pool detection and entry
- **🎓 Graduation Tracking**: Automatic detection of tokens moving from Pump.fun to Raydium
- **📊 Confidence Scoring**: AI-driven signal strength assessment (75-80% thresholds)
- **⚡ Ultra-Fast Execution**: <200ms data latency, <100ms signal generation

### 🌐 **Multi-Source Data Aggregation**
- **6+ Data Sources**: Raydium, Pump.fun, Meteora DLMM, Jupiter, Binance, Helius
- **🔄 Real-Time Feeds**: WebSocket connections with automatic reconnection
- **🧠 Smart Prioritization**: CEX > Established DEX > Meme platforms
- **📈 Data Confidence**: Multi-source validation with confidence scoring
- **⚡ Caching System**: 5-second intelligent caching for optimization

### 🎪 **Pump.fun Specialization**
- **🚀 Early Detection**: Tokens < 24 hours old, $10k-$1M market cap
- **📊 Bonding Curve Analysis**: 10-90% progress monitoring
- **👥 Creator Analysis**: Blacklist support and reputation tracking
- **🎯 Sweet Spot Targeting**: $50k-$500k optimal range detection
- **🔄 Volume Surge Detection**: Real-time volume momentum analysis

### 💧 **DEX Pool Intelligence**
- **🆕 New Pool Detection**: 5min-12hr age window for optimal entry
- **💰 APR Estimation**: >50% minimum with dynamic calculation
- **📊 Volume/Liquidity Ratio**: >10% minimum for quality assessment
- **🎯 Preferred Pairs**: SOL/USDC prioritization
- **⚖️ Price Impact Control**: <3% maximum impact protection

### 🛡️ **Advanced Risk Management**
- **📏 Dynamic Position Sizing**: Market cap and liquidity-based
- **🛑 Multi-Level Stops**: 15% stop-loss, 50% take-profit for memes
- **⏰ Cooldown Periods**: 3-5 minute intervals between signals
- **🚨 Circuit Breakers**: Emergency stop and exposure limits
- **📊 Portfolio Monitoring**: Real-time P&L and drawdown tracking

## 🎉 **Phase 5 Achievements - The Persistent Brain**

### ✅ **Hub-and-Spoke Architecture - COMPLETED!**
- **🧠 DragonflyDB Brain** - Persistent state management with connection pooling
- **🐍 Python-Rust Bridge** - Seamless data translation with compatibility layer
- **📊 Live Production Data** - 40+ hot opportunities with real-time market intelligence
- **🔄 Continuous Operation** - Bot remembers everything between restarts
- **⚡ Sub-millisecond Access** - Ultra-fast data retrieval and caching

### 🚀 **Production-Ready Components:**
- **Soul Meteor Scanner** → DragonflyDB (Producer)
- **Pipeline Controller** → DragonflyDB (Processor)
- **Trading Executor** → DragonflyDB (Consumer)
- **Position Manager** → DragonflyDB (Monitor)

## 🏗️ Architecture

```
                    ┌─────────────────────────────────┐
                    │        DragonflyDB Brain        │
                    │     (Persistent State Hub)      │
                    └─────────────┬───────────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
┌───────▼────────┐    ┌───────────▼──────────┐    ┌────────▼────────┐
│ Soul Meteor    │    │   Pipeline           │    │ Trading         │
│ Scanner        │    │   Controller         │    │ Executor        │
│ (Python)       │    │   (Rust)             │    │ (Rust)          │
└────────────────┘    └──────────────────────┘    └─────────────────┘
        │                         │                         │
        ▼                         ▼                         ▼
┌────────────────┐    ┌──────────────────────┐    ┌─────────────────┐
│ Crawl4AI       │    │ AI Decision Engine   │    │ Position        │
│ Service        │    │ (Mistral AI)         │    │ Manager         │
│ (Python)       │    │ (Rust)               │    │ (Rust)          │
└────────────────┘    └──────────────────────┘    └─────────────────┘
```

## 🛠️ Revolutionary Stack Setup

### Prerequisites
- **Rust 1.75+** - High-performance core
- **Python 3.10+** - Graphiti Knowledge Graph
- **Docker & Docker Compose** - Infrastructure
- **Git** - Version control

### 🚀 One-Command Setup

```bash
# Clone and setup the revolutionary stack
git clone https://github.com/SynergiaOS/SniperBot.git
cd SniperBot
chmod +x setup_dev_environment.sh
./setup_dev_environment.sh
```

This script will:
- ✅ Setup **Graphiti Knowledge Graph** (Neo4j)
- ✅ Setup **DragonflyDB** ultra-fast cache
- ✅ Install Python dependencies
- ✅ Build Rust project
- ✅ Start all services
- ✅ Test connections

### 🔧 Manual Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/SynergiaOS/SniperBot.git
   cd SniperBot
   ```

2. **Start infrastructure services**
   ```bash
   docker-compose -f docker-compose.dev.yml up -d
   ```

3. **Setup Python environment for Graphiti**
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   ```

4. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys
   ```

5. **Build and run SniperBot**
   ```bash
   cargo build --release
   cargo run --bin sniper-bot -- --dry-run
   ```

### Configuration

Edit `configs/bot.toml` to configure:
- Exchange API credentials
- Trading strategies
- Risk management parameters
- Database connections

## 🖥️ Modern Web Interface

SniperBot 2.0 features a cutting-edge React/TypeScript frontend with real-time monitoring and control capabilities.

### 🚀 **Quick Start with Frontend**
```bash
# Start complete system (API + Frontend)
./scripts/start_with_frontend.sh

# Access the dashboard
# http://localhost:8084
```

### 🛠️ **Development Mode**
```bash
# Start in development mode (hot reload)
./scripts/start_dev.sh

# Access points:
# Frontend: http://localhost:3000
# API: http://localhost:8084
```

### ✨ **Frontend Features**
- **🎯 Real-time Dashboard**: Live bot status, portfolio value, strategy performance
- **⚡ Signal Feed**: Real-time trading signals with filtering and strength indicators
- **📊 Trade History**: Complete execution history with status tracking
- **🔧 Bot Control**: Start/stop bot, enable/disable strategies, emergency controls
- **🌐 WebSocket Integration**: Live updates without page refresh
- **📱 Responsive Design**: Works on desktop, tablet, and mobile

### 🎨 **Interface Sections**
- **Bot Status**: Monitor running state, portfolio value, active strategies
- **Live Signals**: Real-time signal feed with buy/sell indicators
- **Trade History**: Detailed trade execution log with P&L tracking
- **Performance**: Strategy-specific analytics and metrics
- **Settings**: Configuration and control panel

## 📊 Monitoring

Access the monitoring stack:
- **SniperBot Dashboard**: http://localhost:8084 (Main Interface)
- **Grafana Dashboard**: http://localhost:3000 (admin/sniperbot123)
- **Prometheus Metrics**: http://localhost:9090
- **QuestDB Console**: http://localhost:9000
- **Neo4j Browser**: http://localhost:7474

## 🔧 Development

### Project Structure
```
src/
├── main.rs                 # Application entry point
├── data_fetcher/          # Market data acquisition
├── strategy/              # Trading strategies
├── risk_management/       # Risk controls
├── execution/             # Order execution
├── analytics_aggregator/  # AI/ML integration
├── models/                # Data structures
└── utils/                 # Utilities & config

configs/                   # Configuration files
python_executables/        # AI/ML binaries
data/                     # Local data storage
logs/                     # Application logs
```

### Running Tests
```bash
cargo test
```

### Code Quality
```bash
cargo clippy
cargo fmt
```

## 🎯 Trading Strategies

### 🔥 **PumpFun Sniping Strategy**
**Target**: Early meme token detection and sniping on Pump.fun

**Criteria**:
- Market cap: $10k - $1M
- Age: < 24 hours
- Volume: > $5k/24h
- Bonding curve progress: 10-90%
- Confidence threshold: 75%

**Features**:
- Graduation tracking (Pump.fun → Raydium)
- Creator analysis and blacklisting
- Risk-adjusted position sizing
- Volume surge detection

### 💧 **Liquidity Pool Sniping Strategy**
**Target**: New liquidity pools on Raydium and Meteora

**Criteria**:
- Initial liquidity: $5k - $100k
- Pool age: 5 minutes - 12 hours
- Estimated APR: > 50%
- Volume/liquidity ratio: > 10%
- Confidence threshold: 80%

**Features**:
- New pool detection
- APR estimation
- Price impact analysis
- Preferred quote token filtering (SOL/USDC)

### 🧠 **Enhanced Strategy Framework**
Implement the `EnhancedStrategy` trait:
```rust
#[async_trait]
impl EnhancedStrategy for MyStrategy {
    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        // Access aggregated data, portfolio state, and market conditions
        let confidence = context.data_confidence();
        let market_cap = context.market_cap();
        // Your advanced strategy logic here
    }
}
```

## 🛡️ Risk Management

### Features
- **Position Sizing**: Fixed, percentage, volatility-adjusted
- **Stop Loss/Take Profit**: Automatic risk controls
- **Daily Loss Limits**: Circuit breakers
- **Portfolio Exposure**: Global risk monitoring
- **Emergency Stop**: Manual override capability

### Configuration
```toml
[risk_management]
global_max_exposure = 10000.0
max_daily_loss = 1000.0
max_drawdown = 0.2
emergency_stop_enabled = true
circuit_breaker_threshold = 0.05
```

## 🔌 Supported Exchanges

### Centralized Exchanges (CEX)
- Binance
- Coinbase Pro
- Kraken
- (More coming soon)

### Decentralized Exchanges (DEX)
- Solana DEXs (via Helius RPC)
- Ethereum DEXs
- Polygon DEXs

### Blockchain Integration
- **Solana**: Native SDK integration
- **Ethereum**: ethers-rs
- **Polygon**: EVM compatibility

## 📈 Performance

### Benchmarks
- **Latency**: <10ms order execution
- **Throughput**: >1000 requests/second
- **Memory**: <512MB baseline usage
- **CPU**: <50% under normal load

### Optimization
- Zero-copy deserialization
- Connection pooling
- Async I/O throughout
- Efficient data structures

## 🔐 Security

### Key Management
- Environment variable storage
- Docker secrets support
- Encrypted configuration files
- Key rotation capabilities

### Network Security
- TLS/SSL everywhere
- API rate limiting
- Request signing
- IP whitelisting support

## 📚 Documentation

- [Frontend Integration Guide](docs/frontend-integration.md)
- [API Documentation](docs/api.md)
- [Strategy Development Guide](docs/strategies.md)
- [Deployment Guide](docs/deployment.md)
- [Configuration Guide](docs/configuration.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is for educational and research purposes only. Trading cryptocurrencies involves substantial risk of loss. The authors are not responsible for any financial losses incurred through the use of this software.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/SynergiaOS/SniperBot/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SynergiaOS/SniperBot/discussions)
- **Email**: synergiaos@outlook.com

---

**Built with ❤️ in Rust** 🦀

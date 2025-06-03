# 🎯 SniperBot 2.0 - Ultra-Fast Solana Token Sniping Bot

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![Tests](https://img.shields.io/badge/tests-43%20passing-green.svg)](#testing)
[![Coverage](https://img.shields.io/badge/coverage-85%25-brightgreen.svg)](#testing)

**SniperBot 2.0** is a cutting-edge, ultra-fast trading bot specifically designed for **early token detection and sniping** on the Solana blockchain. Built with Rust for maximum performance and reliability, it features advanced AI-driven strategies, real-time data aggregation from 6+ sources, and specialized algorithms for Pump.fun and DEX sniping.

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

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Data Sources  │    │   Rust Core     │    │   AI/ML Layer   │
│                 │    │                 │    │                 │
│ • Exchanges     │───▶│ • Data Fetcher  │◀──▶│ • PyInstaller   │
│ • Blockchain    │    │ • Strategy      │    │   Executables   │
│ • Social Media  │    │ • Risk Mgmt     │    │ • ContextGem    │
│ • News Feeds    │    │ • Execution     │    │ • QLib Research │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   Storage       │
                       │                 │
                       │ • SQLite        │
                       │ • Redis         │
                       │ • QuestDB       │
                       │ • Neo4j         │
                       └─────────────────┘
```

## 🛠️ Quick Start

### Prerequisites
- Rust 1.75+
- Docker & Docker Compose
- Git

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/SynergiaOS/SniperBot.git
   cd SniperBot
   ```

2. **Set up environment**
   ```bash
   cp .env.template .env
   # Edit .env with your API keys and configuration
   ```

3. **Build and run with Docker**
   ```bash
   docker-compose up -d
   ```

4. **Or run locally**
   ```bash
   cargo build --release
   ./target/release/sniper-bot --config configs/bot.toml
   ```

### Configuration

Edit `configs/bot.toml` to configure:
- Exchange API credentials
- Trading strategies
- Risk management parameters
- Database connections

## 📊 Monitoring

Access the monitoring stack:
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

- [API Documentation](docs/api.md)
- [Strategy Development Guide](docs/strategies.md)
- [Deployment Guide](docs/deployment.md)
- [Troubleshooting](docs/troubleshooting.md)

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

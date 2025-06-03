# 🎯 SniperBot 2.0 - Trading Strategies Guide

This document provides comprehensive information about the advanced trading strategies implemented in SniperBot 2.0.

## 📋 Table of Contents

- [Strategy Framework](#strategy-framework)
- [PumpFun Sniping Strategy](#pumpfun-sniping-strategy)
- [Liquidity Pool Sniping Strategy](#liquidity-pool-sniping-strategy)
- [Strategy Configuration](#strategy-configuration)
- [Performance Metrics](#performance-metrics)
- [Custom Strategy Development](#custom-strategy-development)

## 🧠 Strategy Framework

### Enhanced Strategy Architecture

SniperBot 2.0 uses an enhanced strategy framework that provides rich context for decision-making:

```rust
pub struct StrategyContext {
    pub aggregated_data: AggregatedMarketData,
    pub portfolio: Portfolio,
    pub market_conditions: MarketConditions,
}
```

### Key Components

- **AggregatedMarketData**: Multi-source data with confidence scoring
- **Portfolio**: Current portfolio state and available balance
- **MarketConditions**: Volatility, volume trends, price momentum

### Strategy Types

- **Sniping**: Early token detection and entry
- **Liquidity**: Pool-based opportunities
- **Arbitrage**: Cross-platform price differences
- **Momentum**: Trend-following strategies
- **Mean Reversion**: Statistical arbitrage

## 🔥 PumpFun Sniping Strategy

### Overview

The PumpFun Sniping Strategy is designed to detect and capitalize on early meme token opportunities on the Pump.fun platform.

### Target Criteria

| Parameter | Value | Description |
|-----------|-------|-------------|
| Market Cap | $10k - $1M | Sweet spot for early entry |
| Token Age | < 24 hours | Newly launched tokens only |
| Daily Volume | > $5k | Minimum activity threshold |
| Bonding Curve Progress | 10-90% | Optimal entry window |
| Confidence Threshold | 75% | Minimum signal strength |

### Key Features

#### 🎓 Graduation Tracking
- Monitors tokens approaching Raydium graduation (80% bonding curve progress)
- Automatic position adjustment before graduation
- Post-graduation liquidity analysis

#### 👥 Creator Analysis
- Creator reputation scoring
- Blacklist management for known bad actors
- Historical performance tracking

#### 📊 Bonding Curve Analysis
```rust
fn calculate_bonding_curve_progress(&self, context: &StrategyContext) -> f64 {
    if let Some(market_cap) = context.market_cap() {
        // Graduation typically happens around $1M market cap
        (market_cap / 1_000_000.0).min(1.0)
    } else {
        0.0
    }
}
```

#### 🎯 Sweet Spot Detection
- Optimal market cap range: $50k - $500k
- Volume surge detection (>20% increase)
- Holder count analysis (minimum 10 holders)

### Signal Strength Calculation

The strategy uses a weighted scoring system:

- **Volume Momentum** (30%): Increasing volume trend
- **Price Momentum** (25%): Bullish price action
- **Data Confidence** (20%): Multi-source validation
- **Market Cap Position** (15%): Within optimal range
- **Newly Listed Bonus** (10%): Recent launch advantage

### Risk Management

- **Position Sizing**: 50% reduction for micro-cap tokens
- **Stop Loss**: 15% (higher for volatile meme tokens)
- **Take Profit**: 50% (capturing meme potential)
- **Cooldown**: 5 minutes between signals

## 💧 Liquidity Pool Sniping Strategy

### Overview

The Liquidity Pool Sniping Strategy targets new liquidity pools on Raydium and Meteora DEXs for early entry opportunities.

### Target Criteria

| Parameter | Value | Description |
|-----------|-------|-------------|
| Initial Liquidity | $5k - $100k | Optimal pool size range |
| Pool Age | 5 min - 12 hours | Fresh pools with stability |
| Estimated APR | > 50% | High yield requirement |
| Volume/Liquidity Ratio | > 10% | Activity indicator |
| Confidence Threshold | 80% | Higher threshold for pools |

### Key Features

#### 🆕 New Pool Detection
- Real-time monitoring of pool creation events
- Age verification to avoid immediate dumps
- Liquidity depth analysis

#### 💰 APR Estimation
```rust
fn estimate_apr(&self, context: &StrategyContext) -> f64 {
    let volume_24h = context.aggregated_data.primary_data.volume;
    let liquidity = context.market_conditions.liquidity_depth;
    
    if liquidity > 0.0 {
        // Assume 0.25% fee and calculate annualized return
        let daily_fees = volume_24h * 0.0025;
        let daily_return = daily_fees / liquidity;
        daily_return * 365.0 * 100.0
    } else {
        0.0
    }
}
```

#### 🎯 Preferred Quote Tokens
- **SOL**: Primary preference for Solana ecosystem
- **USDC**: Stable quote token preference
- **Custom filtering**: Configurable quote token priorities

#### ⚖️ Price Impact Analysis
- Maximum 3% price impact protection
- Position sizing based on available liquidity
- Slippage tolerance: 3% maximum

### Signal Strength Calculation

- **Volume Momentum** (25%): Increasing trading activity
- **Price Momentum** (20%): Positive price trend
- **APR Attractiveness** (20%): High yield potential
- **Volume/Liquidity Ratio** (15%): Quality indicator
- **Data Confidence** (10%): Source reliability
- **Preferred Pair Bonus** (10%): SOL/USDC advantage

### Risk Management

- **Position Sizing**: Limited by price impact (max 3%)
- **Stop Loss**: 10% (lower for established pools)
- **Take Profit**: 25% (moderate gains)
- **Cooldown**: 3 minutes between signals

## ⚙️ Strategy Configuration

### Configuration File Structure

```toml
[strategies.pumpfun_sniping]
enabled = true
confidence_threshold = 0.75
max_position_size = 500.0
stop_loss_percentage = 15.0
take_profit_percentage = 50.0
cooldown_seconds = 300

# Strategy-specific parameters
min_market_cap = 10000.0
max_market_cap = 1000000.0
min_volume_24h = 5000.0
max_age_hours = 24.0
graduation_threshold = 0.8

[strategies.liquidity_sniping]
enabled = true
confidence_threshold = 0.8
max_position_size = 1000.0
stop_loss_percentage = 10.0
take_profit_percentage = 25.0
cooldown_seconds = 180

# Strategy-specific parameters
min_initial_liquidity = 5000.0
max_initial_liquidity = 100000.0
min_apr = 50.0
max_price_impact = 3.0
```

### Dynamic Parameter Updates

Strategies support real-time parameter updates via API:

```bash
curl -X POST http://localhost:8080/api/v1/strategies/pumpfun_sniping/parameters \
  -H "Content-Type: application/json" \
  -d '{
    "min_market_cap": 15000.0,
    "confidence_threshold": 0.8
  }'
```

## 📊 Performance Metrics

### Key Performance Indicators

- **Signal Accuracy**: Percentage of profitable signals
- **Average Return**: Mean return per successful trade
- **Sharpe Ratio**: Risk-adjusted returns
- **Maximum Drawdown**: Largest peak-to-trough decline
- **Win Rate**: Percentage of winning trades

### Monitoring Dashboard

Access real-time strategy performance at:
- **Grafana**: http://localhost:3000/d/strategies
- **Metrics endpoint**: http://localhost:8080/api/v1/strategies/metrics

### Performance Tracking

```rust
pub struct StrategyPerformance {
    pub signals_generated: u64,
    pub successful_trades: u64,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub average_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}
```

## 🛠️ Custom Strategy Development

### Implementing a Custom Strategy

1. **Create Strategy Struct**:
```rust
#[derive(Debug, Clone)]
pub struct MyCustomStrategy {
    name: String,
    config: StrategyConfig,
    // Custom parameters
}
```

2. **Implement EnhancedStrategy Trait**:
```rust
#[async_trait]
impl EnhancedStrategy for MyCustomStrategy {
    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        // Your strategy logic here
    }
    
    // Implement other required methods...
}
```

3. **Register Strategy**:
```rust
let strategy = MyCustomStrategy::new("my_strategy".to_string());
strategy_manager.register_strategy(Box::new(strategy)).await?;
```

### Best Practices

- **Use confidence scoring** to filter weak signals
- **Implement proper risk management** with position sizing
- **Add comprehensive logging** for debugging
- **Test thoroughly** with backtesting framework
- **Monitor performance** and adjust parameters

### Testing Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_strategy_signal_generation() {
        let strategy = MyCustomStrategy::new("test".to_string());
        let context = create_test_context();
        
        let result = strategy.analyze(&context).await;
        assert!(result.is_ok());
    }
}
```

## 🔧 Troubleshooting

### Common Issues

1. **Low Signal Generation**
   - Check confidence thresholds
   - Verify data source connectivity
   - Review market conditions

2. **High False Positives**
   - Increase confidence threshold
   - Add additional filters
   - Improve data quality

3. **Performance Issues**
   - Optimize strategy logic
   - Reduce calculation complexity
   - Check data caching

### Debug Mode

Enable debug logging for detailed strategy analysis:

```bash
RUST_LOG=debug cargo run -- --dry-run
```

---

**For more information, see the [API Documentation](api.md) and [Configuration Guide](configuration.md).**

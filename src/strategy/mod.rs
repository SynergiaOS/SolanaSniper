pub mod enhanced_strategy;
pub mod pumpfun_sniping;
pub mod liquidity_sniping;
pub mod strategy_manager;

use crate::models::{MarketData, StrategySignal, TradingResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export enhanced strategy components
pub use enhanced_strategy::{
    EnhancedStrategy, StrategyContext, StrategyConfig, StrategyType,
    MarketConditions, VolumeTrend, PriceMomentum
};
pub use pumpfun_sniping::PumpFunSnipingStrategy;
pub use liquidity_sniping::LiquidityPoolSnipingStrategy;
pub use strategy_manager::{StrategyManager, StrategyPerformance};

#[async_trait]
pub trait Strategy: Send + Sync {
    async fn analyze(&self, market_data: &MarketData) -> TradingResult<Option<StrategySignal>>;
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()>;
    fn get_name(&self) -> &str;
    fn is_enabled(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMomentumStrategy {
    name: String,
    enabled: bool,
    sma_short: usize,
    sma_long: usize,
    rsi_period: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,
    price_history: Vec<f64>,
}

impl SimpleMomentumStrategy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            enabled: true,
            sma_short: 10,
            sma_long: 30,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            price_history: Vec::new(),
        }
    }

    fn calculate_sma(&self, period: usize) -> Option<f64> {
        if self.price_history.len() < period {
            return None;
        }

        let sum: f64 = self.price_history.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    fn calculate_rsi(&self) -> Option<f64> {
        if self.price_history.len() < self.rsi_period + 1 {
            return None;
        }

        let prices: Vec<f64> = self.price_history.iter().rev().take(self.rsi_period + 1).cloned().collect();
        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..prices.len() {
            let change = prices[i-1] - prices[i];
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / self.rsi_period as f64;
        let avg_loss = losses / self.rsi_period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

#[async_trait]
impl Strategy for SimpleMomentumStrategy {
    async fn analyze(&self, market_data: &MarketData) -> TradingResult<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }

        // This is a simplified implementation
        // In a real strategy, you'd want to maintain state properly
        let mut strategy = self.clone();
        strategy.price_history.push(market_data.price);

        // Keep only the last 100 prices to avoid memory issues
        if strategy.price_history.len() > 100 {
            strategy.price_history.remove(0);
        }

        let sma_short = strategy.calculate_sma(strategy.sma_short);
        let sma_long = strategy.calculate_sma(strategy.sma_long);
        let rsi = strategy.calculate_rsi();

        if let (Some(short), Some(long), Some(rsi_value)) = (sma_short, sma_long, rsi) {
            let signal = if short > long && rsi_value < strategy.rsi_oversold {
                Some(StrategySignal {
                    strategy: strategy.name.clone(),
                    symbol: market_data.symbol.clone(),
                    signal_type: crate::models::SignalType::Buy,
                    strength: 0.8,
                    price: market_data.price,
                    size: 100.0, // This should be calculated based on risk management
                    metadata: serde_json::json!({
                        "sma_short": short,
                        "sma_long": long,
                        "rsi": rsi_value,
                        "reason": "SMA crossover + oversold RSI"
                    }),
                    timestamp: market_data.timestamp,
                })
            } else if short < long && rsi_value > strategy.rsi_overbought {
                Some(StrategySignal {
                    strategy: strategy.name.clone(),
                    symbol: market_data.symbol.clone(),
                    signal_type: crate::models::SignalType::Sell,
                    strength: 0.8,
                    price: market_data.price,
                    size: 100.0,
                    metadata: serde_json::json!({
                        "sma_short": short,
                        "sma_long": long,
                        "rsi": rsi_value,
                        "reason": "SMA crossover + overbought RSI"
                    }),
                    timestamp: market_data.timestamp,
                })
            } else {
                None
            };

            Ok(signal)
        } else {
            Ok(None) // Not enough data yet
        }
    }

    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()> {
        if let Some(sma_short) = parameters.get("sma_short") {
            if let Some(value) = sma_short.as_u64() {
                self.sma_short = value as usize;
            }
        }

        if let Some(sma_long) = parameters.get("sma_long") {
            if let Some(value) = sma_long.as_u64() {
                self.sma_long = value as usize;
            }
        }

        if let Some(rsi_period) = parameters.get("rsi_period") {
            if let Some(value) = rsi_period.as_u64() {
                self.rsi_period = value as usize;
            }
        }

        if let Some(rsi_oversold) = parameters.get("rsi_oversold") {
            if let Some(value) = rsi_oversold.as_f64() {
                self.rsi_oversold = value;
            }
        }

        if let Some(rsi_overbought) = parameters.get("rsi_overbought") {
            if let Some(value) = rsi_overbought.as_f64() {
                self.rsi_overbought = value;
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DataSource, MarketData};
    use chrono::Utc;

    #[tokio::test]
    async fn test_simple_momentum_strategy() {
        let strategy = SimpleMomentumStrategy::new("test_strategy".to_string());
        
        let market_data = MarketData {
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            volume: 1000.0,
            bid: Some(49999.0),
            ask: Some(50001.0),
            timestamp: Utc::now(),
            source: DataSource::Binance,
        };

        let result = strategy.analyze(&market_data).await;
        assert!(result.is_ok());
        
        // With only one data point, should return None
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_sma_calculation() {
        let mut strategy = SimpleMomentumStrategy::new("test".to_string());
        strategy.price_history = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        
        let sma = strategy.calculate_sma(5);
        assert_eq!(sma, Some(30.0));
        
        let sma_short = strategy.calculate_sma(3);
        assert_eq!(sma_short, Some(40.0));
    }
}

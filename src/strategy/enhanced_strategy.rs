use crate::data_fetcher::data_aggregator::AggregatedMarketData;
use crate::models::{MarketData, MarketEvent, StrategySignal, TradingResult, Portfolio};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tracing::debug;

/// Enhanced strategy context with aggregated data and portfolio state
#[derive(Debug, Clone, Serialize)]
pub struct StrategyContext {
    pub aggregated_data: AggregatedMarketData,
    pub portfolio: Portfolio,
    pub market_conditions: MarketConditions,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketConditions {
    pub volatility: f64,
    pub volume_trend: VolumeTrend,
    pub price_momentum: PriceMomentum,
    pub liquidity_depth: f64,
    pub market_cap: Option<f64>,
    pub age_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceMomentum {
    Bullish,
    Bearish,
    Sideways,
}

/// Enhanced strategy trait for advanced trading strategies
#[async_trait]
pub trait EnhancedStrategy: Send + Sync {
    /// Analyze market data and return trading signal
    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>>;
    
    /// Update strategy parameters dynamically
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()>;
    
    /// Get strategy name
    fn get_name(&self) -> &str;
    
    /// Check if strategy is enabled
    fn is_enabled(&self) -> bool;
    
    /// Get strategy confidence level (0.0 - 1.0)
    fn get_confidence(&self) -> f64;
    
    /// Get minimum required data sources for this strategy
    fn required_data_sources(&self) -> Vec<String>;
    
    /// Validate if strategy can operate with current market conditions
    fn can_operate(&self, context: &StrategyContext) -> bool;
    
    /// Get strategy type for categorization
    fn get_strategy_type(&self) -> StrategyType;
    
    /// Get minimum confidence score required for signal generation
    fn min_confidence_threshold(&self) -> f64;

    /// Process real-time market event and potentially generate trading signal
    /// This method is called for every MarketEvent received via WebSocket
    async fn on_market_event(&self, event: &MarketEvent, _context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        // Default implementation - strategies can override this
        // For now, we'll ignore events and return None
        debug!("Strategy {} received market event: {:?}", self.get_name(), event);
        Ok(None)
    }

    /// Check if this strategy is interested in a specific market event type
    fn is_interested_in_event(&self, event: &MarketEvent) -> bool {
        // Default implementation - strategies should override this for efficiency
        match event {
            MarketEvent::PriceUpdate { .. } => true,
            MarketEvent::NewTransaction { .. } => true,
            MarketEvent::LiquidityUpdate { .. } => true,
            MarketEvent::NewTokenListing { .. } => true,
            MarketEvent::WhaleAlert { .. } => true,
            MarketEvent::ConnectionStatus { .. } => false, // Usually not interesting for trading
            MarketEvent::RawMessage { .. } => false, // Usually not interesting for trading
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Sniping,
    Arbitrage,
    Momentum,
    MeanReversion,
    Liquidity,
    Graduation,
    MeteoraDLMM,
    LiquidityProvision,
    VolumeSpike,
}

/// Strategy configuration for different types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub confidence_threshold: f64,
    pub max_position_size: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
    pub cooldown_seconds: u64,
    pub required_sources: Vec<String>,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            confidence_threshold: 0.7,
            max_position_size: 1000.0,
            stop_loss_percentage: 5.0,
            take_profit_percentage: 15.0,
            cooldown_seconds: 60,
            required_sources: vec!["jupiter".to_string()],
        }
    }
}

impl StrategyContext {
    pub fn new(
        aggregated_data: AggregatedMarketData,
        portfolio: Portfolio,
        market_conditions: MarketConditions,
    ) -> Self {
        Self {
            aggregated_data,
            portfolio,
            market_conditions,
        }
    }

    /// Calculate price change percentage over time
    pub fn price_change_percentage(&self) -> f64 {
        if self.aggregated_data.secondary_data.is_empty() {
            return 0.0;
        }

        let current_price = self.aggregated_data.primary_data.price;
        let first_price = self.aggregated_data.secondary_data[0].price;
        
        if first_price > 0.0 {
            ((current_price - first_price) / first_price) * 100.0
        } else {
            0.0
        }
    }

    /// Check if token is newly listed (less than 24 hours)
    pub fn is_newly_listed(&self) -> bool {
        self.market_conditions.age_hours.map_or(false, |age| age < 24.0)
    }

    /// Get average price across all sources
    pub fn average_price(&self) -> f64 {
        let mut total = self.aggregated_data.primary_data.price;
        let mut count = 1;

        for data in &self.aggregated_data.secondary_data {
            total += data.price;
            count += 1;
        }

        total / count as f64
    }

    /// Calculate price spread between sources
    pub fn price_spread_percentage(&self) -> f64 {
        if self.aggregated_data.secondary_data.is_empty() {
            return 0.0;
        }

        let mut prices: Vec<f64> = vec![self.aggregated_data.primary_data.price];
        prices.extend(self.aggregated_data.secondary_data.iter().map(|d| d.price));

        let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if min_price > 0.0 {
            ((max_price - min_price) / min_price) * 100.0
        } else {
            0.0
        }
    }

    /// Check if volume is increasing significantly
    pub fn is_volume_surging(&self) -> bool {
        matches!(self.market_conditions.volume_trend, VolumeTrend::Increasing)
            && self.aggregated_data.primary_data.volume > 0.0
    }

    /// Get market cap if available
    pub fn market_cap(&self) -> Option<f64> {
        self.market_conditions.market_cap
    }

    /// Check if this is a micro-cap token (< $1M market cap)
    pub fn is_micro_cap(&self) -> bool {
        self.market_conditions.market_cap.map_or(false, |cap| cap < 1_000_000.0)
    }

    /// Calculate confidence score based on data quality
    pub fn data_confidence(&self) -> f64 {
        let base_confidence = self.aggregated_data.confidence_score;
        let source_bonus = (self.aggregated_data.sources_count as f64 - 1.0) * 0.1;
        let latency_penalty = if self.aggregated_data.latency_ms > 1000 { 0.1 } else { 0.0 };
        
        (base_confidence + source_bonus - latency_penalty).max(0.0).min(1.0)
    }
}

impl MarketConditions {
    pub fn from_market_data(data: &MarketData, volume_history: &[f64]) -> Self {
        let volume_trend = if volume_history.len() >= 2 {
            let recent_avg = volume_history.iter().rev().take(3).sum::<f64>() / 3.0;
            let older_avg = volume_history.iter().rev().skip(3).take(3).sum::<f64>() / 3.0;
            
            if recent_avg > older_avg * 1.2 {
                VolumeTrend::Increasing
            } else if recent_avg < older_avg * 0.8 {
                VolumeTrend::Decreasing
            } else {
                VolumeTrend::Stable
            }
        } else {
            VolumeTrend::Stable
        };

        Self {
            volatility: 0.0, // Would be calculated from price history
            volume_trend,
            price_momentum: PriceMomentum::Sideways, // Would be calculated from price history
            liquidity_depth: data.volume,
            market_cap: None,
            age_hours: None,
        }
    }

    pub fn with_market_cap(mut self, market_cap: f64) -> Self {
        self.market_cap = Some(market_cap);
        self
    }

    pub fn with_age_hours(mut self, age_hours: f64) -> Self {
        self.age_hours = Some(age_hours);
        self
    }

    pub fn with_volatility(mut self, volatility: f64) -> Self {
        self.volatility = volatility;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DataSource;
    use chrono::Utc;

    fn create_test_context() -> StrategyContext {
        let market_data = MarketData {
            symbol: "TEST/SOL".to_string(),
            price: 0.001,
            volume: 50000.0,
            bid: Some(0.0009),
            ask: Some(0.0011),
            timestamp: Utc::now(),
            source: DataSource::Solana,
        };

        let aggregated_data = AggregatedMarketData {
            primary_data: market_data,
            secondary_data: vec![],
            sources_count: 1,
            confidence_score: 0.8,
            latency_ms: 150,
        };

        let portfolio = Portfolio {
            total_value: 10000.0,
            available_balance: 5000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: vec![],
            daily_pnl: 0.0,
            max_drawdown: 0.0,
            updated_at: Utc::now(),
        };

        let market_conditions = MarketConditions {
            volatility: 0.15,
            volume_trend: VolumeTrend::Increasing,
            price_momentum: PriceMomentum::Bullish,
            liquidity_depth: 50000.0,
            market_cap: Some(500000.0),
            age_hours: Some(2.0),
        };

        StrategyContext::new(aggregated_data, portfolio, market_conditions)
    }

    #[test]
    fn test_strategy_context_creation() {
        let context = create_test_context();
        assert_eq!(context.aggregated_data.primary_data.symbol, "TEST/SOL");
        assert!(context.is_newly_listed());
        assert!(context.is_micro_cap());
    }

    #[test]
    fn test_price_calculations() {
        let context = create_test_context();
        assert_eq!(context.average_price(), 0.001);
        assert_eq!(context.price_spread_percentage(), 0.0);
    }

    #[test]
    fn test_market_conditions() {
        let context = create_test_context();
        assert!(context.is_volume_surging());
        assert_eq!(context.market_cap(), Some(500000.0));
        assert!(context.data_confidence() > 0.7);
    }
}

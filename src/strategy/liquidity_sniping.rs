use crate::models::{SignalType, StrategySignal, TradingResult};
use crate::strategy::enhanced_strategy::{
    EnhancedStrategy, StrategyConfig, StrategyContext, StrategyType, VolumeTrend, PriceMomentum
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tracing::{debug, info};

/// Liquidity pool sniping strategy for new Raydium/Meteora pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPoolSnipingStrategy {
    name: String,
    config: StrategyConfig,
    
    // Pool-specific parameters
    min_initial_liquidity: f64,
    max_initial_liquidity: f64,
    min_pool_age_minutes: f64,
    max_pool_age_hours: f64,
    min_apr: f64,
    max_price_impact: f64,
    
    // Token criteria
    min_token_holders: u32,
    max_token_supply: f64,
    required_token_decimals: Option<u8>,
    
    // Risk parameters
    max_slippage_bps: u16,
    min_volume_ratio: f64, // Volume to liquidity ratio
    blacklisted_tokens: Vec<String>,
    preferred_quote_tokens: Vec<String>,
    
    // Performance tracking
    pools_analyzed: u64,
    signals_generated: u64,
    successful_entries: u64,
    last_signal_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl LiquidityPoolSnipingStrategy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: StrategyConfig {
                enabled: true,
                confidence_threshold: 0.8,
                max_position_size: 1000.0,
                stop_loss_percentage: 10.0,
                take_profit_percentage: 25.0,
                cooldown_seconds: 180, // 3 minute cooldown
                required_sources: vec!["raydium".to_string(), "meteora".to_string()],
            },
            
            // Pool criteria for sniping new liquidity
            min_initial_liquidity: 5_000.0,    // $5k minimum
            max_initial_liquidity: 100_000.0,  // $100k maximum (before it gets crowded)
            min_pool_age_minutes: 5.0,         // At least 5 minutes old (avoid immediate dumps)
            max_pool_age_hours: 12.0,          // Maximum 12 hours old
            min_apr: 50.0,                     // Minimum 50% APR
            max_price_impact: 3.0,             // Maximum 3% price impact
            
            min_token_holders: 20,             // At least 20 holders
            max_token_supply: 1_000_000_000.0, // Max 1B token supply
            required_token_decimals: None,      // Any decimals
            
            max_slippage_bps: 300,             // 3% max slippage
            min_volume_ratio: 0.1,             // Volume should be at least 10% of liquidity
            blacklisted_tokens: vec![],
            preferred_quote_tokens: vec![
                "So11111111111111111111111111111111111111112".to_string(), // SOL
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            ],
            
            pools_analyzed: 0,
            signals_generated: 0,
            successful_entries: 0,
            last_signal_time: None,
        }
    }

    /// Check if pool meets basic liquidity criteria
    fn meets_liquidity_criteria(&self, context: &StrategyContext) -> bool {
        let liquidity = context.market_conditions.liquidity_depth;
        
        if liquidity < self.min_initial_liquidity {
            debug!("Liquidity {} below minimum {}", liquidity, self.min_initial_liquidity);
            return false;
        }

        if liquidity > self.max_initial_liquidity {
            debug!("Liquidity {} above maximum {}", liquidity, self.max_initial_liquidity);
            return false;
        }

        true
    }

    /// Check pool age criteria
    fn meets_age_criteria(&self, context: &StrategyContext) -> bool {
        if let Some(age_hours) = context.market_conditions.age_hours {
            let age_minutes = age_hours * 60.0;
            
            if age_minutes < self.min_pool_age_minutes {
                debug!("Pool age {} minutes below minimum {}", age_minutes, self.min_pool_age_minutes);
                return false;
            }

            if age_hours > self.max_pool_age_hours {
                debug!("Pool age {} hours above maximum {}", age_hours, self.max_pool_age_hours);
                return false;
            }

            true
        } else {
            debug!("No pool age data available");
            false
        }
    }

    /// Calculate volume to liquidity ratio
    fn calculate_volume_ratio(&self, context: &StrategyContext) -> f64 {
        let volume = context.aggregated_data.primary_data.volume;
        let liquidity = context.market_conditions.liquidity_depth;
        
        if liquidity > 0.0 {
            volume / liquidity
        } else {
            0.0
        }
    }

    /// Check if token is in preferred quote tokens
    fn is_preferred_quote_pair(&self, symbol: &str) -> bool {
        // Check if symbol contains any of the preferred quote tokens
        self.preferred_quote_tokens.iter().any(|quote| {
            symbol.contains("SOL") || symbol.contains("USDC") || symbol.contains(quote)
        })
    }

    /// Calculate APR estimation (simplified)
    fn estimate_apr(&self, context: &StrategyContext) -> f64 {
        // Simplified APR calculation based on volume and fees
        let volume_24h = context.aggregated_data.primary_data.volume;
        let liquidity = context.market_conditions.liquidity_depth;
        
        if liquidity > 0.0 {
            // Assume 0.25% fee and calculate annualized return
            let daily_fees = volume_24h * 0.0025;
            let daily_return = daily_fees / liquidity;
            daily_return * 365.0 * 100.0 // Convert to percentage
        } else {
            0.0
        }
    }

    /// Calculate signal strength for liquidity pool
    fn calculate_signal_strength(&self, context: &StrategyContext) -> f64 {
        let mut strength = 0.0;

        // Volume momentum (25% weight)
        if matches!(context.market_conditions.volume_trend, VolumeTrend::Increasing) {
            strength += 0.25;
        }

        // Price momentum (20% weight)
        if matches!(context.market_conditions.price_momentum, PriceMomentum::Bullish) {
            strength += 0.2;
        }

        // APR attractiveness (20% weight)
        let estimated_apr = self.estimate_apr(context);
        if estimated_apr >= self.min_apr {
            let apr_score = (estimated_apr / 200.0).min(1.0); // Cap at 200% APR
            strength += 0.2 * apr_score;
        }

        // Volume to liquidity ratio (15% weight)
        let volume_ratio = self.calculate_volume_ratio(context);
        if volume_ratio >= self.min_volume_ratio {
            let ratio_score = (volume_ratio / 0.5).min(1.0); // Cap at 50% ratio
            strength += 0.15 * ratio_score;
        }

        // Data confidence (10% weight)
        strength += context.data_confidence() * 0.1;

        // Preferred pair bonus (10% weight)
        if self.is_preferred_quote_pair(&context.aggregated_data.primary_data.symbol) {
            strength += 0.1;
        }

        strength.min(1.0)
    }

    /// Check cooldown period
    fn is_in_cooldown(&self) -> bool {
        if let Some(last_signal) = self.last_signal_time {
            let cooldown_duration = chrono::Duration::seconds(self.config.cooldown_seconds as i64);
            Utc::now() - last_signal < cooldown_duration
        } else {
            false
        }
    }

    /// Generate liquidity pool entry signal
    fn generate_entry_signal(&self, context: &StrategyContext, strength: f64) -> StrategySignal {
        let symbol = &context.aggregated_data.primary_data.symbol;
        let price = context.average_price();
        
        // Calculate position size based on liquidity and risk
        let liquidity = context.market_conditions.liquidity_depth;
        let max_impact_size = liquidity * (self.max_price_impact / 100.0);
        let risk_adjusted_size = self.config.max_position_size.min(max_impact_size);

        StrategySignal {
            strategy: self.name.clone(),
            symbol: symbol.clone(),
            signal_type: SignalType::Buy,
            strength,
            price,
            size: risk_adjusted_size,
            metadata: serde_json::json!({
                "strategy_type": "liquidity_pool_sniping",
                "pool_liquidity": liquidity,
                "estimated_apr": self.estimate_apr(context),
                "volume_ratio": self.calculate_volume_ratio(context),
                "pool_age_hours": context.market_conditions.age_hours,
                "data_sources": context.aggregated_data.sources_count,
                "confidence": context.data_confidence(),
                "is_preferred_pair": self.is_preferred_quote_pair(symbol),
                "stop_loss_pct": self.config.stop_loss_percentage,
                "take_profit_pct": self.config.take_profit_percentage,
                "max_slippage_bps": self.max_slippage_bps,
                "max_price_impact": self.max_price_impact,
            }),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl EnhancedStrategy for LiquidityPoolSnipingStrategy {
    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Check cooldown
        if self.is_in_cooldown() {
            debug!("Strategy {} in cooldown", self.name);
            return Ok(None);
        }

        // Check if we can operate
        if !self.can_operate(context) {
            debug!("Strategy {} cannot operate with current context", self.name);
            return Ok(None);
        }

        // Check liquidity criteria
        if !self.meets_liquidity_criteria(context) {
            return Ok(None);
        }

        // Check age criteria
        if !self.meets_age_criteria(context) {
            return Ok(None);
        }

        // Calculate signal strength
        let strength = self.calculate_signal_strength(context);
        
        if strength >= self.config.confidence_threshold {
            info!(
                "🏊 Liquidity pool sniping signal for {} with strength {:.2} (APR: {:.1}%)",
                context.aggregated_data.primary_data.symbol,
                strength,
                self.estimate_apr(context)
            );

            let signal = self.generate_entry_signal(context, strength);
            Ok(Some(signal))
        } else {
            debug!(
                "Signal strength {:.2} below threshold {:.2} for {}",
                strength,
                self.config.confidence_threshold,
                context.aggregated_data.primary_data.symbol
            );
            Ok(None)
        }
    }

    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()> {
        if let Some(min_liquidity) = parameters.get("min_initial_liquidity") {
            if let Some(value) = min_liquidity.as_f64() {
                self.min_initial_liquidity = value;
            }
        }

        if let Some(max_liquidity) = parameters.get("max_initial_liquidity") {
            if let Some(value) = max_liquidity.as_f64() {
                self.max_initial_liquidity = value;
            }
        }

        if let Some(min_apr) = parameters.get("min_apr") {
            if let Some(value) = min_apr.as_f64() {
                self.min_apr = value;
            }
        }

        if let Some(confidence_threshold) = parameters.get("confidence_threshold") {
            if let Some(value) = confidence_threshold.as_f64() {
                self.config.confidence_threshold = value;
            }
        }

        info!("Updated liquidity pool sniping strategy parameters");
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn get_confidence(&self) -> f64 {
        if self.signals_generated > 0 {
            self.successful_entries as f64 / self.signals_generated as f64
        } else {
            0.75 // Default confidence
        }
    }

    fn required_data_sources(&self) -> Vec<String> {
        self.config.required_sources.clone()
    }

    fn can_operate(&self, context: &StrategyContext) -> bool {
        // Need at least one DEX data source
        let has_dex_data = context.aggregated_data.sources_count > 0;
        
        // Need sufficient data confidence
        let sufficient_confidence = context.data_confidence() >= 0.7;
        
        // Need liquidity data
        let has_liquidity_data = context.market_conditions.liquidity_depth > 0.0;

        has_dex_data && sufficient_confidence && has_liquidity_data
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::Liquidity
    }

    fn min_confidence_threshold(&self) -> f64 {
        self.config.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_fetcher::data_aggregator::AggregatedMarketData;
    use crate::models::{DataSource, MarketData, Portfolio};
    use crate::strategy::enhanced_strategy::MarketConditions;

    fn create_test_context() -> StrategyContext {
        let market_data = MarketData {
            symbol: "TOKEN/SOL".to_string(),
            price: 0.01,
            volume: 5000.0,
            bid: Some(0.009),
            ask: Some(0.011),
            timestamp: Utc::now(),
            source: DataSource::Solana,
        };

        let aggregated_data = AggregatedMarketData {
            primary_data: market_data,
            secondary_data: vec![],
            sources_count: 2,
            confidence_score: 0.9,
            latency_ms: 100,
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
            liquidity_depth: 25000.0,
            market_cap: Some(250_000.0),
            age_hours: Some(8.0),
        };

        StrategyContext::new(aggregated_data, portfolio, market_conditions)
    }

    #[tokio::test]
    async fn test_liquidity_pool_sniping_strategy() {
        let strategy = LiquidityPoolSnipingStrategy::new("test_liquidity".to_string());
        let context = create_test_context();

        assert!(strategy.can_operate(&context));
        assert!(strategy.meets_liquidity_criteria(&context));
        assert!(strategy.meets_age_criteria(&context));

        let result = strategy.analyze(&context).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_apr_estimation() {
        let strategy = LiquidityPoolSnipingStrategy::new("test".to_string());
        let context = create_test_context();
        
        let apr = strategy.estimate_apr(&context);
        assert!(apr > 0.0);
    }

    #[test]
    fn test_volume_ratio_calculation() {
        let strategy = LiquidityPoolSnipingStrategy::new("test".to_string());
        let context = create_test_context();
        
        let ratio = strategy.calculate_volume_ratio(&context);
        assert!(ratio >= 0.0);
    }

    #[test]
    fn test_preferred_quote_pair() {
        let strategy = LiquidityPoolSnipingStrategy::new("test".to_string());
        
        assert!(strategy.is_preferred_quote_pair("TOKEN/SOL"));
        assert!(strategy.is_preferred_quote_pair("MEME/USDC"));
        assert!(!strategy.is_preferred_quote_pair("TOKEN/UNKNOWN"));
    }
}

use crate::models::{MarketEvent, SignalType, StrategySignal, TradingResult, TransactionType};
use crate::strategy::enhanced_strategy::{
    EnhancedStrategy, StrategyConfig, StrategyContext, StrategyType, VolumeTrend, PriceMomentum
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tracing::{debug, info, instrument};

/// PumpFun sniping strategy for early meme token detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunSnipingStrategy {
    name: String,
    config: StrategyConfig,
    pumpfun_client: Option<String>, // We'll store client reference differently
    
    // Strategy-specific parameters
    min_market_cap: f64,
    max_market_cap: f64,
    min_volume_24h: f64,
    max_age_hours: f64,
    min_holder_count: u32,
    graduation_threshold: f64,
    bonding_curve_progress_min: f64,
    bonding_curve_progress_max: f64,
    
    // Risk parameters
    max_slippage_bps: u16,
    min_liquidity: f64,
    blacklisted_creators: Vec<String>,
    
    // Performance tracking
    signals_generated: u64,
    successful_trades: u64,
    last_signal_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl PumpFunSnipingStrategy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: StrategyConfig {
                enabled: true,
                confidence_threshold: 0.75,
                max_position_size: 500.0, // Conservative for meme tokens
                stop_loss_percentage: 15.0, // Higher stop loss for volatile tokens
                take_profit_percentage: 50.0, // Higher take profit for meme potential
                cooldown_seconds: 300, // 5 minute cooldown between signals
                required_sources: vec!["pumpfun".to_string()],
            },
            pumpfun_client: None,
            
            // Default parameters for early token sniping
            min_market_cap: 10_000.0,      // $10k minimum
            max_market_cap: 1_000_000.0,   // $1M maximum (before it gets too expensive)
            min_volume_24h: 5_000.0,       // $5k minimum daily volume
            max_age_hours: 24.0,           // Only tokens younger than 24 hours
            min_holder_count: 10,          // At least 10 holders
            graduation_threshold: 0.8,     // 80% progress toward Raydium graduation
            bonding_curve_progress_min: 0.1, // At least 10% progress
            bonding_curve_progress_max: 0.9,  // Not more than 90% (leave room for growth)
            
            max_slippage_bps: 500,         // 5% max slippage
            min_liquidity: 1_000.0,        // $1k minimum liquidity
            blacklisted_creators: vec![],
            
            signals_generated: 0,
            successful_trades: 0,
            last_signal_time: None,
        }
    }

    /// Check if token meets basic criteria for sniping
    fn meets_basic_criteria(&self, context: &StrategyContext) -> bool {
        // Check market cap range
        if let Some(market_cap) = context.market_cap() {
            if market_cap < self.min_market_cap || market_cap > self.max_market_cap {
                debug!("Market cap {} outside range [{}, {}]", market_cap, self.min_market_cap, self.max_market_cap);
                return false;
            }
        } else {
            debug!("No market cap data available");
            return false;
        }

        // Check age
        if let Some(age) = context.market_conditions.age_hours {
            if age > self.max_age_hours {
                debug!("Token age {} hours exceeds maximum {}", age, self.max_age_hours);
                return false;
            }
        }

        // Check volume
        let volume = context.aggregated_data.primary_data.volume;
        if volume < self.min_volume_24h {
            debug!("Volume {} below minimum {}", volume, self.min_volume_24h);
            return false;
        }

        // Check liquidity depth
        if context.market_conditions.liquidity_depth < self.min_liquidity {
            debug!("Liquidity {} below minimum {}", context.market_conditions.liquidity_depth, self.min_liquidity);
            return false;
        }

        true
    }

    /// Calculate bonding curve progress (simplified)
    fn calculate_bonding_curve_progress(&self, context: &StrategyContext) -> f64 {
        // This would normally require specific PumpFun data
        // For now, we'll estimate based on market cap
        if let Some(market_cap) = context.market_cap() {
            // Assume graduation happens around $1M market cap
            (market_cap / 1_000_000.0).min(1.0)
        } else {
            0.0
        }
    }

    /// Check for graduation signals (token moving to Raydium)
    fn check_graduation_signal(&self, context: &StrategyContext) -> bool {
        let progress = self.calculate_bonding_curve_progress(context);
        progress >= self.graduation_threshold
    }

    /// Calculate signal strength based on multiple factors
    fn calculate_signal_strength(&self, context: &StrategyContext) -> f64 {
        let mut strength = 0.0;

        // Volume momentum (30% weight)
        if matches!(context.market_conditions.volume_trend, VolumeTrend::Increasing) {
            strength += 0.3;
        }

        // Price momentum (25% weight)
        if matches!(context.market_conditions.price_momentum, PriceMomentum::Bullish) {
            strength += 0.25;
        }

        // Data confidence (20% weight)
        strength += context.data_confidence() * 0.2;

        // Market cap sweet spot (15% weight)
        if let Some(market_cap) = context.market_cap() {
            let optimal_range = 50_000.0..=500_000.0; // $50k - $500k sweet spot
            if optimal_range.contains(&market_cap) {
                strength += 0.15;
            }
        }

        // Newly listed bonus (10% weight)
        if context.is_newly_listed() {
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

    /// Generate buy signal for early token
    fn generate_buy_signal(&self, context: &StrategyContext, strength: f64) -> StrategySignal {
        let symbol = &context.aggregated_data.primary_data.symbol;
        let price = context.average_price();
        
        // Calculate position size based on risk and market cap
        let base_size = self.config.max_position_size;
        let risk_adjusted_size = if context.is_micro_cap() {
            base_size * 0.5 // Reduce size for micro caps
        } else {
            base_size
        };

        StrategySignal {
            strategy: self.name.clone(),
            symbol: symbol.clone(),
            signal_type: SignalType::Buy,
            strength,
            price,
            size: risk_adjusted_size,
            metadata: serde_json::json!({
                "strategy_type": "pumpfun_sniping",
                "market_cap": context.market_cap(),
                "age_hours": context.market_conditions.age_hours,
                "volume_24h": context.aggregated_data.primary_data.volume,
                "bonding_curve_progress": self.calculate_bonding_curve_progress(context),
                "data_sources": context.aggregated_data.sources_count,
                "confidence": context.data_confidence(),
                "is_graduation_candidate": self.check_graduation_signal(context),
                "stop_loss_pct": self.config.stop_loss_percentage,
                "take_profit_pct": self.config.take_profit_percentage,
                "max_slippage_bps": self.max_slippage_bps,
            }),
            timestamp: Utc::now(),
        }
    }

    /// Calculate signal strength for new token listings
    fn calculate_new_token_strength(&self, price: f64, liquidity: f64, creator: Option<&str>) -> f64 {
        let mut strength = 0.0;

        // Liquidity score (40% weight)
        if liquidity >= 10_000.0 {
            strength += 0.4 * (liquidity / 50_000.0).min(1.0);
        }

        // Price range score (30% weight)
        if price > 0.0 && price < 0.01 {
            strength += 0.3; // Good entry price range
        }

        // Creator reputation (20% weight) - simplified
        if let Some(_creator) = creator {
            if !self.blacklisted_creators.contains(&_creator.to_string()) {
                strength += 0.2;
            }
        } else {
            strength += 0.1; // Unknown creator gets partial score
        }

        // New token bonus (10% weight)
        strength += 0.1;

        strength.min(1.0)
    }

    /// Calculate signal strength for graduation events
    fn calculate_graduation_strength(&self, liquidity: f64, price: f64) -> f64 {
        let mut strength = 0.0;

        // High liquidity indicates successful graduation (50% weight)
        if liquidity >= 50_000.0 {
            strength += 0.5 * (liquidity / 200_000.0).min(1.0);
        }

        // Price stability (30% weight) - simplified
        if price > 0.0 {
            strength += 0.3;
        }

        // Graduation timing bonus (20% weight)
        strength += 0.2;

        strength.min(1.0)
    }

    /// Calculate signal strength for whale following
    fn calculate_whale_follow_strength(&self, amount: f64, price: f64) -> f64 {
        let mut strength = 0.0;

        // Transaction size (60% weight)
        if amount >= 1000.0 {
            strength += 0.6 * (amount / 10_000.0).min(1.0);
        }

        // Price reasonableness (25% weight)
        if price > 0.0 && price < 1.0 {
            strength += 0.25;
        }

        // Whale following bonus (15% weight)
        strength += 0.15;

        strength.min(1.0)
    }

    /// Generate signal for new token listing
    fn generate_new_token_signal(&self, token_address: &str, symbol: &str, price: f64, strength: f64, timestamp: u64) -> StrategySignal {
        StrategySignal {
            strategy: self.name.clone(),
            symbol: format!("{}/SOL", symbol),
            signal_type: SignalType::Buy,
            strength,
            price,
            size: self.config.max_position_size * 0.5, // Conservative size for new tokens
            metadata: serde_json::json!({
                "strategy_type": "pumpfun_new_token",
                "token_address": token_address,
                "event_timestamp": timestamp,
                "signal_reason": "new_token_listing",
                "stop_loss_pct": self.config.stop_loss_percentage,
                "take_profit_pct": self.config.take_profit_percentage,
            }),
            timestamp: Utc::now(),
        }
    }

    /// Generate signal for graduation event
    fn generate_graduation_signal(&self, token_address: &str, price: f64, strength: f64, timestamp: u64) -> StrategySignal {
        StrategySignal {
            strategy: self.name.clone(),
            symbol: format!("{}/SOL", token_address),
            signal_type: SignalType::Buy,
            strength,
            price,
            size: self.config.max_position_size * 0.8, // Larger size for graduation
            metadata: serde_json::json!({
                "strategy_type": "pumpfun_graduation",
                "token_address": token_address,
                "event_timestamp": timestamp,
                "signal_reason": "graduation_to_raydium",
                "stop_loss_pct": self.config.stop_loss_percentage * 0.8, // Tighter stop loss
                "take_profit_pct": self.config.take_profit_percentage * 1.5, // Higher target
            }),
            timestamp: Utc::now(),
        }
    }

    /// Generate signal for whale following
    fn generate_whale_follow_signal(&self, token_address: &str, price: f64, strength: f64, timestamp: u64) -> StrategySignal {
        StrategySignal {
            strategy: self.name.clone(),
            symbol: format!("{}/SOL", token_address),
            signal_type: SignalType::Buy,
            strength,
            price,
            size: self.config.max_position_size * 0.3, // Smaller size for whale following
            metadata: serde_json::json!({
                "strategy_type": "pumpfun_whale_follow",
                "token_address": token_address,
                "event_timestamp": timestamp,
                "signal_reason": "whale_transaction_detected",
                "stop_loss_pct": self.config.stop_loss_percentage,
                "take_profit_pct": self.config.take_profit_percentage,
            }),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl EnhancedStrategy for PumpFunSnipingStrategy {
    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Check cooldown
        if self.is_in_cooldown() {
            debug!("Strategy {} in cooldown", self.name);
            return Ok(None);
        }

        // Check if we can operate with current context
        if !self.can_operate(context) {
            debug!("Strategy {} cannot operate with current context", self.name);
            return Ok(None);
        }

        // Check basic criteria
        if !self.meets_basic_criteria(context) {
            return Ok(None);
        }

        // Calculate signal strength
        let strength = self.calculate_signal_strength(context);
        
        if strength >= self.config.confidence_threshold {
            info!(
                "🎯 PumpFun sniping signal generated for {} with strength {:.2}",
                context.aggregated_data.primary_data.symbol,
                strength
            );

            let signal = self.generate_buy_signal(context, strength);
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
        if let Some(min_market_cap) = parameters.get("min_market_cap") {
            if let Some(value) = min_market_cap.as_f64() {
                self.min_market_cap = value;
            }
        }

        if let Some(max_market_cap) = parameters.get("max_market_cap") {
            if let Some(value) = max_market_cap.as_f64() {
                self.max_market_cap = value;
            }
        }

        if let Some(min_volume_24h) = parameters.get("min_volume_24h") {
            if let Some(value) = min_volume_24h.as_f64() {
                self.min_volume_24h = value;
            }
        }

        if let Some(max_age_hours) = parameters.get("max_age_hours") {
            if let Some(value) = max_age_hours.as_f64() {
                self.max_age_hours = value;
            }
        }

        if let Some(confidence_threshold) = parameters.get("confidence_threshold") {
            if let Some(value) = confidence_threshold.as_f64() {
                self.config.confidence_threshold = value;
            }
        }

        info!("Updated PumpFun sniping strategy parameters");
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn get_confidence(&self) -> f64 {
        // Calculate confidence based on recent performance
        if self.signals_generated > 0 {
            self.successful_trades as f64 / self.signals_generated as f64
        } else {
            0.8 // Default confidence for new strategy
        }
    }

    fn required_data_sources(&self) -> Vec<String> {
        self.config.required_sources.clone()
    }

    fn can_operate(&self, context: &StrategyContext) -> bool {
        // Check if we have required data sources
        let has_pumpfun = context.aggregated_data.sources_count > 0; // Simplified check
        
        // Check data confidence
        let sufficient_confidence = context.data_confidence() >= 0.6;
        
        // Check if it's a newly listed token (our specialty)
        let is_new_token = context.is_newly_listed();

        has_pumpfun && sufficient_confidence && is_new_token
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::Sniping
    }

    fn min_confidence_threshold(&self) -> f64 {
        self.config.confidence_threshold
    }

    /// Process real-time market events for PumpFun sniping
    #[instrument(skip(self, _context), fields(strategy = %self.name))]
    async fn on_market_event(&self, event: &MarketEvent, _context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.config.enabled {
            return Ok(None);
        }

        // Check cooldown
        if self.is_in_cooldown() {
            return Ok(None);
        }

        match event {
            MarketEvent::NewTokenListing {
                token_address,
                symbol,
                initial_price,
                initial_liquidity,
                creator,
                timestamp,
                ..
            } => {
                info!("🆕 New token detected: {} ({:?}) - Price: {:?}, Liquidity: {:?}",
                    token_address, symbol, initial_price, initial_liquidity);

                // Check if this is a PumpFun token (simplified check)
                if let Some(liquidity) = initial_liquidity {
                    if *liquidity >= self.min_liquidity && *liquidity <= 100_000.0 {
                        // This looks like a new PumpFun token
                        let strength = self.calculate_new_token_strength(
                            initial_price.unwrap_or(0.0),
                            *liquidity,
                            creator.as_deref(),
                        );

                        if strength >= self.config.confidence_threshold {
                            info!("🎯 PumpFun new token signal: {} with strength {:.2}",
                                token_address, strength);

                            return Ok(Some(self.generate_new_token_signal(
                                token_address,
                                symbol.as_deref().unwrap_or("UNKNOWN"),
                                initial_price.unwrap_or(0.0),
                                strength,
                                *timestamp,
                            )));
                        }
                    }
                }
            }

            MarketEvent::LiquidityUpdate {
                pool_address,
                token_a,
                token_b: _,
                liquidity_a,
                liquidity_b,
                price,
                timestamp,
                ..
            } => {
                // Check for significant liquidity increases (potential graduation)
                let total_liquidity_usd = liquidity_a + liquidity_b; // Simplified

                if total_liquidity_usd > 50_000.0 { // Potential graduation threshold
                    info!("🚀 Potential graduation detected: {} - Liquidity: ${:.2}",
                        pool_address, total_liquidity_usd);

                    // Check if we should generate a graduation signal
                    let strength = self.calculate_graduation_strength(total_liquidity_usd, *price);

                    if strength >= self.config.confidence_threshold {
                        return Ok(Some(self.generate_graduation_signal(
                            token_a,
                            *price,
                            strength,
                            *timestamp,
                        )));
                    }
                }
            }

            MarketEvent::NewTransaction {
                token_address,
                amount,
                price,
                transaction_type,
                timestamp,
                ..
            } => {
                // Look for large buy transactions that might indicate interest
                if matches!(transaction_type, TransactionType::Buy) && *amount > 1000.0 {
                    info!("🐋 Large buy detected: ${:.2} of {} at price {:?}",
                        amount, token_address, price);

                    // This could be a follow-the-whale signal
                    let strength = self.calculate_whale_follow_strength(*amount, price.unwrap_or(0.0));

                    if strength >= self.config.confidence_threshold * 0.8 { // Lower threshold for whale following
                        return Ok(Some(self.generate_whale_follow_signal(
                            token_address,
                            price.unwrap_or(0.0),
                            strength,
                            *timestamp,
                        )));
                    }
                }
            }

            _ => {
                // Other events not relevant for PumpFun sniping
                return Ok(None);
            }
        }

        Ok(None)
    }

    /// Check if strategy is interested in specific event types
    fn is_interested_in_event(&self, event: &MarketEvent) -> bool {
        matches!(event,
            MarketEvent::NewTokenListing { .. } |
            MarketEvent::LiquidityUpdate { .. } |
            MarketEvent::NewTransaction { transaction_type: TransactionType::Buy, .. }
        )
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
            symbol: "MEME/SOL".to_string(),
            price: 0.0001,
            volume: 10000.0,
            bid: Some(0.00009),
            ask: Some(0.00011),
            timestamp: Utc::now(),
            source: DataSource::Solana,
        };

        let aggregated_data = AggregatedMarketData {
            primary_data: market_data,
            secondary_data: vec![],
            sources_count: 1,
            confidence_score: 0.85,
            latency_ms: 200,
        };

        let portfolio = Portfolio {
            total_value: 10000.0,
            total_value_usd: Some(10000.0),
            available_balance: 5000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: vec![],
            daily_pnl: 0.0,
            max_drawdown: 0.0,
            updated_at: Utc::now(),
        };

        let market_conditions = MarketConditions {
            volatility: 0.25,
            volume_trend: VolumeTrend::Increasing,
            price_momentum: PriceMomentum::Bullish,
            liquidity_depth: 2000.0,
            market_cap: Some(100_000.0),
            age_hours: Some(6.0),
        };

        StrategyContext::new(aggregated_data, portfolio, market_conditions)
    }

    #[tokio::test]
    async fn test_pumpfun_sniping_strategy() {
        let strategy = PumpFunSnipingStrategy::new("test_pumpfun".to_string());
        let context = create_test_context();

        assert!(strategy.can_operate(&context));
        assert!(strategy.meets_basic_criteria(&context));

        let result = strategy.analyze(&context).await;
        assert!(result.is_ok());
        
        if let Some(signal) = result.unwrap() {
            assert_eq!(signal.signal_type, SignalType::Buy);
            assert!(signal.strength > 0.0);
            assert_eq!(signal.symbol, "MEME/SOL");
        }
    }

    #[test]
    fn test_bonding_curve_calculation() {
        let strategy = PumpFunSnipingStrategy::new("test".to_string());
        let context = create_test_context();
        
        let progress = strategy.calculate_bonding_curve_progress(&context);
        assert!(progress >= 0.0 && progress <= 1.0);
    }

    #[test]
    fn test_signal_strength_calculation() {
        let strategy = PumpFunSnipingStrategy::new("test".to_string());
        let context = create_test_context();
        
        let strength = strategy.calculate_signal_strength(&context);
        assert!(strength >= 0.0 && strength <= 1.0);
    }
}

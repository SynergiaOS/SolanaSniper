use async_trait::async_trait;
use crate::models::{MarketEvent, StrategySignal, SignalType, TradingResult};
use crate::strategy::{EnhancedStrategy, StrategyContext, StrategyType};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, debug};

/// MeteoraDLMMStrategy specializes in Dynamic Liquidity Market Making on Meteora
/// Focuses on bin analysis, liquidity concentration, and LP opportunities
pub struct MeteoraDLMMStrategy {
    name: String,
    config: MeteoraDLMMConfig,
    last_analysis_time: Option<Instant>,
    tracked_pools: HashMap<String, PoolState>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MeteoraDLMMConfig {
    pub min_apr_percentage: f64,          // Minimum APR for LP positions
    pub max_price_impact: f64,            // Maximum price impact for trades
    pub min_liquidity_depth: f64,         // Minimum total liquidity
    pub preferred_bin_range: (f64, f64),  // Preferred active bin range (e.g., 0.8-1.2 of current price)
    pub min_volume_24h: f64,              // Minimum 24h volume
    pub max_position_size: f64,           // Maximum position size in SOL
    pub bin_concentration_threshold: f64,  // Minimum liquidity concentration in active bins
    pub rebalance_threshold: f64,         // Price movement threshold for rebalancing
    pub cooldown_seconds: u64,            // Cooldown between actions
}

impl Default for MeteoraDLMMConfig {
    fn default() -> Self {
        Self {
            min_apr_percentage: 0.5,       // 50% minimum APR
            max_price_impact: 0.01,        // 1% max price impact
            min_liquidity_depth: 25000.0,  // $25k minimum liquidity
            preferred_bin_range: (0.95, 1.05), // 5% range around current price
            min_volume_24h: 100000.0,      // $100k minimum volume
            max_position_size: 5.0,        // 5 SOL max position
            bin_concentration_threshold: 0.6, // 60% of liquidity in active bins
            rebalance_threshold: 0.02,     // 2% price movement triggers rebalancing
            cooldown_seconds: 180,         // 3 minute cooldown
        }
    }
}

#[derive(Debug, Clone)]
struct PoolState {
    last_price: f64,
    last_update: Instant,
    active_bins: Vec<LiquidityBin>,
    total_liquidity: f64,
    volume_24h: f64,
    estimated_apr: f64,
}

#[derive(Debug, Clone)]
struct LiquidityBin {
    price_lower: f64,
    price_upper: f64,
    liquidity_amount: f64,
    utilization_rate: f64,
    fees_earned_24h: f64,
}

#[derive(Debug, Clone)]
struct DLMMOpportunity {
    opportunity_type: DLMMOpportunityType,
    estimated_apr: f64,
    required_capital: f64,
    risk_score: f64,
    bin_range: (f64, f64),
    expected_fees_24h: f64,
}

#[derive(Debug, Clone)]
enum DLMMOpportunityType {
    NewLPPosition,      // New liquidity provision opportunity
    RebalancePosition,  // Existing position needs rebalancing
    ArbitrageSwap,      // Arbitrage opportunity within DLMM
    BinConcentration,   // High concentration opportunity
}

impl MeteoraDLMMStrategy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: MeteoraDLMMConfig::default(),
            last_analysis_time: None,
            tracked_pools: HashMap::new(),
            enabled: true,
        }
    }

    pub fn with_config(name: String, config: MeteoraDLMMConfig) -> Self {
        Self {
            name,
            config,
            last_analysis_time: None,
            tracked_pools: HashMap::new(),
            enabled: true,
        }
    }

    /// Analyze DLMM bin distribution and liquidity concentration
    fn analyze_bin_distribution(&self, pool_state: &PoolState, current_price: f64) -> Option<DLMMOpportunity> {
        // Calculate active bin concentration
        let active_bins: Vec<&LiquidityBin> = pool_state.active_bins
            .iter()
            .filter(|bin| current_price >= bin.price_lower && current_price <= bin.price_upper)
            .collect();

        if active_bins.is_empty() {
            return None;
        }

        let active_liquidity: f64 = active_bins.iter().map(|bin| bin.liquidity_amount).sum();
        let concentration_ratio = active_liquidity / pool_state.total_liquidity;

        // Check if concentration is below threshold (opportunity for better positioning)
        if concentration_ratio < self.config.bin_concentration_threshold {
            let estimated_apr = self.calculate_estimated_apr(pool_state, current_price);
            
            if estimated_apr > self.config.min_apr_percentage {
                return Some(DLMMOpportunity {
                    opportunity_type: DLMMOpportunityType::BinConcentration,
                    estimated_apr,
                    required_capital: self.config.max_position_size * current_price,
                    risk_score: self.calculate_risk_score(pool_state, current_price),
                    bin_range: (
                        current_price * self.config.preferred_bin_range.0,
                        current_price * self.config.preferred_bin_range.1,
                    ),
                    expected_fees_24h: pool_state.volume_24h * 0.003, // Assume 0.3% fee
                });
            }
        }

        None
    }

    /// Calculate estimated APR for LP position
    fn calculate_estimated_apr(&self, pool_state: &PoolState, _current_price: f64) -> f64 {
        if pool_state.total_liquidity <= 0.0 {
            return 0.0;
        }

        // Estimate based on volume and fees
        let daily_fees = pool_state.volume_24h * 0.003; // 0.3% fee assumption
        let annual_fees = daily_fees * 365.0;
        let apr = annual_fees / pool_state.total_liquidity;

        // Adjust for bin concentration and utilization
        let concentration_bonus = if pool_state.active_bins.len() < 5 { 1.2 } else { 1.0 };
        
        apr * concentration_bonus
    }

    /// Calculate risk score for DLMM position
    fn calculate_risk_score(&self, pool_state: &PoolState, current_price: f64) -> f64 {
        let mut risk_score: f64 = 0.0;

        // Volume risk (lower volume = higher risk)
        if pool_state.volume_24h < self.config.min_volume_24h {
            risk_score += 0.3;
        }

        // Liquidity risk (lower liquidity = higher risk)
        if pool_state.total_liquidity < self.config.min_liquidity_depth {
            risk_score += 0.2;
        }

        // Price volatility risk (based on bin spread)
        let price_range = pool_state.active_bins.iter()
            .map(|bin| (bin.price_upper - bin.price_lower) / current_price)
            .fold(0.0, |acc, range| acc + range);
        
        if price_range > 0.1 { // High volatility
            risk_score += 0.3;
        }

        // Concentration risk (too concentrated = higher impermanent loss risk)
        let active_bins_count = pool_state.active_bins.len();
        if active_bins_count < 3 {
            risk_score += 0.2;
        }

        risk_score.min(1.0)
    }

    /// Detect rebalancing opportunities
    fn detect_rebalancing_opportunity(&self, pool_state: &PoolState, current_price: f64) -> Option<DLMMOpportunity> {
        let price_change = (current_price - pool_state.last_price) / pool_state.last_price;
        
        if price_change.abs() > self.config.rebalance_threshold {
            let estimated_apr = self.calculate_estimated_apr(pool_state, current_price);
            
            if estimated_apr > self.config.min_apr_percentage {
                return Some(DLMMOpportunity {
                    opportunity_type: DLMMOpportunityType::RebalancePosition,
                    estimated_apr,
                    required_capital: self.config.max_position_size * current_price * 0.5, // Partial rebalance
                    risk_score: self.calculate_risk_score(pool_state, current_price),
                    bin_range: (
                        current_price * self.config.preferred_bin_range.0,
                        current_price * self.config.preferred_bin_range.1,
                    ),
                    expected_fees_24h: pool_state.volume_24h * 0.003 * 0.5, // Partial position
                });
            }
        }

        None
    }

    /// Generate trading signal for DLMM opportunity
    fn generate_dlmm_signal(
        &self,
        opportunity: &DLMMOpportunity,
        context: &StrategyContext,
    ) -> StrategySignal {
        let signal_strength = (opportunity.estimated_apr / self.config.min_apr_percentage)
            .min(1.0)
            .max(0.0);

        let confidence = match opportunity.opportunity_type {
            DLMMOpportunityType::NewLPPosition => 0.8,
            DLMMOpportunityType::BinConcentration => 0.9,
            DLMMOpportunityType::RebalancePosition => 0.7,
            DLMMOpportunityType::ArbitrageSwap => 0.85,
        };

        StrategySignal {
            strategy: self.name.clone(),
            signal_type: SignalType::Buy, // DLMM positions start with providing liquidity
            symbol: context.aggregated_data.primary_data.symbol.clone(),
            price: context.aggregated_data.primary_data.price,
            strength: signal_strength,
            size: self.config.max_position_size,
            metadata: serde_json::json!({
                "strategy_type": "meteora_dlmm",
                "opportunity_type": format!("{:?}", opportunity.opportunity_type),
                "estimated_apr": format!("{:.2}", opportunity.estimated_apr * 100.0),
                "required_capital": opportunity.required_capital,
                "risk_score": format!("{:.3}", opportunity.risk_score),
                "bin_range_lower": opportunity.bin_range.0,
                "bin_range_upper": opportunity.bin_range.1,
                "expected_fees_24h": opportunity.expected_fees_24h,
                "confidence": confidence,
            }),
            timestamp: Utc::now(),
        }
    }

    /// Create mock pool state for testing (in real implementation, this would fetch from Meteora API)
    fn create_mock_pool_state(&self, context: &StrategyContext) -> PoolState {
        let current_price = context.aggregated_data.primary_data.price;
        
        // Create mock bins around current price
        let mut active_bins = Vec::new();
        for i in -2..=2 {
            let price_offset = i as f64 * 0.01; // 1% increments
            active_bins.push(LiquidityBin {
                price_lower: current_price * (1.0 + price_offset - 0.005),
                price_upper: current_price * (1.0 + price_offset + 0.005),
                liquidity_amount: context.market_conditions.liquidity_depth / 5.0,
                utilization_rate: if i == 0 { 0.8 } else { 0.3 }, // Current price bin has higher utilization
                fees_earned_24h: context.aggregated_data.primary_data.volume * 0.003 / 5.0,
            });
        }

        PoolState {
            last_price: current_price * 0.99, // Simulate 1% price movement
            last_update: Instant::now(),
            active_bins,
            total_liquidity: context.market_conditions.liquidity_depth,
            volume_24h: context.aggregated_data.primary_data.volume,
            estimated_apr: 0.0, // Will be calculated
        }
    }
}

#[async_trait]
impl EnhancedStrategy for MeteoraDLMMStrategy {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()> {
        if let Some(min_apr) = parameters.get("min_apr_percentage") {
            if let Some(value) = min_apr.as_f64() {
                self.config.min_apr_percentage = value;
            }
        }

        if let Some(max_position) = parameters.get("max_position_size") {
            if let Some(value) = max_position.as_f64() {
                self.config.max_position_size = value;
            }
        }

        if let Some(enabled) = parameters.get("enabled") {
            if let Some(value) = enabled.as_bool() {
                self.enabled = value;
            }
        }

        Ok(())
    }

    fn get_confidence(&self) -> f64 {
        0.8 // High confidence for DLMM when opportunities are detected
    }

    fn required_data_sources(&self) -> Vec<String> {
        vec![
            "meteora".to_string(),
            "jupiter".to_string(),
            "raydium".to_string(),
        ]
    }

    fn can_operate(&self, context: &StrategyContext) -> bool {
        self.enabled
            && context.aggregated_data.primary_data.volume > self.config.min_volume_24h
            && context.market_conditions.liquidity_depth > self.config.min_liquidity_depth
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::LiquidityProvision
    }

    fn min_confidence_threshold(&self) -> f64 {
        0.7
    }

    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }

        // Check basic requirements
        if context.aggregated_data.primary_data.volume < self.config.min_volume_24h {
            debug!("MeteoraDLMM: Volume too low for {}", context.aggregated_data.primary_data.symbol);
            return Ok(None);
        }

        if context.market_conditions.liquidity_depth < self.config.min_liquidity_depth {
            debug!("MeteoraDLMM: Liquidity too low for {}", context.aggregated_data.primary_data.symbol);
            return Ok(None);
        }

        // Create or update pool state (in real implementation, fetch from Meteora)
        let pool_state = self.create_mock_pool_state(context);
        let current_price = context.aggregated_data.primary_data.price;

        // Analyze for opportunities
        let mut best_opportunity = None;

        // Check for bin concentration opportunities
        if let Some(opportunity) = self.analyze_bin_distribution(&pool_state, current_price) {
            best_opportunity = Some(opportunity);
        }

        // Check for rebalancing opportunities
        if let Some(opportunity) = self.detect_rebalancing_opportunity(&pool_state, current_price) {
            if best_opportunity.is_none() || opportunity.estimated_apr > best_opportunity.as_ref().unwrap().estimated_apr {
                best_opportunity = Some(opportunity);
            }
        }

        if let Some(opportunity) = best_opportunity {
            info!(
                "💧 Meteora DLMM opportunity found for {}: {:.2}% APR ({:?})",
                context.aggregated_data.primary_data.symbol,
                opportunity.estimated_apr * 100.0,
                opportunity.opportunity_type
            );

            let signal = self.generate_dlmm_signal(&opportunity, context);
            return Ok(Some(signal));
        }

        Ok(None)
    }

    async fn on_market_event(&self, event: &MarketEvent, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        match event {
            MarketEvent::LiquidityUpdate { .. } => {
                // Liquidity updates are crucial for DLMM analysis
                self.analyze(context).await
            }
            MarketEvent::PriceUpdate { .. } => {
                // Price updates might trigger rebalancing
                self.analyze(context).await
            }
            _ => Ok(None),
        }
    }

    fn is_interested_in_event(&self, event: &MarketEvent) -> bool {
        matches!(event, 
            MarketEvent::LiquidityUpdate { .. } | 
            MarketEvent::PriceUpdate { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meteora_dlmm_strategy_creation() {
        let strategy = MeteoraDLMMStrategy::new("test_meteora".to_string());
        assert_eq!(strategy.get_name(), "test_meteora");
        assert_eq!(strategy.get_strategy_type(), StrategyType::LiquidityProvision);
        assert!(strategy.is_enabled());
    }

    #[test]
    fn test_apr_calculation() {
        let strategy = MeteoraDLMMStrategy::new("test".to_string());
        
        let pool_state = PoolState {
            last_price: 100.0,
            last_update: Instant::now(),
            active_bins: vec![],
            total_liquidity: 100000.0,
            volume_24h: 50000.0,
            estimated_apr: 0.0,
        };

        let apr = strategy.calculate_estimated_apr(&pool_state, 100.0);
        assert!(apr > 0.0);
        assert!(apr < 10.0); // Reasonable APR range
    }
}

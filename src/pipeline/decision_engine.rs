/*!
🤖 Decision Engine - AI-powered trading decision making

This module implements the core decision-making logic that determines
what actions to take based on validated opportunities.
*/

use crate::pipeline::opportunity::{ValidatedOpportunity, StrategyType, RiskLevel};
use crate::models::TradingResult;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Trading decision with specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    /// Unique decision ID
    pub id: String,
    /// Related opportunity ID
    pub opportunity_id: String,
    /// Decision type
    pub decision_type: DecisionType,
    /// Recommended position size in SOL
    pub position_size_sol: f64,
    /// Maximum slippage tolerance (basis points)
    pub max_slippage_bps: u16,
    /// Stop loss percentage (if applicable)
    pub stop_loss_percentage: Option<f64>,
    /// Take profit percentage (if applicable)
    pub take_profit_percentage: Option<f64>,
    /// Priority level (1-10, 10 being highest)
    pub priority: u8,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Reasoning for the decision
    pub reasoning: String,
}

/// Types of trading decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionType {
    /// Buy token immediately
    BuyToken {
        token_address: String,
        target_amount_sol: f64,
    },
    /// Provide liquidity to pool
    ProvideLiquidity {
        pool_address: String,
        amount_sol: f64,
        duration_hours: u32,
    },
    /// Execute arbitrage
    Arbitrage {
        buy_pool: String,
        sell_pool: String,
        amount_sol: f64,
    },
    /// Monitor opportunity
    Monitor {
        check_interval_minutes: u32,
        max_monitoring_hours: u32,
    },
    /// No action - avoid this opportunity
    NoAction {
        reason: String,
    },
}

/// Decision engine configuration
#[derive(Debug, Clone)]
pub struct DecisionEngineConfig {
    /// Maximum position size per trade (SOL)
    pub max_position_size_sol: f64,
    /// Minimum position size per trade (SOL)
    pub min_position_size_sol: f64,
    /// Risk tolerance (0.0 to 1.0)
    pub risk_tolerance: f64,
    /// Minimum confidence required for execution
    pub min_confidence: f64,
    /// Maximum number of concurrent positions
    pub max_concurrent_positions: u32,
    /// Available balance for trading (SOL)
    pub available_balance_sol: f64,
}

impl Default for DecisionEngineConfig {
    fn default() -> Self {
        Self {
            max_position_size_sol: 1.0,
            min_position_size_sol: 0.1,
            risk_tolerance: 0.6,
            min_confidence: 0.7,
            max_concurrent_positions: 5,
            available_balance_sol: 10.0,
        }
    }
}

/// Core decision engine
pub struct DecisionEngine {
    config: DecisionEngineConfig,
    active_positions: u32,
}

impl DecisionEngine {
    /// Create new decision engine with default config
    pub fn new() -> Self {
        Self {
            config: DecisionEngineConfig::default(),
            active_positions: 0,
        }
    }

    /// Create new decision engine with custom config
    pub fn with_config(config: DecisionEngineConfig) -> Self {
        Self {
            config,
            active_positions: 0,
        }
    }

    /// Analyze opportunity and generate trading decision
    pub async fn analyze_opportunity(&self, opportunity: &ValidatedOpportunity) -> TradingResult<TradingDecision> {
        info!("🤖 Analyzing opportunity: {}", opportunity.candidate.name);

        // Check if we can take more positions
        if self.active_positions >= self.config.max_concurrent_positions {
            return Ok(self.create_no_action_decision(
                opportunity,
                "Maximum concurrent positions reached".to_string(),
            ));
        }

        // Check minimum confidence
        if opportunity.combined_score < self.config.min_confidence * 10.0 {
            return Ok(self.create_no_action_decision(
                opportunity,
                format!("Confidence too low: {:.2}", opportunity.combined_score),
            ));
        }

        // Risk-based filtering
        let decision = match opportunity.risk_level {
            RiskLevel::VeryHigh => {
                self.create_no_action_decision(opportunity, "Risk level too high".to_string())
            }
            RiskLevel::High => {
                if self.config.risk_tolerance < 0.8 {
                    self.create_no_action_decision(opportunity, "High risk exceeds tolerance".to_string())
                } else {
                    self.create_strategy_decision(opportunity).await?
                }
            }
            _ => self.create_strategy_decision(opportunity).await?,
        };

        info!("🎯 Decision: {:?} for {}", decision.decision_type, opportunity.candidate.name);
        Ok(decision)
    }

    /// Create strategy-specific decision
    async fn create_strategy_decision(&self, opportunity: &ValidatedOpportunity) -> TradingResult<TradingDecision> {
        let position_size = self.calculate_position_size(opportunity);
        
        let decision_type = match opportunity.recommended_strategy {
            StrategyType::TokenSniper => {
                DecisionType::BuyToken {
                    token_address: opportunity.candidate.address.clone(),
                    target_amount_sol: position_size,
                }
            }
            StrategyType::LiquidityProvider => {
                DecisionType::ProvideLiquidity {
                    pool_address: opportunity.candidate.address.clone(),
                    amount_sol: position_size,
                    duration_hours: self.calculate_lp_duration(opportunity),
                }
            }
            StrategyType::Arbitrage => {
                DecisionType::Arbitrage {
                    buy_pool: opportunity.candidate.address.clone(),
                    sell_pool: "TBD".to_string(), // Would need additional logic
                    amount_sol: position_size,
                }
            }
            StrategyType::Monitor => {
                DecisionType::Monitor {
                    check_interval_minutes: 5,
                    max_monitoring_hours: 24,
                }
            }
            StrategyType::Avoid => {
                return Ok(self.create_no_action_decision(
                    opportunity,
                    "Strategy recommends avoiding".to_string(),
                ));
            }
        };

        Ok(TradingDecision {
            id: format!("decision_{}_{}", opportunity.id, chrono::Utc::now().timestamp()),
            opportunity_id: opportunity.id.clone(),
            decision_type,
            position_size_sol: position_size,
            max_slippage_bps: self.calculate_slippage_tolerance(opportunity),
            stop_loss_percentage: self.calculate_stop_loss(opportunity),
            take_profit_percentage: self.calculate_take_profit(opportunity),
            priority: self.calculate_priority(opportunity),
            confidence: opportunity.combined_score / 10.0,
            reasoning: self.generate_reasoning(opportunity),
        })
    }

    /// Create no-action decision
    fn create_no_action_decision(&self, opportunity: &ValidatedOpportunity, reason: String) -> TradingDecision {
        TradingDecision {
            id: format!("no_action_{}_{}", opportunity.id, chrono::Utc::now().timestamp()),
            opportunity_id: opportunity.id.clone(),
            decision_type: DecisionType::NoAction { reason: reason.clone() },
            position_size_sol: 0.0,
            max_slippage_bps: 0,
            stop_loss_percentage: None,
            take_profit_percentage: None,
            priority: 1,
            confidence: 0.0,
            reasoning: reason,
        }
    }

    /// Calculate optimal position size based on risk and opportunity
    fn calculate_position_size(&self, opportunity: &ValidatedOpportunity) -> f64 {
        let base_size = self.config.available_balance_sol * 0.1; // 10% of available balance
        
        // Adjust based on confidence
        let confidence_multiplier = opportunity.combined_score / 10.0;
        
        // Adjust based on risk
        let risk_multiplier = match opportunity.risk_level {
            RiskLevel::VeryLow => 1.5,
            RiskLevel::Low => 1.2,
            RiskLevel::Medium => 1.0,
            RiskLevel::High => 0.7,
            RiskLevel::VeryHigh => 0.3,
        };

        let calculated_size = base_size * confidence_multiplier * risk_multiplier;
        
        // Clamp to min/max limits
        calculated_size
            .max(self.config.min_position_size_sol)
            .min(self.config.max_position_size_sol)
            .min(self.config.available_balance_sol * 0.2) // Never more than 20% of balance
    }

    /// Calculate slippage tolerance based on liquidity
    fn calculate_slippage_tolerance(&self, opportunity: &ValidatedOpportunity) -> u16 {
        if opportunity.candidate.liquidity_usd > 1000000.0 {
            50 // 0.5% for high liquidity
        } else if opportunity.candidate.liquidity_usd > 100000.0 {
            100 // 1% for medium liquidity
        } else {
            300 // 3% for low liquidity
        }
    }

    /// Calculate stop loss percentage
    fn calculate_stop_loss(&self, opportunity: &ValidatedOpportunity) -> Option<f64> {
        match opportunity.recommended_strategy {
            StrategyType::TokenSniper => {
                match opportunity.risk_level {
                    RiskLevel::VeryLow | RiskLevel::Low => Some(15.0), // 15% stop loss
                    RiskLevel::Medium => Some(20.0), // 20% stop loss
                    RiskLevel::High => Some(25.0), // 25% stop loss
                    RiskLevel::VeryHigh => Some(30.0), // 30% stop loss
                }
            }
            _ => None, // No stop loss for LP or other strategies
        }
    }

    /// Calculate take profit percentage
    fn calculate_take_profit(&self, opportunity: &ValidatedOpportunity) -> Option<f64> {
        match opportunity.recommended_strategy {
            StrategyType::TokenSniper => {
                if opportunity.combined_score > 8.0 {
                    Some(100.0) // 100% take profit for high confidence
                } else if opportunity.combined_score > 6.0 {
                    Some(50.0) // 50% take profit for medium confidence
                } else {
                    Some(25.0) // 25% take profit for lower confidence
                }
            }
            _ => None,
        }
    }

    /// Calculate priority level
    fn calculate_priority(&self, opportunity: &ValidatedOpportunity) -> u8 {
        let base_priority = (opportunity.combined_score / 10.0 * 8.0) as u8 + 1;
        
        // Boost priority for time-sensitive opportunities
        if opportunity.candidate.volume_24h > 5000000.0 {
            (base_priority + 2).min(10)
        } else {
            base_priority
        }
    }

    /// Calculate LP duration in hours
    fn calculate_lp_duration(&self, opportunity: &ValidatedOpportunity) -> u32 {
        if opportunity.candidate.apr >= 50.0 {
            4 // Short duration for very high APR
        } else if opportunity.candidate.apr >= 25.0 {
            12 // Medium duration for high APR
        } else {
            24 // Longer duration for moderate APR
        }
    }

    /// Generate human-readable reasoning
    fn generate_reasoning(&self, opportunity: &ValidatedOpportunity) -> String {
        format!(
            "Score: {:.1}/10, Strategy: {:?}, Risk: {:?}, Liquidity: ${:.0}, Volume: ${:.0}, APR: {:.1}%",
            opportunity.combined_score,
            opportunity.recommended_strategy,
            opportunity.risk_level,
            opportunity.candidate.liquidity_usd,
            opportunity.candidate.volume_24h,
            opportunity.candidate.apr
        )
    }

    /// Update active positions count
    pub fn set_active_positions(&mut self, count: u32) {
        self.active_positions = count;
    }

    /// Get current configuration
    pub fn config(&self) -> &DecisionEngineConfig {
        &self.config
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::opportunity::{ValidatedOpportunity, SentimentReport};
    use crate::data_fetcher::soul_meteor_scanner::HotCandidate;

    #[tokio::test]
    async fn test_decision_engine() {
        let engine = DecisionEngine::new();
        
        let candidate = HotCandidate {
            name: "TEST-SOL".to_string(),
            address: "test123".to_string(),
            liquidity_usd: 500000.0,
            volume_24h: 2000000.0,
            fees_24h: 5000.0,
            fee_tvl_ratio_24h: 1.0,
            apr: 30.0,
            apy: 300.0,
            opportunity_score: 3.8,
            mint_x: "mint1".to_string(),
            mint_y: "mint2".to_string(),
            current_price: 1.0,
            is_blacklisted: false,
            hide: false,
        };

        let sentiment = SentimentReport {
            aggregated_score: 0.7,
            confidence: 0.9,
            patterns: vec!["BULLISH".to_string()],
            sources_count: 10,
            textual_data: None,
            analyzed_at: chrono::Utc::now(),
        };

        let opportunity = ValidatedOpportunity::new(candidate, sentiment);
        let decision = engine.analyze_opportunity(&opportunity).await.unwrap();
        
        assert!(matches!(decision.decision_type, DecisionType::ProvideLiquidity { .. }));
        assert!(decision.confidence > 0.7);
        assert!(decision.position_size_sol > 0.0);
    }
}

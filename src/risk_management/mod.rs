use crate::models::{Order, Portfolio, StrategySignal, TradingError, TradingResult, AIRecommendation};
use crate::utils::config::RiskManagementConfig;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct RiskManager {
    config: RiskManagementConfig,
    daily_pnl: f64,
    max_drawdown_reached: f64,
    emergency_stop_triggered: bool,
    position_limits: PositionLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLimits {
    pub max_position_size: f64,
    pub max_portfolio_exposure: f64,
    pub max_correlation_exposure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub approved: bool,
    pub risk_score: f64,
    pub warnings: Vec<String>,
    pub suggested_size: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

impl RiskManager {
    pub fn new(config: RiskManagementConfig) -> Self {
        Self {
            config,
            daily_pnl: 0.0,
            max_drawdown_reached: 0.0,
            emergency_stop_triggered: false,
            position_limits: PositionLimits {
                max_position_size: 10000.0,
                max_portfolio_exposure: 0.8,
                max_correlation_exposure: 0.3,
            },
        }
    }

    pub async fn assess_signal(&self, signal: &StrategySignal, portfolio: &Portfolio) -> TradingResult<RiskAssessment> {
        let mut assessment = RiskAssessment {
            approved: true,
            risk_score: 0.0,
            warnings: Vec::new(),
            suggested_size: Some(signal.size),
            stop_loss: None,
            take_profit: None,
        };

        // Check emergency stop
        if self.emergency_stop_triggered {
            assessment.approved = false;
            assessment.warnings.push("Emergency stop is active".to_string());
            return Ok(assessment);
        }

        // Check daily loss limit
        if self.daily_pnl <= -self.config.max_daily_loss {
            assessment.approved = false;
            assessment.warnings.push(format!(
                "Daily loss limit exceeded: {} <= -{}",
                self.daily_pnl, self.config.max_daily_loss
            ));
            return Ok(assessment);
        }

        // Check maximum drawdown
        if self.max_drawdown_reached >= self.config.max_drawdown {
            assessment.approved = false;
            assessment.warnings.push(format!(
                "Maximum drawdown exceeded: {} >= {}",
                self.max_drawdown_reached, self.config.max_drawdown
            ));
            return Ok(assessment);
        }

        // Check global exposure
        let current_exposure = self.calculate_portfolio_exposure(portfolio);
        if current_exposure >= self.config.global_max_exposure {
            assessment.approved = false;
            assessment.warnings.push(format!(
                "Global exposure limit exceeded: {} >= {}",
                current_exposure, self.config.global_max_exposure
            ));
            return Ok(assessment);
        }

        // Calculate position size based on risk management method
        let suggested_size = self.calculate_position_size(signal, portfolio)?;
        assessment.suggested_size = Some(suggested_size);

        // Calculate stop loss and take profit
        let (stop_loss, take_profit) = self.calculate_risk_levels(signal);
        assessment.stop_loss = stop_loss;
        assessment.take_profit = take_profit;

        // Calculate risk score (0.0 = low risk, 1.0 = high risk)
        assessment.risk_score = self.calculate_risk_score(signal, portfolio);

        // Add warnings based on risk score
        if assessment.risk_score > 0.8 {
            assessment.warnings.push("High risk signal detected".to_string());
        } else if assessment.risk_score > 0.6 {
            assessment.warnings.push("Medium risk signal detected".to_string());
        }

        // Final approval check
        if assessment.risk_score > 0.9 {
            assessment.approved = false;
            assessment.warnings.push("Risk score too high for execution".to_string());
        }

        info!(
            "Risk assessment for {}: approved={}, risk_score={:.2}, size={}",
            signal.symbol, assessment.approved, assessment.risk_score, suggested_size
        );

        Ok(assessment)
    }

    /// Enhanced risk assessment with AI recommendation integration
    /// Combines traditional risk metrics with AI-driven insights for superior risk management
    pub async fn assess_signal_with_ai(
        &self,
        signal: &StrategySignal,
        portfolio: &Portfolio,
        ai_recommendation: Option<&AIRecommendation>
    ) -> TradingResult<RiskAssessment> {
        // Start with base risk assessment
        let mut assessment = self.assess_signal(signal, portfolio).await?;

        // Integrate AI recommendation if available
        if let Some(ai_rec) = ai_recommendation {
            info!("🧠 Integrating AI recommendation into risk assessment");

            // AI Risk Score Integration (weighted combination)
            let ai_risk_weight = 0.4; // 40% weight to AI assessment
            let base_risk_weight = 0.6; // 60% weight to traditional assessment

            let combined_risk_score = (assessment.risk_score * base_risk_weight) +
                                    (ai_rec.risk_score * ai_risk_weight);

            assessment.risk_score = combined_risk_score.min(1.0);

            // AI Confidence Impact on Position Sizing
            if ai_rec.confidence < 0.5 {
                // Low AI confidence -> reduce position size
                if let Some(size) = assessment.suggested_size {
                    let confidence_factor = ai_rec.confidence.max(0.2); // Minimum 20% size
                    assessment.suggested_size = Some(size * confidence_factor);
                    assessment.warnings.push(format!(
                        "Position size reduced due to low AI confidence: {:.2}",
                        ai_rec.confidence
                    ));
                }
            }

            // AI Action Override Logic
            match ai_rec.action.as_str() {
                "REJECT" => {
                    assessment.approved = false;
                    assessment.warnings.push(format!(
                        "Trade rejected by AI: {}",
                        ai_rec.rationale
                    ));
                    warn!("🚫 AI rejected trade: {}", ai_rec.rationale);
                }
                "HOLD" if ai_rec.confidence > 0.8 => {
                    // High-confidence HOLD from AI should be respected
                    assessment.approved = false;
                    assessment.warnings.push(format!(
                        "High-confidence AI HOLD recommendation: {}",
                        ai_rec.rationale
                    ));
                    info!("⏸️ AI recommends HOLD with high confidence");
                }
                _ => {
                    // BUY/SELL actions are processed normally
                    info!("✅ AI supports {} action", ai_rec.action);
                }
            }

            // Dynamic Stop-Loss/Take-Profit from AI
            if let Some(ai_stop_loss) = ai_rec.stop_loss_price {
                assessment.stop_loss = Some(ai_stop_loss);
                info!("🛡️ AI-suggested stop-loss: {}", ai_stop_loss);
            }

            if let Some(ai_target) = ai_rec.target_price {
                assessment.take_profit = Some(ai_target);
                info!("🎯 AI-suggested target price: {}", ai_target);
            }

            // Enhanced Risk Warnings
            if assessment.risk_score > 0.7 {
                assessment.warnings.push(format!(
                    "High combined risk score (AI + Traditional): {:.2}",
                    assessment.risk_score
                ));
            }

            // Final AI-Enhanced Approval Check
            if assessment.risk_score > 0.85 {
                assessment.approved = false;
                assessment.warnings.push(
                    "Risk score too high after AI integration".to_string()
                );
                warn!("🚨 Trade blocked due to high AI-enhanced risk score: {:.2}", assessment.risk_score);
            }

            info!(
                "🧠 AI-Enhanced Risk Assessment: approved={}, risk_score={:.2}, ai_confidence={:.2}",
                assessment.approved, assessment.risk_score, ai_rec.confidence
            );
        } else {
            warn!("⚠️ No AI recommendation available, using traditional risk assessment only");
        }

        Ok(assessment)
    }

    pub async fn validate_order(&self, order: &Order, portfolio: &Portfolio) -> TradingResult<bool> {
        // Pre-execution validation
        
        // Check if we have sufficient balance
        let required_balance = order.size * order.price.unwrap_or(0.0);
        if required_balance > portfolio.available_balance {
            return Err(TradingError::InsufficientBalance {
                required: required_balance,
                available: portfolio.available_balance,
            });
        }

        // Check position limits
        if order.size > self.position_limits.max_position_size {
            return Err(TradingError::RiskLimitExceeded(format!(
                "Order size {} exceeds maximum position size {}",
                order.size, self.position_limits.max_position_size
            )));
        }

        // Check circuit breaker
        if self.should_trigger_circuit_breaker(portfolio) {
            warn!("Circuit breaker triggered - blocking order execution");
            return Ok(false);
        }

        Ok(true)
    }

    pub fn update_daily_pnl(&mut self, pnl_change: f64) {
        self.daily_pnl += pnl_change;
        
        if self.daily_pnl <= -self.config.max_daily_loss {
            warn!("Daily loss limit reached: {}", self.daily_pnl);
        }
    }

    pub fn update_drawdown(&mut self, current_drawdown: f64) {
        if current_drawdown > self.max_drawdown_reached {
            self.max_drawdown_reached = current_drawdown;
            
            if current_drawdown >= self.config.max_drawdown {
                error!("Maximum drawdown exceeded: {}", current_drawdown);
            }
        }
    }

    pub fn trigger_emergency_stop(&mut self, reason: &str) {
        self.emergency_stop_triggered = true;
        error!("Emergency stop triggered: {}", reason);
    }

    pub fn reset_emergency_stop(&mut self) {
        self.emergency_stop_triggered = false;
        info!("Emergency stop reset");
    }

    pub fn is_emergency_stop_active(&self) -> bool {
        self.emergency_stop_triggered
    }

    fn calculate_portfolio_exposure(&self, portfolio: &Portfolio) -> f64 {
        portfolio.positions.iter()
            .map(|pos| pos.size * pos.current_price)
            .sum()
    }

    fn calculate_position_size(&self, signal: &StrategySignal, portfolio: &Portfolio) -> TradingResult<f64> {
        match self.config.position_sizing_method.as_str() {
            "fixed" => Ok(signal.size),
            "percentage" => {
                let percentage = 0.02; // 2% of portfolio
                Ok(portfolio.total_value * percentage / signal.price)
            }
            "volatility_adjusted" => {
                // Simplified volatility adjustment
                let base_size = portfolio.total_value * 0.02 / signal.price;
                let volatility_factor = 1.0 / signal.strength; // Higher strength = lower volatility
                Ok(base_size * volatility_factor)
            }
            _ => Ok(signal.size),
        }
    }

    fn calculate_risk_levels(&self, signal: &StrategySignal) -> (Option<f64>, Option<f64>) {
        let stop_loss_pct = 0.02; // 2% stop loss
        let take_profit_pct = 0.04; // 4% take profit

        let stop_loss = match signal.signal_type {
            crate::models::SignalType::Buy => Some(signal.price * (1.0 - stop_loss_pct)),
            crate::models::SignalType::Sell => Some(signal.price * (1.0 + stop_loss_pct)),
            _ => None,
        };

        let take_profit = match signal.signal_type {
            crate::models::SignalType::Buy => Some(signal.price * (1.0 + take_profit_pct)),
            crate::models::SignalType::Sell => Some(signal.price * (1.0 - take_profit_pct)),
            _ => None,
        };

        (stop_loss, take_profit)
    }

    fn calculate_risk_score(&self, signal: &StrategySignal, portfolio: &Portfolio) -> f64 {
        let mut risk_score = 0.0;

        // Factor 1: Signal strength (inverse relationship)
        risk_score += (1.0 - signal.strength) * 0.3;

        // Factor 2: Portfolio concentration
        let exposure = self.calculate_portfolio_exposure(portfolio);
        let concentration = exposure / portfolio.total_value;
        risk_score += concentration * 0.3;

        // Factor 3: Current drawdown
        let drawdown_factor = self.max_drawdown_reached / self.config.max_drawdown;
        risk_score += drawdown_factor * 0.2;

        // Factor 4: Daily P&L impact
        let daily_pnl_factor = (-self.daily_pnl / self.config.max_daily_loss).max(0.0);
        risk_score += daily_pnl_factor * 0.2;

        risk_score.min(1.0)
    }

    fn should_trigger_circuit_breaker(&self, portfolio: &Portfolio) -> bool {
        if !self.config.emergency_stop_enabled {
            return false;
        }

        // Trigger if daily loss exceeds circuit breaker threshold
        let daily_loss_pct = -self.daily_pnl / portfolio.total_value;
        daily_loss_pct >= self.config.circuit_breaker_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SignalType, StrategySignal};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_config() -> RiskManagementConfig {
        RiskManagementConfig {
            global_max_exposure: 10000.0,
            max_daily_loss: 1000.0,
            max_drawdown: 0.2,
            position_sizing_method: "percentage".to_string(),
            emergency_stop_enabled: true,
            circuit_breaker_threshold: 0.05,
        }
    }

    fn create_test_portfolio() -> Portfolio {
        Portfolio {
            total_value: 10000.0,
            available_balance: 5000.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: Vec::new(),
            daily_pnl: 0.0,
            max_drawdown: 0.0,
            total_value_usd: Some(10000.0),
            updated_at: Utc::now(),
        }
    }

    fn create_test_signal() -> StrategySignal {
        StrategySignal {
            strategy: "test_strategy".to_string(),
            symbol: "BTCUSDT".to_string(),
            signal_type: SignalType::Buy,
            strength: 0.8,
            price: 50000.0,
            size: 100.0,
            metadata: serde_json::json!({}),
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_risk_assessment() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let portfolio = create_test_portfolio();
        let signal = create_test_signal();

        let assessment = risk_manager.assess_signal(&signal, &portfolio).await;
        assert!(assessment.is_ok());
        
        let assessment = assessment.unwrap();
        assert!(assessment.approved);
        assert!(assessment.risk_score >= 0.0 && assessment.risk_score <= 1.0);
    }

    #[tokio::test]
    async fn test_ai_enhanced_risk_assessment() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let portfolio = create_test_portfolio();
        let signal = create_test_signal();

        // Test with AI recommendation
        let ai_recommendation = AIRecommendation {
            action: "BUY".to_string(),
            confidence: 0.8,
            rationale: "Strong technical indicators and positive sentiment".to_string(),
            risk_score: 0.3,
            target_price: Some(52000.0),
            stop_loss_price: Some(48000.0),
            strategy_parameters: HashMap::new(),
        };

        let assessment = risk_manager.assess_signal_with_ai(&signal, &portfolio, Some(&ai_recommendation)).await;
        assert!(assessment.is_ok());

        let assessment = assessment.unwrap();
        assert!(assessment.approved);
        assert!(assessment.risk_score >= 0.0 && assessment.risk_score <= 1.0);
        assert!(assessment.take_profit.is_some());
        assert!(assessment.stop_loss.is_some());
    }

    #[tokio::test]
    async fn test_ai_rejection_override() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let portfolio = create_test_portfolio();
        let signal = create_test_signal();

        // Test AI rejection
        let ai_recommendation = AIRecommendation {
            action: "REJECT".to_string(),
            confidence: 0.9,
            rationale: "High market volatility detected".to_string(),
            risk_score: 0.9,
            target_price: None,
            stop_loss_price: None,
            strategy_parameters: HashMap::new(),
        };

        let assessment = risk_manager.assess_signal_with_ai(&signal, &portfolio, Some(&ai_recommendation)).await;
        assert!(assessment.is_ok());

        let assessment = assessment.unwrap();
        assert!(!assessment.approved); // Should be rejected by AI
        assert!(!assessment.warnings.is_empty());
    }

    #[test]
    fn test_emergency_stop() {
        let config = create_test_config();
        let mut risk_manager = RiskManager::new(config);

        assert!(!risk_manager.is_emergency_stop_active());

        risk_manager.trigger_emergency_stop("Test emergency");
        assert!(risk_manager.is_emergency_stop_active());

        risk_manager.reset_emergency_stop();
        assert!(!risk_manager.is_emergency_stop_active());
    }
}

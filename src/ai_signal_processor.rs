use crate::models::{
    StrategySignal, AIEnhancedSignal, AIRecommendation, TradingResult,
    TokenInfo, AggregatedAnalytics, MarketConditions, PortfolioState
};
use crate::ai_decision_engine::AIDecisionEngine;
use serde_json::json;
use chrono::Utc;
use tracing::{info, warn, instrument};
use std::sync::Arc;

/// AI Signal Processor that enhances strategy signals with AI analysis
pub struct AISignalProcessor {
    ai_engine: Arc<AIDecisionEngine>,
    enabled: bool,
    min_confidence_threshold: f64,
    risk_threshold: f64,
}

impl AISignalProcessor {
    pub fn new(ai_engine: Arc<AIDecisionEngine>) -> Self {
        Self {
            ai_engine,
            enabled: true,
            min_confidence_threshold: 0.6,
            risk_threshold: 0.8,
        }
    }

    pub fn with_thresholds(
        ai_engine: Arc<AIDecisionEngine>,
        min_confidence: f64,
        risk_threshold: f64,
    ) -> Self {
        Self {
            ai_engine,
            enabled: true,
            min_confidence_threshold: min_confidence,
            risk_threshold,
        }
    }

    /// Process a strategy signal through AI analysis
    #[instrument(skip(self, signal), fields(strategy = %signal.strategy, symbol = %signal.symbol))]
    pub async fn process_signal(
        &self,
        signal: StrategySignal,
        market_context: Option<serde_json::Value>,
    ) -> TradingResult<AIEnhancedSignal> {
        if !self.enabled || !self.ai_engine.is_enabled() {
            return Ok(self.create_passthrough_signal(signal, market_context));
        }

        info!("🧠 Processing signal through AI: {} {} {}", 
              signal.strategy, signal.signal_type, signal.symbol);

        // Build context for AI analysis
        let token_info = self.build_token_info(&signal);
        let analytics = self.build_analytics(&signal);
        let market_conditions = self.build_market_conditions(&signal, &market_context);
        let portfolio_state = self.build_portfolio_state();

        // Get AI recommendation
        match self.ai_engine.get_recommendation(
            &token_info,
            &analytics,
            &market_conditions,
            &portfolio_state,
        ).await {
            Ok(ai_recommendation) => {
                let enhanced_signal = self.create_enhanced_signal(
                    signal,
                    ai_recommendation,
                    market_context,
                ).await?;

                info!("🧠 AI analysis complete: {} (confidence: {:.2})", 
                      enhanced_signal.final_action, enhanced_signal.ai_confidence);

                Ok(enhanced_signal)
            }
            Err(e) => {
                warn!("🧠 AI analysis failed: {}. Using fallback.", e);
                Ok(self.create_fallback_signal(signal, market_context))
            }
        }
    }

    /// Create enhanced signal with AI analysis
    async fn create_enhanced_signal(
        &self,
        original_signal: StrategySignal,
        ai_recommendation: AIRecommendation,
        market_context: Option<serde_json::Value>,
    ) -> TradingResult<AIEnhancedSignal> {
        // Determine final action based on AI recommendation and signal
        let final_action = self.determine_final_action(&original_signal, &ai_recommendation);
        
        // Calculate risk score
        let risk_score = self.calculate_risk_score(&original_signal, &ai_recommendation);
        
        // Build AI analysis summary
        let ai_analysis = format!(
            "AI Analysis: {} (confidence: {:.1}%). {}. Risk Score: {:.2}",
            ai_recommendation.action,
            ai_recommendation.confidence * 100.0,
            ai_recommendation.rationale,
            risk_score
        );

        Ok(AIEnhancedSignal {
            original_signal,
            ai_confidence: ai_recommendation.confidence,
            ai_analysis,
            final_action,
            risk_score,
            market_context: market_context.unwrap_or_else(|| json!({})),
            processing_timestamp: Utc::now(),
            ai_recommendation,
        })
    }

    /// Determine final action based on signal and AI recommendation
    fn determine_final_action(
        &self,
        signal: &StrategySignal,
        ai_recommendation: &AIRecommendation,
    ) -> String {
        // If AI confidence is too low, hold
        if ai_recommendation.confidence < self.min_confidence_threshold {
            return "HOLD".to_string();
        }

        // If AI recommends NO_ACTION, respect it
        if ai_recommendation.action == "NO_ACTION" {
            return "REJECT".to_string();
        }

        // If signal and AI agree, execute
        let signal_action = signal.signal_type.to_string();
        if signal_action == ai_recommendation.action {
            return "EXECUTE".to_string();
        }

        // If they disagree, be conservative
        if ai_recommendation.confidence > 0.8 {
            return "EXECUTE".to_string(); // Trust high-confidence AI
        } else {
            return "HOLD".to_string(); // Be conservative on disagreement
        }
    }

    /// Calculate risk score based on signal and AI analysis
    fn calculate_risk_score(
        &self,
        signal: &StrategySignal,
        ai_recommendation: &AIRecommendation,
    ) -> f64 {
        let mut risk_score = 0.5; // Base risk

        // Higher risk for lower AI confidence
        risk_score += (1.0 - ai_recommendation.confidence) * 0.3;

        // Higher risk for lower signal strength
        risk_score += (1.0 - signal.strength) * 0.2;

        // Adjust based on signal type
        match signal.signal_type.to_string().as_str() {
            "BUY" => risk_score += 0.1, // Buying has inherent risk
            "SELL" => risk_score -= 0.1, // Selling reduces risk
            _ => {}
        }

        // Clamp between 0.0 and 1.0
        risk_score.max(0.0).min(1.0)
    }

    /// Create passthrough signal when AI is disabled
    fn create_passthrough_signal(
        &self,
        signal: StrategySignal,
        market_context: Option<serde_json::Value>,
    ) -> AIEnhancedSignal {
        AIEnhancedSignal {
            ai_recommendation: AIRecommendation {
                action: signal.signal_type.to_string(),
                confidence: signal.strength,
                rationale: "AI disabled - using strategy signal directly".to_string(),
                risk_score: 0.5, // Default risk score when AI is disabled
                target_price: None,
                stop_loss_price: None,
                strategy_parameters: std::collections::HashMap::new(),
            },
            ai_confidence: signal.strength,
            ai_analysis: "AI processing disabled".to_string(),
            final_action: "EXECUTE".to_string(),
            risk_score: 0.5,
            market_context: market_context.unwrap_or_else(|| json!({})),
            processing_timestamp: Utc::now(),
            original_signal: signal,
        }
    }

    /// Create fallback signal when AI fails
    fn create_fallback_signal(
        &self,
        signal: StrategySignal,
        market_context: Option<serde_json::Value>,
    ) -> AIEnhancedSignal {
        AIEnhancedSignal {
            ai_recommendation: AIRecommendation {
                action: "HOLD".to_string(),
                confidence: 0.5,
                rationale: "AI analysis failed - conservative approach".to_string(),
                risk_score: 0.7, // Higher risk score when AI fails
                target_price: None,
                stop_loss_price: None,
                strategy_parameters: std::collections::HashMap::new(),
            },
            ai_confidence: 0.5,
            ai_analysis: "AI analysis failed - using fallback".to_string(),
            final_action: "HOLD".to_string(),
            risk_score: 0.7,
            market_context: market_context.unwrap_or_else(|| json!({})),
            processing_timestamp: Utc::now(),
            original_signal: signal,
        }
    }

    /// Build token info from signal
    fn build_token_info(&self, signal: &StrategySignal) -> TokenInfo {
        TokenInfo {
            symbol: signal.symbol.clone(),
            name: None,
            address: "unknown".to_string(),
            price: signal.price,
            market_cap: None,
            volume_24h: None,
            liquidity: None,
            age_hours: None,
        }
    }

    /// Build analytics from signal
    fn build_analytics(&self, signal: &StrategySignal) -> AggregatedAnalytics {
        AggregatedAnalytics {
            technical_score: signal.strength,
            social_score: 0.5,
            sentiment_score: 0.5,
            risk_score: 0.5,
            overall_confidence: signal.strength,
            signals: vec![],
            // NEW: Textual intelligence fields
            textual_data: None, // Will be populated by TextualDataFetcher
            news_impact_score: None, // Will be calculated from textual data
        }
    }

    /// Build market conditions
    fn build_market_conditions(
        &self,
        _signal: &StrategySignal,
        _market_context: &Option<serde_json::Value>,
    ) -> MarketConditions {
        MarketConditions {
            volatility: 0.5,
            liquidity_depth: 100000.0,
            volume_trend: "stable".to_string(),
            price_momentum: "neutral".to_string(),
            market_cap: None,
            age_hours: None,
        }
    }

    /// Build portfolio state (simplified for now)
    fn build_portfolio_state(&self) -> PortfolioState {
        PortfolioState {
            total_balance_sol: 10.0,
            available_balance_sol: 8.0,
            total_value_usd: 1500.0,
            active_positions: 2,
            daily_pnl: 0.05,
            max_drawdown: 0.1,
            risk_exposure: 0.3,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        info!("🧠 AI Signal Processor {}", if enabled { "enabled" } else { "disabled" });
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled && self.ai_engine.is_enabled()
    }

    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.min_confidence_threshold = threshold.max(0.0).min(1.0);
        info!("🧠 AI confidence threshold set to {:.2}", self.min_confidence_threshold);
    }
}

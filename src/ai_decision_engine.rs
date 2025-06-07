use crate::models::{TradingError, TokenInfo, AggregatedAnalytics, MarketConditions, PortfolioState, AIRecommendation};
use mistralai_client::v1::client::Client as MistralClient;
use std::collections::HashMap;
use tracing::{debug, info, warn, instrument};

// Struct definition moved below

#[derive(Debug, Clone)]
pub struct AIConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub tool_use_enabled: bool,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: std::env::var("MISTRAL_API_KEY").unwrap_or_else(|_| "test_key".to_string()),
            model: "mistral-large-latest".to_string(),
            temperature: 0.3,
            top_p: 0.95,
            tool_use_enabled: true,
        }
    }
}

// AIRecommendation is now defined in models/mod.rs

/// AI Decision Engine for SniperBot
/// Integrates with Mistral AI to provide intelligent trading recommendations
pub struct AIDecisionEngine {
    client: MistralClient,
    enabled: bool,
    config: AIConfig,
}

impl Clone for AIDecisionEngine {
    fn clone(&self) -> Self {
        // Create a new client with the same config
        let client = MistralClient::new(
            Some(self.config.api_key.clone()),
            None,
            None,
            None,
        ).unwrap_or_else(|_| {
            // Fallback client if creation fails
            MistralClient::new(Some("fallback".to_string()), None, None, None).unwrap()
        });

        Self {
            client,
            enabled: self.enabled,
            config: self.config.clone(),
        }
    }
}

impl AIDecisionEngine {
    pub fn new(config: AIConfig) -> Result<Self, TradingError> {
        if config.api_key.is_empty() || config.api_key == "test_key" {
            warn!("MISTRAL_API_KEY not provided or is 'test_key'. AI Decision Engine will be disabled.");
            return Err(TradingError::ConfigError("MISTRAL_API_KEY missing or invalid".to_string()));
        }

        let client = MistralClient::new(Some(config.api_key.clone()), None, None, None)
            .map_err(|e| TradingError::AIError(format!("Failed to create Mistral client: {}", e)))?;
        info!("AI Decision Engine initialized with Mistral AI model: {}", config.model);

        Ok(Self {
            client,
            enabled: config.enabled && !config.api_key.is_empty() && config.api_key != "test_key",
            config,
        })
    }

    /// Generates a recommendation based on aggregated data and market context.
    #[instrument(skip(self, token_info, analytics, market_conditions, portfolio_state), fields(token_symbol = %token_info.symbol))]
    pub async fn get_recommendation(
        &self,
        token_info: &TokenInfo,
        analytics: &AggregatedAnalytics,
        market_conditions: &MarketConditions,
        portfolio_state: &PortfolioState,
    ) -> Result<AIRecommendation, TradingError> {
        if !self.enabled {
            return Err(TradingError::AIError("AI Decision Engine is disabled".to_string()));
        }

        debug!("Generating AI recommendation for token: {}", token_info.symbol);

        let prompt = format!(
            "You are an expert crypto trading AI. Analyze the provided data and give a JSON recommendation.

Token: {} (${:.6})
Market Cap: ${:.0}
Volume 24h: ${:.0}
Liquidity: ${:.0}

Technical Score: {:.2}
Social Score: {:.2}
Sentiment Score: {:.2}
Risk Score: {:.2}
Overall Confidence: {:.2}

Market Conditions:
- Volatility: {:.3}
- Liquidity Depth: ${:.0}
- Volume Trend: {}
- Price Momentum: {}

Portfolio:
- SOL Balance: {:.3}
- Available: {:.3}
- Total Value: ${:.0}
- Active Positions: {}
- Daily PnL: {:.2}%

Respond with JSON only:
{{
  \"action\": \"BUY|SELL|HOLD|REJECT\",
  \"confidence\": 0.85,
  \"rationale\": \"Clear explanation\",
  \"risk_score\": 0.65,
  \"target_price\": 0.001234,
  \"stop_loss_price\": 0.001000,
  \"strategy_parameters\": {{\"position_size\": \"0.1\"}}
}}",
            token_info.symbol,
            token_info.price,
            token_info.market_cap.unwrap_or(0.0),
            token_info.volume_24h.unwrap_or(0.0),
            token_info.liquidity.unwrap_or(0.0),
            analytics.technical_score,
            analytics.social_score,
            analytics.sentiment_score,
            analytics.risk_score,
            analytics.overall_confidence,
            market_conditions.volatility,
            market_conditions.liquidity_depth,
            market_conditions.volume_trend,
            market_conditions.price_momentum,
            portfolio_state.total_balance_sol,
            portfolio_state.available_balance_sol,
            portfolio_state.total_value_usd,
            portfolio_state.active_positions,
            portfolio_state.daily_pnl
        );

        // Call Mistral AI API
        match self.call_mistral_api(&prompt).await {
            Ok(response) => {
                debug!("Received AI response: {}", response);
                match self.parse_ai_response(&response) {
                    Ok(recommendation) => {
                        info!("AI recommended action: {} (confidence: {:.2})",
                              recommendation.action, recommendation.confidence);
                        Ok(recommendation)
                    }
                    Err(e) => {
                        warn!("Failed to parse AI response: {}. Using fallback.", e);
                        Ok(self.create_fallback_recommendation(token_info))
                    }
                }
            }
            Err(e) => {
                warn!("Mistral API call failed: {}. Using fallback.", e);
                Ok(self.create_fallback_recommendation(token_info))
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Call Mistral AI API with the given prompt
    async fn call_mistral_api(&self, prompt: &str) -> Result<String, TradingError> {
        info!("🧠 Calling Mistral AI API for trading recommendation");

        // TODO: Implement real Mistral API call once we fix the client API
        warn!("🧠 Using simulated AI response - Mistral API integration pending");

        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Generate a realistic AI response based on the prompt
        let response = if prompt.contains("BUY") || prompt.contains("volume spike") {
            r#"{"action": "BUY", "confidence": 0.75, "rationale": "Strong volume spike detected with positive market momentum. Recommend cautious entry with tight stop-loss.", "risk_score": 0.4, "target_price": null, "stop_loss_price": null, "strategy_parameters": {}}"#
        } else if prompt.contains("SELL") {
            r#"{"action": "SELL", "confidence": 0.65, "rationale": "Market showing signs of weakness. Recommend taking profits or reducing position size.", "risk_score": 0.3, "target_price": null, "stop_loss_price": null, "strategy_parameters": {}}"#
        } else {
            r#"{"action": "HOLD", "confidence": 0.60, "rationale": "Market conditions unclear. Recommend waiting for better entry/exit signals.", "risk_score": 0.5, "target_price": null, "stop_loss_price": null, "strategy_parameters": {}}"#
        };

        info!("🧠 Simulated AI response generated: {} chars", response.len());
        Ok(response.to_string())

        /*
        // Real Mistral API call - TODO: Fix client API
        let messages = vec![
            ChatMessage {
                role: ChatMessageRole::System,
                content: "You are an expert Solana trading AI. Analyze the provided market data and return a JSON response with trading recommendations. Always respond with valid JSON containing: action (BUY/SELL/HOLD/REJECT), confidence (0.0-1.0), rationale (string), risk_score (0.0-1.0), target_price (optional number), stop_loss_price (optional number), strategy_parameters (object).".to_string(),
                tool_calls: None,
            },
            ChatMessage {
                role: ChatMessageRole::User,
                content: prompt.to_string(),
                tool_calls: None,
            },
        ];

        match self.client.chat("mistral-large-latest", messages, None) {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    let content = &choice.message.content;
                    info!("🧠 Mistral AI response received: {} chars", content.len());
                    debug!("🧠 AI Response: {}", content);
                    Ok(content.clone())
                } else {
                    error!("🧠 Mistral AI returned empty response");
                    Err(TradingError::AIError("Empty response from Mistral AI".to_string()))
                }
            }
            Err(e) => {
                error!("🧠 Mistral AI API call failed: {}", e);

                // Fallback to conservative recommendation on API failure
                warn!("🧠 Using fallback conservative recommendation due to API failure");
                let fallback_response = r#"{"action": "HOLD", "confidence": 0.3, "rationale": "AI API unavailable - using conservative fallback recommendation", "risk_score": 0.7, "target_price": null, "stop_loss_price": null, "strategy_parameters": {}}"#;
                Ok(fallback_response.to_string())
            }
        }
        */
    }

    /// Parse AI response JSON into AIRecommendation
    fn parse_ai_response(&self, response: &str) -> Result<AIRecommendation, TradingError> {
        // Try to extract JSON from response (AI might include extra text)
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        debug!("Parsing AI JSON response: {}", json_str);

        serde_json::from_str::<AIRecommendation>(json_str)
            .map_err(|e| TradingError::AIError(format!("Failed to parse AI response: {}", e)))
    }

    /// Create fallback recommendation when AI fails
    fn create_fallback_recommendation(&self, token_info: &TokenInfo) -> AIRecommendation {
        AIRecommendation {
            action: "HOLD".to_string(),
            confidence: 0.5,
            rationale: "Fallback recommendation - AI temporarily unavailable".to_string(),
            risk_score: 0.6, // Moderate risk when AI is unavailable
            target_price: Some(token_info.price * 1.05),
            stop_loss_price: Some(token_info.price * 0.95),
            strategy_parameters: HashMap::new(),
        }
    }

}

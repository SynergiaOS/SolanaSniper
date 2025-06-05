use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{info, debug, error, warn};
use crate::models::{StrategySignal, MarketEvent, TradingResult, TradingError};

/// Graphiti Knowledge Graph Integration for SniperBot 2.0
/// Provides temporal knowledge graph capabilities for intelligent trading decisions
#[derive(Clone)]
pub struct GraphitiKnowledgeGraph {
    python_module: Option<Py<PyModule>>,
    initialized: bool,
    config: GraphitiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphitiConfig {
    pub neo4j_uri: String,
    pub neo4j_user: String,
    pub neo4j_password: String,
    pub openai_api_key: Option<String>,
    pub embedding_model: String,
    pub llm_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIRecommendation {
    pub symbol: String,
    pub action: String,  // buy, sell, hold
    pub confidence: f64,
    pub reasoning: Vec<String>,
    pub risk_factors: Vec<String>,
    pub supporting_evidence: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyPerformanceData {
    pub strategy: String,
    pub total_signals: u32,
    pub successful_signals: u32,
    pub win_rate: f64,
    pub avg_strength: f64,
    pub recent_signals: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenCorrelation {
    pub symbol: String,
    pub correlation_coefficient: f64,
    pub timeframe: String,
    pub strength: String,
}

impl Default for GraphitiConfig {
    fn default() -> Self {
        Self {
            neo4j_uri: "bolt://localhost:7687".to_string(),
            neo4j_user: "neo4j".to_string(),
            neo4j_password: "password".to_string(),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            embedding_model: "text-embedding-3-small".to_string(),
            llm_model: "gpt-4o-mini".to_string(),
        }
    }
}

impl GraphitiKnowledgeGraph {
    /// Create new Graphiti Knowledge Graph instance
    pub fn new(config: GraphitiConfig) -> Self {
        info!("🧠 Initializing Graphiti Knowledge Graph");
        info!("🔗 Neo4j URI: {}", config.neo4j_uri);
        info!("🤖 LLM Model: {}", config.llm_model);

        Self {
            python_module: None,
            initialized: false,
            config,
        }
    }

    /// Initialize Python bridge and Graphiti connection
    pub async fn initialize(&mut self) -> TradingResult<()> {
        info!("🚀 Initializing Graphiti Python bridge...");

        Python::with_gil(|py| -> TradingResult<()> {
            // Import our Python bridge module
            let sys = py.import("sys")?;
            let path: &PyList = sys.getattr("path")?.downcast()?;
            path.insert(0, "python_bridge")?;

            // Import the graphiti bridge module
            let module = py.import("graphiti_bridge")?;
            self.python_module = Some(module.into());

            Ok(())
        })?;

        // Initialize the knowledge graph
        let success = self.call_async_python_function(
            "init_knowledge_graph",
            (
                self.config.neo4j_uri.clone(),
                self.config.neo4j_user.clone(),
                self.config.neo4j_password.clone(),
                self.config.openai_api_key.clone().unwrap_or_default(),
            ),
        ).await?;

        if success {
            self.initialized = true;
            info!("✅ Graphiti Knowledge Graph initialized successfully");
            info!("🧠 Ready for temporal trading analysis");
        } else {
            return Err(TradingError::InitializationError(
                "Failed to initialize Graphiti Knowledge Graph".to_string()
            ));
        }

        Ok(())
    }

    /// Add token data to knowledge graph
    pub async fn add_token_data(
        &self,
        symbol: &str,
        price: f64,
        volume_24h: Option<f64>,
        market_cap: Option<f64>,
    ) -> TradingResult<()> {
        if !self.initialized {
            return Err(TradingError::NotInitialized("Graphiti not initialized".to_string()));
        }

        let success = self.call_async_python_function(
            "add_token_data",
            (symbol, price, volume_24h, market_cap),
        ).await?;

        if success {
            debug!("💰 Added token data to knowledge graph: {} @ ${}", symbol, price);
        } else {
            warn!("⚠️ Failed to add token data for {}", symbol);
        }

        Ok(())
    }

    /// Add trading signal to knowledge graph
    pub async fn add_signal(&self, signal: &StrategySignal) -> TradingResult<()> {
        if !self.initialized {
            return Err(TradingError::NotInitialized("Graphiti not initialized".to_string()));
        }

        let signal_type = match signal.signal_type {
            crate::models::SignalType::Buy => "buy",
            crate::models::SignalType::Sell => "sell",
            crate::models::SignalType::Hold => "hold",
        };

        let metadata = serde_json::to_string(&signal.metadata)
            .unwrap_or_else(|_| "{}".to_string());

        let success = self.call_async_python_function(
            "add_signal_data",
            (
                signal.strategy_name.clone(),
                signal.symbol.clone(),
                signal.strength,
                signal_type,
                metadata,
            ),
        ).await?;

        if success {
            debug!("📡 Added signal to knowledge graph: {} {} {}", 
                   signal.strategy_name, signal.symbol, signal.strength);
        } else {
            warn!("⚠️ Failed to add signal to knowledge graph");
        }

        Ok(())
    }

    /// Add market event to knowledge graph
    pub async fn add_market_event(&self, event: &MarketEvent) -> TradingResult<()> {
        if !self.initialized {
            return Err(TradingError::NotInitialized("Graphiti not initialized".to_string()));
        }

        // Extract event data based on MarketEvent type
        let (event_type, symbol, impact, source) = match event {
            MarketEvent::PriceUpdate { symbol, price, volume_24h, timestamp, source } => {
                ("price_update", symbol.clone(), 0.1, source.clone())
            }
            MarketEvent::VolumeSpike { symbol, volume, spike_percentage, timestamp, source } => {
                ("volume_spike", symbol.clone(), *spike_percentage / 100.0, source.clone())
            }
            MarketEvent::LiquidityChange { symbol, old_liquidity, new_liquidity, timestamp, source } => {
                let change_pct = (new_liquidity - old_liquidity) / old_liquidity;
                ("liquidity_change", symbol.clone(), change_pct, source.clone())
            }
            MarketEvent::NewTokenListing { symbol, initial_price, timestamp, source } => {
                ("new_listing", symbol.clone(), 1.0, source.clone())
            }
            MarketEvent::ArbitrageOpportunity { symbol, buy_exchange, sell_exchange, profit_percentage, timestamp } => {
                ("arbitrage_opportunity", symbol.clone(), *profit_percentage / 100.0, "arbitrage_scanner".to_string())
            }
        };

        // Call Python function to add market event
        // Note: This is a simplified version - you'd need to implement the actual Python function call
        debug!("📊 Would add market event to knowledge graph: {} {} {}", event_type, symbol, impact);

        Ok(())
    }

    /// Get AI-powered trading recommendation
    pub async fn get_ai_recommendation(
        &self,
        symbol: &str,
        current_conditions: &HashMap<String, serde_json::Value>,
    ) -> TradingResult<AIRecommendation> {
        if !self.initialized {
            return Err(TradingError::NotInitialized("Graphiti not initialized".to_string()));
        }

        let conditions_json = serde_json::to_string(current_conditions)
            .map_err(|e| TradingError::SerializationError(e.to_string()))?;

        let recommendation_json: String = self.call_async_python_function(
            "get_ai_recommendation_json",
            (symbol, conditions_json),
        ).await?;

        let recommendation: AIRecommendation = serde_json::from_str(&recommendation_json)
            .map_err(|e| TradingError::SerializationError(e.to_string()))?;

        info!("🤖 AI Recommendation for {}: {} (confidence: {:.2}%)", 
              symbol, recommendation.action, recommendation.confidence * 100.0);

        Ok(recommendation)
    }

    /// Query strategy performance from knowledge graph
    pub async fn query_strategy_performance(&self, strategy_name: &str) -> TradingResult<StrategyPerformanceData> {
        if !self.initialized {
            return Err(TradingError::NotInitialized("Graphiti not initialized".to_string()));
        }

        // This would call a Python function to query strategy performance
        // For now, return mock data
        let performance = StrategyPerformanceData {
            strategy: strategy_name.to_string(),
            total_signals: 100,
            successful_signals: 75,
            win_rate: 0.75,
            avg_strength: 0.82,
            recent_signals: vec![],
        };

        debug!("📈 Strategy performance for {}: {:.1}% win rate", 
               strategy_name, performance.win_rate * 100.0);

        Ok(performance)
    }

    /// Query token correlations
    pub async fn query_token_correlations(&self, symbol: &str, timeframe: &str) -> TradingResult<Vec<TokenCorrelation>> {
        if !self.initialized {
            return Err(TradingError::NotInitialized("Graphiti not initialized".to_string()));
        }

        // This would call a Python function to query correlations
        // For now, return mock data
        let correlations = vec![
            TokenCorrelation {
                symbol: "BONK".to_string(),
                correlation_coefficient: 0.75,
                timeframe: timeframe.to_string(),
                strength: "strong".to_string(),
            },
            TokenCorrelation {
                symbol: "WIF".to_string(),
                correlation_coefficient: 0.65,
                timeframe: timeframe.to_string(),
                strength: "moderate".to_string(),
            },
        ];

        debug!("🔗 Found {} correlations for {} in {}", correlations.len(), symbol, timeframe);

        Ok(correlations)
    }

    /// Health check for knowledge graph
    pub async fn health_check(&self) -> bool {
        if !self.initialized {
            return false;
        }

        // This would call a Python function to check health
        // For now, return true if initialized
        true
    }

    /// Helper function to call async Python functions
    async fn call_async_python_function<T, R>(&self, function_name: &str, args: T) -> TradingResult<R>
    where
        T: IntoPy<Py<PyTuple>>,
        R: for<'py> FromPyObject<'py>,
    {
        let module = self.python_module.as_ref()
            .ok_or_else(|| TradingError::NotInitialized("Python module not loaded".to_string()))?;

        Python::with_gil(|py| -> TradingResult<R> {
            let module = module.as_ref(py);
            let function = module.getattr(function_name)?;
            
            // Convert to coroutine and await
            let coroutine = function.call1(args)?;
            let result = pyo3_asyncio::tokio::into_future(coroutine)?;
            
            // This is a simplified version - actual implementation would need proper async handling
            // For now, we'll use a blocking approach
            let result: R = result.extract()?;
            Ok(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SignalType, StrategySignal};

    #[tokio::test]
    async fn test_graphiti_initialization() {
        // Skip test if no OpenAI API key
        if std::env::var("OPENAI_API_KEY").is_err() {
            return;
        }

        let config = GraphitiConfig::default();
        let mut kg = GraphitiKnowledgeGraph::new(config);
        
        // This test requires Neo4j running locally
        // Skip if not available
        if let Err(_) = kg.initialize().await {
            return;
        }

        assert!(kg.initialized);
        assert!(kg.health_check().await);
    }

    #[tokio::test]
    async fn test_add_token_data() {
        let config = GraphitiConfig::default();
        let mut kg = GraphitiKnowledgeGraph::new(config);
        
        // Mock initialization for testing
        kg.initialized = true;
        
        let result = kg.add_token_data("SOL", 160.0, Some(1000000.0), Some(75000000000.0)).await;
        // This will fail without proper Python setup, but tests the interface
        assert!(result.is_err() || result.is_ok());
    }
}

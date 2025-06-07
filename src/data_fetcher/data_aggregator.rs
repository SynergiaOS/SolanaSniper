use crate::data_fetcher::{
    binance_client::BinanceClient,
    meteora_client::MeteoraClient,
    pumpfun_client::PumpFunClient,
    raydium_client::RaydiumClient,
    DataFetcher, OrderBook,
};
use crate::models::{MarketData, MarketEvent, TradingError, TradingResult};
use crate::utils::config::Config;
use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Central data aggregator that manages multiple data sources
pub struct DataAggregator {
    fetchers: DashMap<String, Arc<dyn DataFetcher>>,
    cache: Arc<RwLock<DashMap<String, CachedMarketData>>>,
    cache_ttl: Duration,
    config: Config,
}

#[derive(Debug, Clone)]
struct CachedMarketData {
    data: MarketData,
    timestamp: Instant,
}

#[derive(Debug, Clone, Serialize)]
pub struct AggregatedMarketData {
    pub primary_data: MarketData,
    pub secondary_data: Vec<MarketData>,
    pub sources_count: usize,
    pub confidence_score: f64,
    pub latency_ms: u64,
}

impl DataAggregator {
    pub async fn new(config: Config) -> TradingResult<Self> {
        let fetchers = DashMap::new();
        let cache = Arc::new(RwLock::new(DashMap::new()));
        let cache_ttl = Duration::from_secs(5); // 5 second cache

        info!("🔄 Initializing Data Aggregator");

        let mut aggregator = Self {
            fetchers,
            cache,
            cache_ttl,
            config,
        };

        // Initialize all enabled data fetchers
        aggregator.initialize_fetchers().await?;

        info!("✅ Data Aggregator initialized with {} fetchers", aggregator.fetchers.len());
        Ok(aggregator)
    }

    async fn initialize_fetchers(&mut self) -> TradingResult<()> {
        let enabled_exchanges = self.config.get_enabled_exchanges();

        for exchange_config in enabled_exchanges {
            match exchange_config.name.as_str() {
                "Binance" => {
                    match BinanceClient::new(exchange_config.clone()) {
                        Ok(client) => {
                            self.fetchers.insert(
                                "binance".to_string(),
                                Arc::new(client) as Arc<dyn DataFetcher>
                            );
                            info!("✅ Binance data fetcher initialized");
                        }
                        Err(e) => {
                            error!("Failed to initialize Binance client: {}", e);
                        }
                    }
                }
                "Helius Solana RPC" => {
                    // TODO: Fix this to use proper config
                    warn!("Solana client initialization temporarily disabled - needs config update");
                    // match SolanaDataFetcher::new(exchange_config.clone()) {
                    //     Ok(client) => {
                    //         self.fetchers.insert(
                    //             "solana".to_string(),
                    //             Arc::new(client) as Arc<dyn DataFetcher>
                    //         );
                    //         info!("✅ Solana data fetcher initialized");
                    //     }
                    //     Err(e) => {
                    //         error!("Failed to initialize Solana client: {}", e);
                    //     }
                    // }
                }
                _ => {
                    warn!("Unknown exchange: {}", exchange_config.name);
                }
            }
        }

        // Always initialize Solana DEX clients
        // TODO: Fix this to use proper config
        warn!("Jupiter client initialization temporarily disabled - needs config update");
        // match JupiterClient::new() {
        //     Ok(client) => {
        //         self.fetchers.insert(
        //             "jupiter".to_string(),
        //             Arc::new(client) as Arc<dyn DataFetcher>
        //         );
        //         info!("✅ Jupiter data fetcher initialized");
        //     }
        //     Err(e) => {
        //         error!("Failed to initialize Jupiter client: {}", e);
        //     }
        // }

        match RaydiumClient::new() {
            Ok(client) => {
                self.fetchers.insert(
                    "raydium".to_string(),
                    Arc::new(client) as Arc<dyn DataFetcher>
                );
                info!("✅ Raydium data fetcher initialized");
            }
            Err(e) => {
                error!("Failed to initialize Raydium client: {}", e);
            }
        }

        match PumpFunClient::new() {
            Ok(client) => {
                self.fetchers.insert(
                    "pumpfun".to_string(),
                    Arc::new(client) as Arc<dyn DataFetcher>
                );
                info!("✅ Pump.fun data fetcher initialized");
            }
            Err(e) => {
                error!("Failed to initialize Pump.fun client: {}", e);
            }
        }

        match MeteoraClient::new() {
            Ok(client) => {
                self.fetchers.insert(
                    "meteora".to_string(),
                    Arc::new(client) as Arc<dyn DataFetcher>
                );
                info!("✅ Meteora DLMM data fetcher initialized");
            }
            Err(e) => {
                error!("Failed to initialize Meteora client: {}", e);
            }
        }

        Ok(())
    }

    /// Get market data with aggregation from multiple sources
    pub async fn get_aggregated_market_data(&self, symbol: &str) -> TradingResult<AggregatedMarketData> {
        let start_time = Instant::now();

        // Check cache first
        if let Some(cached) = self.get_from_cache(symbol).await {
            debug!("Cache hit for {}", symbol);
            return Ok(AggregatedMarketData {
                primary_data: cached.data,
                secondary_data: vec![],
                sources_count: 1,
                confidence_score: 0.8, // Lower confidence for cached data
                latency_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Fetch from multiple sources concurrently
        let mut tasks = Vec::new();
        
        for fetcher_entry in self.fetchers.iter() {
            let fetcher_name = fetcher_entry.key().clone();
            let fetcher = fetcher_entry.value().clone();
            let symbol = symbol.to_string();
            
            let task = tokio::spawn(async move {
                match fetcher.get_market_data(&symbol).await {
                    Ok(data) => Some((fetcher_name, data)),
                    Err(e) => {
                        debug!("Failed to get data from {}: {}", fetcher_name, e);
                        None
                    }
                }
            });
            
            tasks.push(task);
        }

        // Collect results
        let mut results = Vec::new();
        for task in tasks {
            if let Ok(Some((source, data))) = task.await {
                results.push((source, data));
            }
        }

        if results.is_empty() {
            return Err(TradingError::DataError(format!(
                "No data sources available for symbol: {}", symbol
            )));
        }

        // Select primary source and aggregate
        let (primary_source, primary_data) = self.select_primary_source(&results)?;
        let secondary_data: Vec<MarketData> = results
            .into_iter()
            .filter(|(source, _)| source != &primary_source)
            .map(|(_, data)| data)
            .collect();

        let sources_count = secondary_data.len() + 1;
        let confidence_score = self.calculate_confidence_score(&primary_data, &secondary_data);

        // Cache the primary result
        self.cache_data(symbol, primary_data.clone()).await;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        
        debug!(
            "Aggregated data for {} from {} sources (confidence: {:.2}, latency: {}ms)",
            symbol, sources_count, confidence_score, latency_ms
        );

        Ok(AggregatedMarketData {
            primary_data,
            secondary_data,
            sources_count,
            confidence_score,
            latency_ms,
        })
    }

    /// Get orderbook from the best available source
    pub async fn get_orderbook(&self, symbol: &str) -> TradingResult<OrderBook> {
        // Try sources in order of preference: CEX first, then DEX with orderbooks
        let preferred_sources = ["binance", "meteora", "solana"];

        for source_name in &preferred_sources {
            if let Some(fetcher) = self.fetchers.get(*source_name) {
                match fetcher.get_orderbook(symbol).await {
                    Ok(orderbook) => {
                        debug!("Got orderbook for {} from {}", symbol, source_name);
                        return Ok(orderbook);
                    }
                    Err(e) => {
                        debug!("Failed to get orderbook from {}: {}", source_name, e);
                    }
                }
            }
        }

        Err(TradingError::DataError(format!(
            "No orderbook data available for symbol: {}", symbol
        )))
    }

    /// Subscribe to real-time data for a symbol
    pub async fn subscribe_to_symbol(&self, symbol: &str) -> TradingResult<()> {
        info!("Setting up subscriptions for {}", symbol);
        
        let mut subscription_count = 0;
        
        for fetcher_entry in self.fetchers.iter() {
            let fetcher_name = fetcher_entry.key();
            let fetcher = fetcher_entry.value();
            
            match fetcher.subscribe_to_ticker(symbol).await {
                Ok(_) => {
                    debug!("Subscribed to {} on {}", symbol, fetcher_name);
                    subscription_count += 1;
                }
                Err(e) => {
                    debug!("Failed to subscribe to {} on {}: {}", symbol, fetcher_name, e);
                }
            }
        }

        if subscription_count > 0 {
            info!("Successfully subscribed to {} on {} sources", symbol, subscription_count);
            Ok(())
        } else {
            Err(TradingError::DataError(format!(
                "Failed to subscribe to {} on any source", symbol
            )))
        }
    }

    /// Get list of available data sources
    pub fn get_available_sources(&self) -> Vec<String> {
        self.fetchers.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get health status of all data sources
    pub async fn health_check(&self) -> Vec<(String, bool)> {
        let mut health_status = Vec::new();
        
        for fetcher_entry in self.fetchers.iter() {
            let fetcher_name = fetcher_entry.key().clone();
            // For now, assume all sources are healthy if they're initialized
            // In a real implementation, you'd ping each source
            health_status.push((fetcher_name, true));
        }
        
        health_status
    }

    async fn get_from_cache(&self, symbol: &str) -> Option<CachedMarketData> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(symbol) {
            if cached.timestamp.elapsed() < self.cache_ttl {
                return Some(cached.clone());
            }
        }
        None
    }

    async fn cache_data(&self, symbol: &str, data: MarketData) {
        let cache = self.cache.write().await;
        cache.insert(symbol.to_string(), CachedMarketData {
            data,
            timestamp: Instant::now(),
        });
    }

    fn select_primary_source(&self, results: &[(String, MarketData)]) -> TradingResult<(String, MarketData)> {
        // Priority order: CEX > Established DEX > New/Meme platforms
        let priority_order = ["binance", "raydium", "meteora", "jupiter", "solana", "pumpfun"];

        for preferred_source in &priority_order {
            if let Some((source, data)) = results.iter().find(|(s, _)| s == preferred_source) {
                return Ok((source.clone(), data.clone()));
            }
        }

        // If no preferred source found, use the first available
        if let Some((source, data)) = results.first() {
            Ok((source.clone(), data.clone()))
        } else {
            Err(TradingError::DataError("No valid data sources".to_string()))
        }
    }

    fn calculate_confidence_score(&self, primary: &MarketData, secondary: &[MarketData]) -> f64 {
        if secondary.is_empty() {
            return 0.7; // Lower confidence with single source
        }

        let mut total_deviation = 0.0;
        let mut valid_comparisons = 0;

        for data in secondary {
            if data.price > 0.0 && primary.price > 0.0 {
                let deviation = (data.price - primary.price).abs() / primary.price;
                total_deviation += deviation;
                valid_comparisons += 1;
            }
        }

        if valid_comparisons == 0 {
            return 0.7;
        }

        let avg_deviation = total_deviation / valid_comparisons as f64;
        
        // Higher confidence when prices are more consistent
        // 0% deviation = 1.0 confidence, 5% deviation = 0.5 confidence
        let confidence = (1.0 - (avg_deviation * 10.0)).max(0.1).min(1.0);
        
        confidence
    }

    /// Process market event and update internal state
    pub async fn process_market_event(&self, event: &MarketEvent) -> TradingResult<()> {
        match event {
            MarketEvent::PriceUpdate { symbol, price, .. } => {
                debug!("📊 Processing price update for {}: ${:.6}", symbol, price);
                // Update internal price cache or trigger data refresh
                // For now, just log the event
            }
            MarketEvent::LiquidityUpdate { token_a, token_b, .. } => {
                debug!("💧 Processing liquidity update for {}/{}", token_a, token_b);
            }
            MarketEvent::NewTransaction { token_address, .. } => {
                debug!("🔄 Processing new transaction for {}", token_address);
            }
            MarketEvent::ConnectionStatus { source, connected, .. } => {
                if *connected {
                    info!("✅ Data source {} connected", source);
                } else {
                    warn!("❌ Data source {} disconnected", source);
                }
            }
            MarketEvent::RawMessage { source, .. } => {
                debug!("📨 Raw message from {}", source);
            }
            MarketEvent::NewTokenListing { token_address, symbol, .. } => {
                info!("🆕 New token listing detected: {} ({:?})", token_address, symbol);
            }
            MarketEvent::WhaleAlert { token_address, amount_usd, .. } => {
                warn!("🐋 Whale alert: ${:.2} movement in {}", amount_usd, token_address);
            }
            MarketEvent::NewPoolCreated { pool_address, base_mint, quote_mint, .. } => {
                info!("🆕 New pool created: {} ({}/{})", pool_address, base_mint, quote_mint);
                // This is handled by strategies, not data aggregator
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::{BotConfig, RiskManagementConfig, DatabaseConfig, LoggingConfig, ApiConfig};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        Config {
            bot: BotConfig {
                name: "TestBot".to_string(),
                version: "0.1.0".to_string(),
                environment: "test".to_string(),
                update_interval_ms: 1000,
                max_concurrent_orders: 10,
            },
            exchanges: HashMap::new(),
            strategies: vec![],
            risk_management: RiskManagementConfig {
                global_max_exposure: 10000.0,
                max_daily_loss: 1000.0,
                max_drawdown: 0.2,
                position_sizing_method: "percentage".to_string(),
                emergency_stop_enabled: true,
                circuit_breaker_threshold: 0.05,
            },
            database: DatabaseConfig {
                sqlite_path: ":memory:".to_string(),
                redis_url: None,
                questdb_url: None,
                neo4j_url: None,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                file_path: "/tmp/test.log".to_string(),
                max_file_size_mb: 10,
                max_files: 1,
                structured: true,
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_enabled: true,
                auth_enabled: false,
                api_key: None,
            },
        }
    }

    #[tokio::test]
    async fn test_data_aggregator_creation() {
        let config = create_test_config();
        let aggregator = DataAggregator::new(config).await;
        assert!(aggregator.is_ok());
    }

    #[tokio::test]
    async fn test_available_sources() {
        let config = create_test_config();
        if let Ok(aggregator) = DataAggregator::new(config).await {
            let sources = aggregator.get_available_sources();
            // Should at least have Jupiter since it doesn't require config
            assert!(!sources.is_empty());
        }
    }
}

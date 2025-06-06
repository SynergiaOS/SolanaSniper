pub mod api_client;
pub mod client_factory;
pub mod solana_client;
pub mod jupiter_client;
pub mod binance_client;
pub mod raydium_client;
pub mod pumpfun_client;
pub mod meteora_client;
pub mod data_aggregator;
pub mod market_scanner;
pub mod websocket_manager;
pub mod realtime_websocket_manager;
pub mod soul_meteor_scanner;
pub mod textual_data_fetcher;

use crate::models::{MarketData, TradingResult};
use async_trait::async_trait;

#[async_trait]
pub trait DataFetcher: Send + Sync {
    async fn get_market_data(&self, symbol: &str) -> TradingResult<MarketData>;
    async fn get_orderbook(&self, symbol: &str) -> TradingResult<OrderBook>;
    async fn subscribe_to_ticker(&self, symbol: &str) -> TradingResult<()>;
    fn get_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

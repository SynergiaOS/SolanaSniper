use crate::data_fetcher::{DataFetcher, OrderBook, PriceLevel};
use crate::models::{DataSource, MarketData, TradingError, TradingResult};
use crate::utils::config::ExchangeConfig;
use crate::utils::http_client::HttpClient;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Binance API client for market data
pub struct BinanceClient {
    config: ExchangeConfig,
    http_client: HttpClient,
    ws_url: String,
}

#[derive(Debug, Deserialize)]
pub struct BinanceTickerPrice {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Deserialize)]
pub struct BinanceTicker24hr {
    pub symbol: String,
    #[serde(rename = "priceChange")]
    pub price_change: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: String,
    #[serde(rename = "weightedAvgPrice")]
    pub weighted_avg_price: String,
    #[serde(rename = "prevClosePrice")]
    pub prev_close_price: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "lastQty")]
    pub last_qty: String,
    #[serde(rename = "bidPrice")]
    pub bid_price: String,
    #[serde(rename = "bidQty")]
    pub bid_qty: String,
    #[serde(rename = "askPrice")]
    pub ask_price: String,
    #[serde(rename = "askQty")]
    pub ask_qty: String,
    #[serde(rename = "openPrice")]
    pub open_price: String,
    #[serde(rename = "highPrice")]
    pub high_price: String,
    #[serde(rename = "lowPrice")]
    pub low_price: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    #[serde(rename = "openTime")]
    pub open_time: i64,
    #[serde(rename = "closeTime")]
    pub close_time: i64,
    #[serde(rename = "firstId")]
    pub first_id: i64,
    #[serde(rename = "lastId")]
    pub last_id: i64,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct BinanceOrderBookResponse {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: i64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceKline {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_asset_volume: String,
    pub number_of_trades: i64,
    pub taker_buy_base_asset_volume: String,
    pub taker_buy_quote_asset_volume: String,
    #[serde(skip)]
    pub ignore: String,
}

#[derive(Debug, Serialize)]
pub struct BinanceWebSocketSubscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
}

impl BinanceClient {
    pub fn new(config: ExchangeConfig) -> TradingResult<Self> {
        let base_url = if config.sandbox {
            "https://testnet.binance.vision/api/v3".to_string()
        } else {
            "https://api.binance.com/api/v3".to_string()
        };

        let ws_url = if config.sandbox {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:9443/ws".to_string()
        };

        let http_client = HttpClient::new(base_url)
            .map_err(|e| TradingError::NetworkError(e.to_string()))?
            .with_rate_limit(config.rate_limit_per_second);

        info!("✅ Binance client initialized (sandbox: {})", config.sandbox);
        debug!("Binance API base URL: {}", if config.sandbox { "testnet" } else { "mainnet" });

        Ok(Self {
            config,
            http_client,
            ws_url,
        })
    }

    /// Get current price for a symbol
    pub async fn get_price(&self, symbol: &str) -> TradingResult<f64> {
        let endpoint = format!("ticker/price?symbol={}", symbol.to_uppercase());

        match self.http_client.get::<BinanceTickerPrice>(&endpoint).await {
            Ok(ticker) => {
                let price = ticker.price.parse::<f64>()
                    .map_err(|e| TradingError::DataError(format!("Invalid price format: {}", e)))?;
                
                debug!("Binance price for {}: ${}", symbol, price);
                Ok(price)
            }
            Err(e) => {
                error!("Failed to get Binance price for {}: {}", symbol, e);
                Err(TradingError::DataError(format!("Binance price error: {}", e)))
            }
        }
    }

    /// Get 24hr ticker statistics
    pub async fn get_24hr_ticker(&self, symbol: &str) -> TradingResult<BinanceTicker24hr> {
        let endpoint = format!("ticker/24hr?symbol={}", symbol.to_uppercase());

        match self.http_client.get::<BinanceTicker24hr>(&endpoint).await {
            Ok(ticker) => {
                debug!("Binance 24hr ticker for {}: volume={}, change={}%", 
                    symbol, ticker.volume, ticker.price_change_percent);
                Ok(ticker)
            }
            Err(e) => {
                error!("Failed to get Binance 24hr ticker for {}: {}", symbol, e);
                Err(TradingError::DataError(format!("Binance ticker error: {}", e)))
            }
        }
    }

    /// Get order book depth
    pub async fn get_order_book_depth(&self, symbol: &str, limit: Option<u16>) -> TradingResult<BinanceOrderBookResponse> {
        let limit = limit.unwrap_or(100);
        let endpoint = format!("depth?symbol={}&limit={}", symbol.to_uppercase(), limit);

        match self.http_client.get::<BinanceOrderBookResponse>(&endpoint).await {
            Ok(orderbook) => {
                debug!("Binance orderbook for {}: {} bids, {} asks", 
                    symbol, orderbook.bids.len(), orderbook.asks.len());
                Ok(orderbook)
            }
            Err(e) => {
                error!("Failed to get Binance orderbook for {}: {}", symbol, e);
                Err(TradingError::DataError(format!("Binance orderbook error: {}", e)))
            }
        }
    }

    /// Get kline/candlestick data
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u16>,
    ) -> TradingResult<Vec<BinanceKline>> {
        let limit = limit.unwrap_or(500);
        let endpoint = format!(
            "klines?symbol={}&interval={}&limit={}", 
            symbol.to_uppercase(), 
            interval, 
            limit
        );

        match self.http_client.get::<Vec<Vec<serde_json::Value>>>(&endpoint).await {
            Ok(raw_klines) => {
                let mut klines = Vec::new();
                
                for raw_kline in raw_klines {
                    if raw_kline.len() >= 11 {
                        let kline = BinanceKline {
                            open_time: raw_kline[0].as_i64().unwrap_or(0),
                            open: raw_kline[1].as_str().unwrap_or("0").to_string(),
                            high: raw_kline[2].as_str().unwrap_or("0").to_string(),
                            low: raw_kline[3].as_str().unwrap_or("0").to_string(),
                            close: raw_kline[4].as_str().unwrap_or("0").to_string(),
                            volume: raw_kline[5].as_str().unwrap_or("0").to_string(),
                            close_time: raw_kline[6].as_i64().unwrap_or(0),
                            quote_asset_volume: raw_kline[7].as_str().unwrap_or("0").to_string(),
                            number_of_trades: raw_kline[8].as_i64().unwrap_or(0),
                            taker_buy_base_asset_volume: raw_kline[9].as_str().unwrap_or("0").to_string(),
                            taker_buy_quote_asset_volume: raw_kline[10].as_str().unwrap_or("0").to_string(),
                            ignore: String::new(),
                        };
                        klines.push(kline);
                    }
                }

                debug!("Retrieved {} klines for {} ({})", klines.len(), symbol, interval);
                Ok(klines)
            }
            Err(e) => {
                error!("Failed to get Binance klines for {}: {}", symbol, e);
                Err(TradingError::DataError(format!("Binance klines error: {}", e)))
            }
        }
    }

    /// Get exchange info
    pub async fn get_exchange_info(&self) -> TradingResult<serde_json::Value> {
        let endpoint = "exchangeInfo";

        match self.http_client.get::<serde_json::Value>(endpoint).await {
            Ok(info) => {
                debug!("Retrieved Binance exchange info");
                Ok(info)
            }
            Err(e) => {
                error!("Failed to get Binance exchange info: {}", e);
                Err(TradingError::DataError(format!("Binance exchange info error: {}", e)))
            }
        }
    }

    /// Check if symbol is supported
    pub fn is_symbol_supported(&self, symbol: &str) -> bool {
        self.config.supported_pairs.contains(&symbol.to_uppercase())
    }

    /// Get WebSocket URL for streaming
    pub fn get_websocket_url(&self) -> &str {
        &self.ws_url
    }
}

#[async_trait]
impl DataFetcher for BinanceClient {
    async fn get_market_data(&self, symbol: &str) -> TradingResult<MarketData> {
        if !self.is_symbol_supported(symbol) {
            return Err(TradingError::DataError(format!(
                "Symbol {} not supported by Binance client", symbol
            )));
        }

        let ticker = self.get_24hr_ticker(symbol).await?;
        
        let price = ticker.last_price.parse::<f64>()
            .map_err(|e| TradingError::DataError(format!("Invalid price format: {}", e)))?;
        
        let volume = ticker.volume.parse::<f64>()
            .map_err(|e| TradingError::DataError(format!("Invalid volume format: {}", e)))?;
        
        let bid = ticker.bid_price.parse::<f64>().ok();
        let ask = ticker.ask_price.parse::<f64>().ok();

        Ok(MarketData {
            symbol: symbol.to_string(),
            price,
            volume,
            bid,
            ask,
            timestamp: Utc::now(),
            source: DataSource::Binance,
        })
    }

    async fn get_orderbook(&self, symbol: &str) -> TradingResult<OrderBook> {
        if !self.is_symbol_supported(symbol) {
            return Err(TradingError::DataError(format!(
                "Symbol {} not supported by Binance client", symbol
            )));
        }

        let binance_orderbook = self.get_order_book_depth(symbol, Some(20)).await?;
        
        let bids: Vec<PriceLevel> = binance_orderbook.bids
            .into_iter()
            .filter_map(|bid| {
                if bid.len() >= 2 {
                    let price = bid[0].parse::<f64>().ok()?;
                    let size = bid[1].parse::<f64>().ok()?;
                    Some(PriceLevel { price, size })
                } else {
                    None
                }
            })
            .collect();

        let asks: Vec<PriceLevel> = binance_orderbook.asks
            .into_iter()
            .filter_map(|ask| {
                if ask.len() >= 2 {
                    let price = ask[0].parse::<f64>().ok()?;
                    let size = ask[1].parse::<f64>().ok()?;
                    Some(PriceLevel { price, size })
                } else {
                    None
                }
            })
            .collect();

        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn subscribe_to_ticker(&self, symbol: &str) -> TradingResult<()> {
        info!("Setting up Binance WebSocket subscription for {}", symbol);
        // WebSocket implementation would go here
        // For now, just log the intent
        debug!("WebSocket URL: {}", self.ws_url);
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.config.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn create_test_config() -> ExchangeConfig {
        ExchangeConfig {
            name: "Binance".to_string(),
            api_key: "test_key".to_string(),
            api_secret: "test_secret".to_string(),
            sandbox: true,
            rate_limit_per_second: 10,
            enabled: true,
            supported_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            endpoints: None,
        }
    }

    #[tokio::test]
    async fn test_binance_client_creation() {
        let config = create_test_config();
        let client = BinanceClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_symbol_support() {
        let config = create_test_config();
        let client = BinanceClient::new(config).unwrap();
        
        assert!(client.is_symbol_supported("BTCUSDT"));
        assert!(client.is_symbol_supported("btcusdt")); // Case insensitive
        assert!(!client.is_symbol_supported("INVALID"));
    }

    #[test]
    fn test_websocket_url() {
        let config = create_test_config();
        let client = BinanceClient::new(config).unwrap();
        
        let ws_url = client.get_websocket_url();
        assert!(ws_url.starts_with("wss://"));
        assert!(ws_url.contains("testnet")); // Should be testnet for sandbox
    }
}

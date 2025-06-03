use crate::data_fetcher::{DataFetcher, OrderBook, PriceLevel};
use crate::models::{DataSource, MarketData, TradingError, TradingResult};
use crate::utils::http_client::HttpClient;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Meteora DLMM client for dynamic liquidity market making
pub struct MeteoraClient {
    http_client: HttpClient,
    api_base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraPool {
    pub address: String,
    pub name: String,
    pub mint_x: String,
    pub mint_y: String,
    pub reserve_x: String,
    pub reserve_y: String,
    pub reserve_x_amount: u64,
    pub reserve_y_amount: u64,
    pub bin_step: u16,
    pub base_fee_percentage: String,
    pub max_fee_percentage: String,
    pub protocol_fee_percentage: String,
    pub liquidity: String,
    pub reward_infos: Vec<MeteoraRewardInfo>,
    pub farm_infos: Vec<MeteoraFarmInfo>,
    pub total_fee_24h: f64,
    pub total_volume_24h: f64,
    pub fee_apr: f64,
    pub today_fee: f64,
    pub trade_volume_24h: f64,
    pub cumulative_trade_volume: String,
    pub cumulative_fee_volume: String,
    pub current_price: f64,
    pub apr: f64,
    pub apy: f64,
    pub hide: bool,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraRewardInfo {
    pub mint: String,
    pub symbol: String,
    pub decimals: u8,
    pub reward_per_second: String,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraFarmInfo {
    pub farm_address: String,
    pub reward_mint: String,
    pub reward_symbol: String,
    pub reward_decimals: u8,
    pub base_reward_per_second: String,
    pub boosted_reward_per_second: String,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraPoolStats {
    pub address: String,
    pub total_volume_24h: f64,
    pub total_fee_24h: f64,
    pub total_volume_7d: f64,
    pub total_fee_7d: f64,
    pub fee_apr: f64,
    pub liquidity: String,
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraActiveBin {
    pub bin_id: i32,
    pub price: f64,
    pub liquidity_x: String,
    pub liquidity_y: String,
    pub supply: String,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraPoolBins {
    pub pool_address: String,
    pub active_bin: MeteoraActiveBin,
    pub bins: Vec<MeteoraActiveBin>,
}

#[derive(Debug, Serialize)]
pub struct MeteoraSwapRequest {
    pub pool_address: String,
    pub token_in_mint: String,
    pub token_out_mint: String,
    pub amount_in: u64,
    pub slippage_bps: u16,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraSwapResponse {
    pub amount_in: String,
    pub amount_out: String,
    pub min_amount_out: String,
    pub price_impact: f64,
    pub fee: String,
    pub bin_arrays_to_initialize: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MeteoraTokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
    pub tags: Vec<String>,
}

impl MeteoraClient {
    pub fn new() -> TradingResult<Self> {
        let api_base_url = "https://dlmm-api.meteora.ag".to_string();
        
        let http_client = HttpClient::new(api_base_url.clone())
            .map_err(|e| TradingError::NetworkError(e.to_string()))?
            .with_rate_limit(10); // Conservative rate limit

        info!("✅ Meteora DLMM client initialized");
        debug!("Meteora API base URL: {}", api_base_url);

        Ok(Self {
            http_client,
            api_base_url,
        })
    }

    /// Get all DLMM pools
    pub async fn get_pools(&self) -> TradingResult<Vec<MeteoraPool>> {
        let endpoint = "pair/all";

        match self.http_client.get::<Vec<MeteoraPool>>(endpoint).await {
            Ok(pools) => {
                info!("Retrieved {} Meteora DLMM pools", pools.len());
                Ok(pools)
            }
            Err(e) => {
                error!("Failed to get Meteora pools: {}", e);
                Err(TradingError::DataError(format!("Meteora pools error: {}", e)))
            }
        }
    }

    /// Get pool information by address
    pub async fn get_pool_info(&self, pool_address: &str) -> TradingResult<MeteoraPool> {
        let endpoint = format!("pair/{}", pool_address);

        match self.http_client.get::<MeteoraPool>(&endpoint).await {
            Ok(pool) => {
                debug!("Retrieved Meteora pool info for {}", pool_address);
                Ok(pool)
            }
            Err(e) => {
                error!("Failed to get Meteora pool info for {}: {}", pool_address, e);
                Err(TradingError::DataError(format!("Meteora pool info error: {}", e)))
            }
        }
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self, pool_address: &str) -> TradingResult<MeteoraPoolStats> {
        let endpoint = format!("pair/{}/stats", pool_address);

        match self.http_client.get::<MeteoraPoolStats>(&endpoint).await {
            Ok(stats) => {
                debug!("Retrieved Meteora pool stats for {}", pool_address);
                Ok(stats)
            }
            Err(e) => {
                error!("Failed to get Meteora pool stats for {}: {}", pool_address, e);
                Err(TradingError::DataError(format!("Meteora pool stats error: {}", e)))
            }
        }
    }

    /// Get pool bins (liquidity distribution)
    pub async fn get_pool_bins(&self, pool_address: &str) -> TradingResult<MeteoraPoolBins> {
        let endpoint = format!("pair/{}/bins", pool_address);

        match self.http_client.get::<MeteoraPoolBins>(&endpoint).await {
            Ok(bins) => {
                debug!("Retrieved {} bins for Meteora pool {}", bins.bins.len(), pool_address);
                Ok(bins)
            }
            Err(e) => {
                error!("Failed to get Meteora pool bins for {}: {}", pool_address, e);
                Err(TradingError::DataError(format!("Meteora pool bins error: {}", e)))
            }
        }
    }

    /// Get swap quote
    pub async fn get_swap_quote(&self, request: MeteoraSwapRequest) -> TradingResult<MeteoraSwapResponse> {
        let endpoint = "swap/quote";

        match self.http_client.post::<MeteoraSwapResponse, _>(endpoint, &request).await {
            Ok(response) => {
                debug!(
                    "Meteora swap quote: {} -> {} (impact: {:.2}%)",
                    request.amount_in,
                    response.amount_out,
                    response.price_impact
                );
                Ok(response)
            }
            Err(e) => {
                error!("Failed to get Meteora swap quote: {}", e);
                Err(TradingError::DataError(format!("Meteora swap quote error: {}", e)))
            }
        }
    }

    /// Find pool by token pair
    pub async fn find_pool_by_tokens(&self, mint_x: &str, mint_y: &str) -> TradingResult<Option<MeteoraPool>> {
        let pools = self.get_pools().await?;
        
        for pool in pools {
            if (pool.mint_x == mint_x && pool.mint_y == mint_y) ||
               (pool.mint_x == mint_y && pool.mint_y == mint_x) {
                return Ok(Some(pool));
            }
        }
        
        Ok(None)
    }

    /// Get token information
    pub async fn get_token_info(&self, mint: &str) -> TradingResult<MeteoraTokenInfo> {
        let endpoint = format!("token/{}", mint);

        match self.http_client.get::<MeteoraTokenInfo>(&endpoint).await {
            Ok(token) => {
                debug!("Retrieved Meteora token info for {}: {}", mint, token.symbol);
                Ok(token)
            }
            Err(e) => {
                error!("Failed to get Meteora token info for {}: {}", mint, e);
                Err(TradingError::DataError(format!("Meteora token info error: {}", e)))
            }
        }
    }

    /// Calculate effective price from reserves
    pub fn calculate_pool_price(&self, pool: &MeteoraPool, invert: bool) -> f64 {
        if pool.reserve_x_amount > 0 && pool.reserve_y_amount > 0 {
            let price = pool.reserve_y_amount as f64 / pool.reserve_x_amount as f64;
            if invert { 1.0 / price } else { price }
        } else {
            pool.current_price
        }
    }

    /// Get pools with highest APR
    pub async fn get_high_apr_pools(&self, min_apr: f64) -> TradingResult<Vec<MeteoraPool>> {
        let pools = self.get_pools().await?;
        
        let high_apr_pools: Vec<MeteoraPool> = pools
            .into_iter()
            .filter(|pool| pool.apr >= min_apr && !pool.hide)
            .collect();

        info!("Found {} Meteora pools with APR >= {:.2}%", high_apr_pools.len(), min_apr);
        Ok(high_apr_pools)
    }

    /// Get pools by volume
    pub async fn get_pools_by_volume(&self, min_volume_24h: f64) -> TradingResult<Vec<MeteoraPool>> {
        let pools = self.get_pools().await?;
        
        let high_volume_pools: Vec<MeteoraPool> = pools
            .into_iter()
            .filter(|pool| pool.total_volume_24h >= min_volume_24h && !pool.hide)
            .collect();

        info!("Found {} Meteora pools with 24h volume >= ${:.2}", high_volume_pools.len(), min_volume_24h);
        Ok(high_volume_pools)
    }
}

#[async_trait]
impl DataFetcher for MeteoraClient {
    async fn get_market_data(&self, symbol: &str) -> TradingResult<MarketData> {
        // Symbol format: "POOL_ADDRESS" or "TOKEN_X/TOKEN_Y"
        if symbol.len() > 40 {
            // Looks like a pool address
            let pool = self.get_pool_info(symbol).await?;
            
            Ok(MarketData {
                symbol: format!("{}/{}", pool.mint_x, pool.mint_y),
                price: pool.current_price,
                volume: pool.total_volume_24h,
                bid: None,
                ask: None,
                timestamp: Utc::now(),
                source: DataSource::Solana,
            })
        } else {
            // Token pair format
            let parts: Vec<&str> = symbol.split('/').collect();
            if parts.len() != 2 {
                return Err(TradingError::DataError(
                    "Invalid symbol format. Use TOKEN_X/TOKEN_Y or POOL_ADDRESS".to_string()
                ));
            }

            match self.find_pool_by_tokens(parts[0], parts[1]).await? {
                Some(pool) => {
                    let price = self.calculate_pool_price(&pool, false);
                    
                    Ok(MarketData {
                        symbol: symbol.to_string(),
                        price,
                        volume: pool.total_volume_24h,
                        bid: None,
                        ask: None,
                        timestamp: Utc::now(),
                        source: DataSource::Solana,
                    })
                }
                None => {
                    Err(TradingError::DataError(format!("No Meteora pool found for {}", symbol)))
                }
            }
        }
    }

    async fn get_orderbook(&self, symbol: &str) -> TradingResult<OrderBook> {
        // Use DLMM bins as a proxy for orderbook
        let pool_address = if symbol.len() > 40 {
            symbol.to_string()
        } else {
            let parts: Vec<&str> = symbol.split('/').collect();
            if parts.len() != 2 {
                return Err(TradingError::DataError(
                    "Invalid symbol format for orderbook".to_string()
                ));
            }
            
            match self.find_pool_by_tokens(parts[0], parts[1]).await? {
                Some(pool) => pool.address,
                None => {
                    return Err(TradingError::DataError(format!("No pool found for {}", symbol)));
                }
            }
        };

        let bins = self.get_pool_bins(&pool_address).await?;
        
        // Convert bins to orderbook-like structure
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        let active_price = bins.active_bin.price;
        
        for bin in &bins.bins {
            let liquidity_x: f64 = bin.liquidity_x.parse().unwrap_or(0.0);
            let liquidity_y: f64 = bin.liquidity_y.parse().unwrap_or(0.0);
            
            if bin.price <= active_price && liquidity_x > 0.0 {
                // Below current price = bids
                bids.push(PriceLevel {
                    price: bin.price,
                    size: liquidity_x,
                });
            } else if bin.price > active_price && liquidity_y > 0.0 {
                // Above current price = asks
                asks.push(PriceLevel {
                    price: bin.price,
                    size: liquidity_y,
                });
            }
        }

        // Sort bids (highest first) and asks (lowest first)
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        Ok(OrderBook {
            symbol: symbol.to_string(),
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn subscribe_to_ticker(&self, symbol: &str) -> TradingResult<()> {
        info!("Meteora doesn't support real-time subscriptions for {}", symbol);
        // Would need to implement WebSocket or polling mechanism
        Ok(())
    }

    fn get_name(&self) -> &str {
        "Meteora DLMM"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_meteora_client_creation() {
        let client = MeteoraClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_calculate_pool_price() {
        let client = MeteoraClient::new().unwrap();
        let pool = MeteoraPool {
            address: "test".to_string(),
            name: "Test Pool".to_string(),
            mint_x: "mint_x".to_string(),
            mint_y: "mint_y".to_string(),
            reserve_x: "1000".to_string(),
            reserve_y: "2000".to_string(),
            reserve_x_amount: 1000,
            reserve_y_amount: 2000,
            bin_step: 25,
            base_fee_percentage: "0.25".to_string(),
            max_fee_percentage: "2.5".to_string(),
            protocol_fee_percentage: "0.1".to_string(),
            liquidity: "1000000".to_string(),
            reward_infos: vec![],
            farm_infos: vec![],
            total_fee_24h: 100.0,
            total_volume_24h: 10000.0,
            fee_apr: 5.0,
            today_fee: 50.0,
            trade_volume_24h: 8000.0,
            cumulative_trade_volume: "1000000".to_string(),
            cumulative_fee_volume: "10000".to_string(),
            current_price: 2.0,
            apr: 10.0,
            apy: 10.5,
            hide: false,
        };

        let price = client.calculate_pool_price(&pool, false);
        assert_eq!(price, 2.0); // 2000 / 1000 = 2.0

        let inverted_price = client.calculate_pool_price(&pool, true);
        assert_eq!(inverted_price, 0.5); // 1 / 2.0 = 0.5
    }

    #[test]
    fn test_swap_request_serialization() {
        let request = MeteoraSwapRequest {
            pool_address: "test_pool".to_string(),
            token_in_mint: "token_in".to_string(),
            token_out_mint: "token_out".to_string(),
            amount_in: 1000000,
            slippage_bps: 50,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("pool_address"));
        assert!(json.contains("token_in_mint"));
        assert!(json.contains("amount_in"));
    }
}

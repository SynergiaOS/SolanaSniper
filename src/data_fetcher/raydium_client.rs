use crate::data_fetcher::{DataFetcher, OrderBook};
use crate::models::{DataSource, MarketData, TradingError, TradingResult};
use crate::utils::http_client::HttpClient;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Raydium DEX client for Solana AMM and CLMM pools
pub struct RaydiumClient {
    http_client: HttpClient,
    api_base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumPoolInfo {
    pub id: String,
    #[serde(rename = "baseMint")]
    pub base_mint: String,
    #[serde(rename = "quoteMint")]
    pub quote_mint: String,
    #[serde(rename = "lpMint")]
    pub lp_mint: String,
    #[serde(rename = "baseDecimals")]
    pub base_decimals: u8,
    #[serde(rename = "quoteDecimals")]
    pub quote_decimals: u8,
    #[serde(rename = "lpDecimals")]
    pub lp_decimals: u8,
    pub version: u8,
    #[serde(rename = "programId")]
    pub program_id: String,
    pub authority: String,
    #[serde(rename = "openOrders")]
    pub open_orders: String,
    #[serde(rename = "targetOrders")]
    pub target_orders: String,
    #[serde(rename = "baseVault")]
    pub base_vault: String,
    #[serde(rename = "quoteVault")]
    pub quote_vault: String,
    #[serde(rename = "withdrawQueue")]
    pub withdraw_queue: String,
    #[serde(rename = "lpVault")]
    pub lp_vault: String,
    #[serde(rename = "marketVersion")]
    pub market_version: u8,
    #[serde(rename = "marketProgramId")]
    pub market_program_id: String,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "marketAuthority")]
    pub market_authority: String,
    #[serde(rename = "marketBaseVault")]
    pub market_base_vault: String,
    #[serde(rename = "marketQuoteVault")]
    pub market_quote_vault: String,
    #[serde(rename = "marketBids")]
    pub market_bids: String,
    #[serde(rename = "marketAsks")]
    pub market_asks: String,
    #[serde(rename = "marketEventQueue")]
    pub market_event_queue: String,
    #[serde(rename = "lookupTableAccount")]
    pub lookup_table_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumPoolStats {
    pub id: String,
    #[serde(rename = "baseMint")]
    pub base_mint: String,
    #[serde(rename = "quoteMint")]
    pub quote_mint: String,
    #[serde(rename = "lpMint")]
    pub lp_mint: String,
    #[serde(rename = "baseReserve")]
    pub base_reserve: String,
    #[serde(rename = "quoteReserve")]
    pub quote_reserve: String,
    #[serde(rename = "lpSupply")]
    pub lp_supply: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumPriceInfo {
    pub id: String,
    #[serde(rename = "mintSymbol")]
    pub mint_symbol: String,
    #[serde(rename = "vsToken")]
    pub vs_token: String,
    #[serde(rename = "vsTokenSymbol")]
    pub vs_token_symbol: String,
    pub price: f64,
    #[serde(rename = "24hChange")]
    pub change_24h: Option<f64>,
    #[serde(rename = "24hVolume")]
    pub volume_24h: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RaydiumSwapRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "txVersion")]
    pub tx_version: String,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumSwapResponse {
    pub id: String,
    pub success: bool,
    pub version: String,
    pub data: RaydiumSwapData,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumSwapData {
    #[serde(rename = "swapType")]
    pub swap_type: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inputAmount")]
    pub input_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outputAmount")]
    pub output_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: f64,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RaydiumRoutePlan>,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumRoutePlan {
    #[serde(rename = "poolId")]
    pub pool_id: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
    #[serde(rename = "feeRate")]
    pub fee_rate: f64,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
}

impl RaydiumClient {
    pub fn new() -> TradingResult<Self> {
        let api_base_url = "https://api-v3.raydium.io".to_string();
        
        let http_client = HttpClient::new(api_base_url.clone())
            .map_err(|e| TradingError::NetworkError(e.to_string()))?
            .with_rate_limit(5); // Conservative rate limit for Raydium

        info!("✅ Raydium client initialized");
        debug!("Raydium API base URL: {}", api_base_url);

        Ok(Self {
            http_client,
            api_base_url,
        })
    }

    /// Get all available pools
    pub async fn get_pools(&self) -> TradingResult<Vec<RaydiumPoolInfo>> {
        let endpoint = "pools/info/list";

        match self.http_client.get::<Vec<RaydiumPoolInfo>>(endpoint).await {
            Ok(pools) => {
                info!("Retrieved {} Raydium pools", pools.len());
                Ok(pools)
            }
            Err(e) => {
                error!("Failed to get Raydium pools: {}", e);
                Err(TradingError::DataError(format!("Raydium pools error: {}", e)))
            }
        }
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self, pool_id: &str) -> TradingResult<RaydiumPoolStats> {
        let endpoint = format!("pools/info/ids?ids={}", pool_id);

        match self.http_client.get::<Vec<RaydiumPoolStats>>(&endpoint).await {
            Ok(mut stats) => {
                if let Some(pool_stats) = stats.pop() {
                    debug!("Retrieved Raydium pool stats for {}", pool_id);
                    Ok(pool_stats)
                } else {
                    Err(TradingError::DataError(format!("Pool not found: {}", pool_id)))
                }
            }
            Err(e) => {
                error!("Failed to get Raydium pool stats for {}: {}", pool_id, e);
                Err(TradingError::DataError(format!("Raydium pool stats error: {}", e)))
            }
        }
    }

    /// Get price information
    pub async fn get_price_info(&self, mint: &str) -> TradingResult<RaydiumPriceInfo> {
        let endpoint = format!("mint/price?mints={}", mint);

        match self.http_client.get::<HashMap<String, RaydiumPriceInfo>>(&endpoint).await {
            Ok(mut prices) => {
                if let Some(price_info) = prices.remove(mint) {
                    debug!("Retrieved Raydium price for {}: ${}", mint, price_info.price);
                    Ok(price_info)
                } else {
                    Err(TradingError::DataError(format!("Price not found for mint: {}", mint)))
                }
            }
            Err(e) => {
                error!("Failed to get Raydium price for {}: {}", mint, e);
                Err(TradingError::DataError(format!("Raydium price error: {}", e)))
            }
        }
    }

    /// Get swap quote
    pub async fn get_swap_quote(&self, request: RaydiumSwapRequest) -> TradingResult<RaydiumSwapResponse> {
        let endpoint = "compute/swap-base-in";

        match self.http_client.post::<RaydiumSwapResponse, _>(endpoint, &request).await {
            Ok(response) => {
                if response.success {
                    debug!(
                        "Raydium swap quote: {} {} -> {} {} (impact: {:.2}%)",
                        request.amount,
                        request.input_mint,
                        response.data.output_amount,
                        request.output_mint,
                        response.data.price_impact_pct
                    );
                    Ok(response)
                } else {
                    Err(TradingError::DataError("Raydium swap quote failed".to_string()))
                }
            }
            Err(e) => {
                error!("Failed to get Raydium swap quote: {}", e);
                Err(TradingError::DataError(format!("Raydium swap quote error: {}", e)))
            }
        }
    }

    /// Calculate price from pool reserves
    pub fn calculate_pool_price(&self, pool_stats: &RaydiumPoolStats) -> TradingResult<f64> {
        let base_reserve: f64 = pool_stats.base_reserve.parse()
            .map_err(|e| TradingError::DataError(format!("Invalid base reserve: {}", e)))?;
        
        let quote_reserve: f64 = pool_stats.quote_reserve.parse()
            .map_err(|e| TradingError::DataError(format!("Invalid quote reserve: {}", e)))?;

        if base_reserve > 0.0 {
            Ok(quote_reserve / base_reserve)
        } else {
            Err(TradingError::DataError("Zero base reserve".to_string()))
        }
    }

    /// Find pool by token pair
    pub async fn find_pool_by_tokens(&self, base_mint: &str, quote_mint: &str) -> TradingResult<Option<RaydiumPoolInfo>> {
        let pools = self.get_pools().await?;
        
        for pool in pools {
            if (pool.base_mint == base_mint && pool.quote_mint == quote_mint) ||
               (pool.base_mint == quote_mint && pool.quote_mint == base_mint) {
                return Ok(Some(pool));
            }
        }
        
        Ok(None)
    }
}

#[async_trait]
impl DataFetcher for RaydiumClient {
    async fn get_market_data(&self, symbol: &str) -> TradingResult<MarketData> {
        // Symbol format: "BASE_MINT/QUOTE_MINT" or just "MINT" for price vs USDC
        let parts: Vec<&str> = symbol.split('/').collect();
        
        if parts.len() == 1 {
            // Single mint - get price info
            match self.get_price_info(parts[0]).await {
                Ok(price_info) => {
                    Ok(MarketData {
                        symbol: symbol.to_string(),
                        price: price_info.price,
                        volume: price_info.volume_24h.unwrap_or(0.0),
                        bid: None,
                        ask: None,
                        timestamp: Utc::now(),
                        source: DataSource::Solana,
                    })
                }
                Err(e) => {
                    warn!("Failed to get Raydium price for {}: {}", symbol, e);
                    Err(e)
                }
            }
        } else if parts.len() == 2 {
            // Token pair - find pool and calculate price
            match self.find_pool_by_tokens(parts[0], parts[1]).await? {
                Some(pool) => {
                    let pool_stats = self.get_pool_stats(&pool.id).await?;
                    let price = self.calculate_pool_price(&pool_stats)?;
                    
                    Ok(MarketData {
                        symbol: symbol.to_string(),
                        price,
                        volume: 0.0, // Would need additional API call for volume
                        bid: None,
                        ask: None,
                        timestamp: Utc::now(),
                        source: DataSource::Solana,
                    })
                }
                None => {
                    Err(TradingError::DataError(format!("No Raydium pool found for {}", symbol)))
                }
            }
        } else {
            Err(TradingError::DataError(
                "Invalid symbol format. Use MINT or BASE_MINT/QUOTE_MINT".to_string()
            ))
        }
    }

    async fn get_orderbook(&self, _symbol: &str) -> TradingResult<OrderBook> {
        // Raydium is an AMM, not an orderbook-based exchange
        warn!("Raydium is an AMM - no traditional orderbook available");
        Err(TradingError::DataError(
            "Raydium AMM doesn't support orderbook data".to_string()
        ))
    }

    async fn subscribe_to_ticker(&self, symbol: &str) -> TradingResult<()> {
        info!("Raydium doesn't support real-time subscriptions for {}", symbol);
        // Would need to implement WebSocket or polling mechanism
        Ok(())
    }

    fn get_name(&self) -> &str {
        "Raydium"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_raydium_client_creation() {
        let client = RaydiumClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_calculate_pool_price() {
        let client = RaydiumClient::new().unwrap();
        let pool_stats = RaydiumPoolStats {
            id: "test".to_string(),
            base_mint: "base".to_string(),
            quote_mint: "quote".to_string(),
            lp_mint: "lp".to_string(),
            base_reserve: "1000000000".to_string(), // 1000 tokens
            quote_reserve: "50000000000".to_string(), // 50000 tokens
            lp_supply: "1000000".to_string(),
            start_time: "0".to_string(),
        };

        let price = client.calculate_pool_price(&pool_stats).unwrap();
        assert_eq!(price, 50.0); // 50000 / 1000 = 50
    }

    #[test]
    fn test_swap_request_serialization() {
        let request = RaydiumSwapRequest {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: 1000000000,
            slippage_bps: 50,
            tx_version: "V0".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("inputMint"));
        assert!(json.contains("outputMint"));
    }
}

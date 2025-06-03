use crate::data_fetcher::{DataFetcher, OrderBook};
use crate::models::{DataSource, MarketData, TradingError, TradingResult};
use crate::config::JupiterConfig;
use crate::utils::http_client::HttpClient;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Jupiter API client for Solana DEX aggregation
pub struct JupiterClient {
    config: JupiterConfig,
    http_client: HttpClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JupiterQuoteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "onlyDirectRoutes", skip_serializing_if = "Option::is_none")]
    pub only_direct_routes: Option<bool>,
    #[serde(rename = "asLegacyTransaction", skip_serializing_if = "Option::is_none")]
    pub as_legacy_transaction: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JupiterQuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "platformFee")]
    pub platform_fee: Option<PlatformFee>,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
    #[serde(rename = "contextSlot")]
    pub context_slot: u64,
    #[serde(rename = "timeTaken")]
    pub time_taken: f64,
}

#[derive(Debug, Deserialize)]
pub struct PlatformFee {
    pub amount: String,
    #[serde(rename = "feeBps")]
    pub fee_bps: u16,
}

#[derive(Debug, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

#[derive(Debug, Deserialize)]
pub struct JupiterTokenInfo {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub tags: Vec<String>,
    #[serde(rename = "daily_volume")]
    pub daily_volume: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct JupiterPriceData {
    pub id: String,
    #[serde(rename = "mintSymbol")]
    pub mint_symbol: String,
    #[serde(rename = "vsToken")]
    pub vs_token: String,
    #[serde(rename = "vsTokenSymbol")]
    pub vs_token_symbol: String,
    pub price: f64,
}

impl JupiterClient {
    pub fn new(config: JupiterConfig) -> TradingResult<Self> {
        let http_client = HttpClient::new(config.api_url.clone())
            .map_err(|e| TradingError::NetworkError(e.to_string()))?;

        info!("✅ Jupiter client initialized");
        debug!("Jupiter API base URL: {}", config.api_url);
        debug!("Max retries: {}", config.max_retries);

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Get quote for token swap
    pub async fn get_quote(&self, request: JupiterQuoteRequest) -> TradingResult<JupiterQuoteResponse> {
        let endpoint = "quote";
        
        // Convert request to query parameters
        let mut params = vec![
            ("inputMint", request.input_mint.clone()),
            ("outputMint", request.output_mint.clone()),
            ("amount", request.amount.to_string()),
            ("slippageBps", request.slippage_bps.to_string()),
        ];

        if let Some(only_direct) = request.only_direct_routes {
            params.push(("onlyDirectRoutes", only_direct.to_string()));
        }

        if let Some(as_legacy) = request.as_legacy_transaction {
            params.push(("asLegacyTransaction", as_legacy.to_string()));
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let full_endpoint = format!("{}?{}", endpoint, query_string);

        match self.http_client.get::<JupiterQuoteResponse>(&full_endpoint).await {
            Ok(quote) => {
                debug!(
                    "Jupiter quote: {} {} -> {} {} (price impact: {}%)",
                    request.amount,
                    request.input_mint,
                    quote.out_amount,
                    quote.output_mint,
                    quote.price_impact_pct
                );
                Ok(quote)
            }
            Err(e) => {
                error!("Failed to get Jupiter quote: {}", e);
                Err(TradingError::DataError(format!("Jupiter quote error: {}", e)))
            }
        }
    }

    /// Get all available tokens
    pub async fn get_tokens(&self) -> TradingResult<Vec<JupiterTokenInfo>> {
        let endpoint = "tokens";

        match self.http_client.get::<Vec<JupiterTokenInfo>>(endpoint).await {
            Ok(tokens) => {
                info!("Retrieved {} tokens from Jupiter", tokens.len());
                Ok(tokens)
            }
            Err(e) => {
                error!("Failed to get Jupiter tokens: {}", e);
                Err(TradingError::DataError(format!("Jupiter tokens error: {}", e)))
            }
        }
    }

    /// Get price data for tokens
    pub async fn get_price(&self, ids: &[String]) -> TradingResult<HashMap<String, JupiterPriceData>> {
        let endpoint = "price";
        let ids_param = ids.join(",");
        let full_endpoint = format!("{}?ids={}", endpoint, ids_param);

        match self.http_client.get::<HashMap<String, JupiterPriceData>>(&full_endpoint).await {
            Ok(prices) => {
                debug!("Retrieved prices for {} tokens", prices.len());
                Ok(prices)
            }
            Err(e) => {
                error!("Failed to get Jupiter prices: {}", e);
                Err(TradingError::DataError(format!("Jupiter price error: {}", e)))
            }
        }
    }

    /// Calculate effective price from quote
    pub fn calculate_effective_price(&self, quote: &JupiterQuoteResponse) -> f64 {
        let in_amount: f64 = quote.in_amount.parse().unwrap_or(0.0);
        let out_amount: f64 = quote.out_amount.parse().unwrap_or(0.0);
        
        if in_amount > 0.0 {
            out_amount / in_amount
        } else {
            0.0
        }
    }

    /// Get best route for a token pair
    pub async fn get_best_route(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> TradingResult<JupiterQuoteResponse> {
        let request = JupiterQuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount,
            slippage_bps,
            only_direct_routes: Some(false), // Allow multi-hop routes for better prices
            as_legacy_transaction: Some(false),
        };

        self.get_quote(request).await
    }
}

#[async_trait]
impl DataFetcher for JupiterClient {
    async fn get_market_data(&self, symbol: &str) -> TradingResult<MarketData> {
        // For Jupiter, symbol should be in format "INPUT_MINT/OUTPUT_MINT"
        // For simplicity, let's assume we're getting SOL price in USDC
        let parts: Vec<&str> = symbol.split('/').collect();
        if parts.len() != 2 {
            return Err(TradingError::DataError(
                "Symbol must be in format INPUT_MINT/OUTPUT_MINT".to_string()
            ));
        }

        let input_mint = parts[0];
        let output_mint = parts[1];
        
        // Get quote for 1 SOL (1e9 lamports) to USDC
        let amount = 1_000_000_000; // 1 SOL in lamports
        
        match self.get_best_route(input_mint, output_mint, amount, 50).await {
            Ok(quote) => {
                let price = self.calculate_effective_price(&quote);
                
                Ok(MarketData {
                    symbol: symbol.to_string(),
                    price,
                    volume: 0.0, // Jupiter doesn't provide volume directly
                    bid: None,
                    ask: None,
                    timestamp: Utc::now(),
                    source: DataSource::Solana,
                })
            }
            Err(e) => {
                warn!("Failed to get market data from Jupiter for {}: {}", symbol, e);
                Err(e)
            }
        }
    }

    async fn get_orderbook(&self, _symbol: &str) -> TradingResult<OrderBook> {
        // Jupiter is an aggregator, not an orderbook-based exchange
        warn!("Jupiter doesn't provide orderbook data - it's a DEX aggregator");
        Err(TradingError::DataError(
            "Jupiter doesn't support orderbook data".to_string()
        ))
    }

    async fn subscribe_to_ticker(&self, symbol: &str) -> TradingResult<()> {
        info!("Jupiter doesn't support real-time subscriptions for {}", symbol);
        // Jupiter doesn't have WebSocket support, would need to poll
        Ok(())
    }

    fn get_name(&self) -> &str {
        "Jupiter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jupiter_client_creation() {
        let config = JupiterConfig {
            api_url: "https://quote-api.jup.ag/v6".to_string(),
            swap_url: "https://quote-api.jup.ag/v6/swap".to_string(),
            price_url: "https://price.jup.ag/v4/price".to_string(),
            timeout_seconds: 10,
            max_retries: 3,
        };
        let client = JupiterClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_calculate_effective_price() {
        let config = JupiterConfig {
            api_url: "https://quote-api.jup.ag/v6".to_string(),
            swap_url: "https://quote-api.jup.ag/v6/swap".to_string(),
            price_url: "https://price.jup.ag/v4/price".to_string(),
            timeout_seconds: 10,
            max_retries: 3,
        };
        let client = JupiterClient::new(config).unwrap();
        let quote = JupiterQuoteResponse {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            in_amount: "1000000000".to_string(), // 1 SOL
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            out_amount: "100000000".to_string(), // 100 USDC
            other_amount_threshold: "99000000".to_string(),
            swap_mode: "ExactIn".to_string(),
            slippage_bps: 50,
            platform_fee: None,
            price_impact_pct: "0.1".to_string(),
            route_plan: vec![],
            context_slot: 12345,
            time_taken: 0.5,
        };

        let price = client.calculate_effective_price(&quote);
        assert_eq!(price, 0.1); // 100 USDC / 1000 SOL units = 0.1
    }
}

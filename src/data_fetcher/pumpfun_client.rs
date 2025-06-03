use crate::data_fetcher::{DataFetcher, OrderBook};
use crate::models::{DataSource, MarketData, TradingError, TradingResult};
use crate::utils::http_client::HttpClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// Pump.fun client for meme token launches and trading
pub struct PumpFunClient {
    http_client: HttpClient,
    api_base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PumpFunToken {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_uri: String,
    pub metadata_uri: String,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub bonding_curve: String,
    pub associated_bonding_curve: String,
    pub creator: String,
    pub created_timestamp: i64,
    pub raydium_pool: Option<String>,
    pub complete: bool,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub hidden: bool,
    pub total_supply: u64,
    pub website: Option<String>,
    pub show_name: bool,
    pub king_of_the_hill_timestamp: Option<i64>,
    pub market_cap: f64,
    pub reply_count: u32,
    pub last_reply: Option<i64>,
    pub nsfw: bool,
    pub market_id: Option<String>,
    pub inverted: Option<bool>,
    pub is_currently_live: bool,
    pub username: Option<String>,
    pub profile_image: Option<String>,
    pub usd_market_cap: f64,
}

#[derive(Debug, Deserialize)]
pub struct PumpFunTrade {
    pub signature: String,
    pub mint: String,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: String,
    pub timestamp: i64,
    pub tx_index: u32,
    pub username: Option<String>,
    pub profile_image: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PumpFunTokenResponse {
    pub tokens: Vec<PumpFunToken>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PumpFunTradeResponse {
    pub trades: Vec<PumpFunTrade>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PumpFunBuyRequest {
    pub mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PumpFunSellRequest {
    pub mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
    pub priority_fee: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PumpFunQuoteResponse {
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub price_impact_pct: f64,
    pub slippage_bps: u16,
}

impl PumpFunClient {
    pub fn new() -> TradingResult<Self> {
        let api_base_url = "https://frontend-api.pump.fun".to_string();
        
        let http_client = HttpClient::new(api_base_url.clone())
            .map_err(|e| TradingError::NetworkError(e.to_string()))?
            .with_rate_limit(10); // Conservative rate limit

        info!("✅ Pump.fun client initialized");
        debug!("Pump.fun API base URL: {}", api_base_url);

        Ok(Self {
            http_client,
            api_base_url,
        })
    }

    /// Get trending tokens
    pub async fn get_trending_tokens(&self, limit: Option<u32>) -> TradingResult<Vec<PumpFunToken>> {
        let limit = limit.unwrap_or(50);
        let endpoint = format!("coins/trending?limit={}", limit);

        match self.http_client.get::<PumpFunTokenResponse>(&endpoint).await {
            Ok(response) => {
                info!("Retrieved {} trending tokens from Pump.fun", response.tokens.len());
                Ok(response.tokens)
            }
            Err(e) => {
                error!("Failed to get trending tokens from Pump.fun: {}", e);
                Err(TradingError::DataError(format!("Pump.fun trending tokens error: {}", e)))
            }
        }
    }

    /// Get recently created tokens
    pub async fn get_recent_tokens(&self, limit: Option<u32>) -> TradingResult<Vec<PumpFunToken>> {
        let limit = limit.unwrap_or(50);
        let endpoint = format!("coins?sort=created_timestamp&order=DESC&limit={}", limit);

        match self.http_client.get::<PumpFunTokenResponse>(&endpoint).await {
            Ok(response) => {
                info!("Retrieved {} recent tokens from Pump.fun", response.tokens.len());
                Ok(response.tokens)
            }
            Err(e) => {
                error!("Failed to get recent tokens from Pump.fun: {}", e);
                Err(TradingError::DataError(format!("Pump.fun recent tokens error: {}", e)))
            }
        }
    }

    /// Get token information by mint
    pub async fn get_token_info(&self, mint: &str) -> TradingResult<PumpFunToken> {
        let endpoint = format!("coins/{}", mint);

        match self.http_client.get::<PumpFunToken>(&endpoint).await {
            Ok(token) => {
                debug!("Retrieved token info for {}: {} ({})", mint, token.name, token.symbol);
                Ok(token)
            }
            Err(e) => {
                error!("Failed to get token info for {}: {}", mint, e);
                Err(TradingError::DataError(format!("Pump.fun token info error: {}", e)))
            }
        }
    }

    /// Get recent trades for a token
    pub async fn get_token_trades(&self, mint: &str, limit: Option<u32>) -> TradingResult<Vec<PumpFunTrade>> {
        let limit = limit.unwrap_or(100);
        let endpoint = format!("coins/{}/trades?limit={}", mint, limit);

        match self.http_client.get::<PumpFunTradeResponse>(&endpoint).await {
            Ok(response) => {
                debug!("Retrieved {} trades for token {}", response.trades.len(), mint);
                Ok(response.trades)
            }
            Err(e) => {
                error!("Failed to get trades for {}: {}", mint, e);
                Err(TradingError::DataError(format!("Pump.fun trades error: {}", e)))
            }
        }
    }

    /// Get buy quote
    pub async fn get_buy_quote(&self, mint: &str, sol_amount: u64, slippage_bps: u16) -> TradingResult<PumpFunQuoteResponse> {
        let endpoint = format!("coins/{}/quote-buy?amount={}&slippage_bps={}", mint, sol_amount, slippage_bps);

        match self.http_client.get::<PumpFunQuoteResponse>(&endpoint).await {
            Ok(quote) => {
                debug!(
                    "Pump.fun buy quote for {}: {} SOL -> {} tokens (impact: {:.2}%)",
                    mint, sol_amount, quote.out_amount, quote.price_impact_pct
                );
                Ok(quote)
            }
            Err(e) => {
                error!("Failed to get buy quote for {}: {}", mint, e);
                Err(TradingError::DataError(format!("Pump.fun buy quote error: {}", e)))
            }
        }
    }

    /// Get sell quote
    pub async fn get_sell_quote(&self, mint: &str, token_amount: u64, slippage_bps: u16) -> TradingResult<PumpFunQuoteResponse> {
        let endpoint = format!("coins/{}/quote-sell?amount={}&slippage_bps={}", mint, token_amount, slippage_bps);

        match self.http_client.get::<PumpFunQuoteResponse>(&endpoint).await {
            Ok(quote) => {
                debug!(
                    "Pump.fun sell quote for {}: {} tokens -> {} SOL (impact: {:.2}%)",
                    mint, token_amount, quote.out_amount, quote.price_impact_pct
                );
                Ok(quote)
            }
            Err(e) => {
                error!("Failed to get sell quote for {}: {}", mint, e);
                Err(TradingError::DataError(format!("Pump.fun sell quote error: {}", e)))
            }
        }
    }

    /// Calculate current price from bonding curve
    pub fn calculate_token_price(&self, token: &PumpFunToken) -> f64 {
        if token.virtual_sol_reserves > 0 && token.virtual_token_reserves > 0 {
            let sol_reserves = token.virtual_sol_reserves as f64 / 1_000_000_000.0; // Convert lamports to SOL
            let token_reserves = token.virtual_token_reserves as f64 / 10_f64.powi(6); // Assume 6 decimals
            
            if token_reserves > 0.0 {
                sol_reserves / token_reserves
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Check if token has graduated to Raydium
    pub fn is_graduated(&self, token: &PumpFunToken) -> bool {
        token.complete && token.raydium_pool.is_some()
    }

    /// Get tokens by creator
    pub async fn get_tokens_by_creator(&self, creator: &str) -> TradingResult<Vec<PumpFunToken>> {
        let endpoint = format!("coins?creator={}", creator);

        match self.http_client.get::<PumpFunTokenResponse>(&endpoint).await {
            Ok(response) => {
                info!("Retrieved {} tokens by creator {}", response.tokens.len(), creator);
                Ok(response.tokens)
            }
            Err(e) => {
                error!("Failed to get tokens by creator {}: {}", creator, e);
                Err(TradingError::DataError(format!("Pump.fun creator tokens error: {}", e)))
            }
        }
    }

    /// Search tokens by name or symbol
    pub async fn search_tokens(&self, query: &str) -> TradingResult<Vec<PumpFunToken>> {
        let endpoint = format!("coins/search?q={}", urlencoding::encode(query));

        match self.http_client.get::<PumpFunTokenResponse>(&endpoint).await {
            Ok(response) => {
                info!("Found {} tokens matching '{}'", response.tokens.len(), query);
                Ok(response.tokens)
            }
            Err(e) => {
                error!("Failed to search tokens for '{}': {}", query, e);
                Err(TradingError::DataError(format!("Pump.fun search error: {}", e)))
            }
        }
    }
}

#[async_trait]
impl DataFetcher for PumpFunClient {
    async fn get_market_data(&self, symbol: &str) -> TradingResult<MarketData> {
        // Symbol can be mint address or token symbol
        let token = if symbol.len() > 40 {
            // Looks like a mint address
            self.get_token_info(symbol).await?
        } else {
            // Search by symbol
            let search_results = self.search_tokens(symbol).await?;
            if let Some(token) = search_results.into_iter().next() {
                token
            } else {
                return Err(TradingError::DataError(format!("Token not found: {}", symbol)));
            }
        };

        let price = self.calculate_token_price(&token);
        
        // Get recent trades for volume calculation
        let trades = self.get_token_trades(&token.mint, Some(100)).await.unwrap_or_default();
        let volume_24h = trades.iter()
            .filter(|trade| {
                let trade_time = DateTime::from_timestamp(trade.timestamp, 0).unwrap_or_default();
                Utc::now().signed_duration_since(trade_time).num_hours() < 24
            })
            .map(|trade| trade.sol_amount as f64 / 1_000_000_000.0)
            .sum::<f64>();

        Ok(MarketData {
            symbol: format!("{}/{}", token.symbol, "SOL"),
            price,
            volume: volume_24h,
            bid: None,
            ask: None,
            timestamp: Utc::now(),
            source: DataSource::Solana,
        })
    }

    async fn get_orderbook(&self, _symbol: &str) -> TradingResult<OrderBook> {
        // Pump.fun uses bonding curves, not orderbooks
        warn!("Pump.fun uses bonding curves - no traditional orderbook available");
        Err(TradingError::DataError(
            "Pump.fun bonding curve doesn't support orderbook data".to_string()
        ))
    }

    async fn subscribe_to_ticker(&self, symbol: &str) -> TradingResult<()> {
        info!("Pump.fun doesn't support real-time subscriptions for {}", symbol);
        // Would need to implement WebSocket or polling mechanism
        Ok(())
    }

    fn get_name(&self) -> &str {
        "Pump.fun"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pumpfun_client_creation() {
        let client = PumpFunClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_calculate_token_price() {
        let client = PumpFunClient::new().unwrap();
        let token = PumpFunToken {
            mint: "test".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "Test".to_string(),
            image_uri: "".to_string(),
            metadata_uri: "".to_string(),
            twitter: None,
            telegram: None,
            bonding_curve: "".to_string(),
            associated_bonding_curve: "".to_string(),
            creator: "".to_string(),
            created_timestamp: 0,
            raydium_pool: None,
            complete: false,
            virtual_sol_reserves: 1_000_000_000, // 1 SOL
            virtual_token_reserves: 1_000_000_000_000, // 1M tokens (6 decimals)
            hidden: false,
            total_supply: 1_000_000_000_000_000,
            website: None,
            show_name: true,
            king_of_the_hill_timestamp: None,
            market_cap: 0.0,
            reply_count: 0,
            last_reply: None,
            nsfw: false,
            market_id: None,
            inverted: None,
            is_currently_live: true,
            username: None,
            profile_image: None,
            usd_market_cap: 0.0,
        };

        let price = client.calculate_token_price(&token);
        assert_eq!(price, 0.000001); // 1 SOL / 1M tokens = 0.000001 SOL per token
    }

    #[test]
    fn test_is_graduated() {
        let client = PumpFunClient::new().unwrap();
        let mut token = PumpFunToken {
            mint: "test".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "Test".to_string(),
            image_uri: "".to_string(),
            metadata_uri: "".to_string(),
            twitter: None,
            telegram: None,
            bonding_curve: "".to_string(),
            associated_bonding_curve: "".to_string(),
            creator: "".to_string(),
            created_timestamp: 0,
            raydium_pool: None,
            complete: false,
            virtual_sol_reserves: 0,
            virtual_token_reserves: 0,
            hidden: false,
            total_supply: 0,
            website: None,
            show_name: true,
            king_of_the_hill_timestamp: None,
            market_cap: 0.0,
            reply_count: 0,
            last_reply: None,
            nsfw: false,
            market_id: None,
            inverted: None,
            is_currently_live: true,
            username: None,
            profile_image: None,
            usd_market_cap: 0.0,
        };

        assert!(!client.is_graduated(&token));

        token.complete = true;
        token.raydium_pool = Some("pool_address".to_string());
        assert!(client.is_graduated(&token));
    }
}

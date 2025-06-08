use crate::models::{TokenBalance, TradingError, TradingResult, WalletBalance};
use chrono::Utc;
use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct BalanceManager {
    rpc_client: RpcClient,
    http_client: Client,
    wallet_pubkey: Pubkey,
    cached_balances: Arc<RwLock<WalletBalance>>,
    locked_amounts: Arc<DashMap<String, f64>>, // token_mint -> locked_amount
    price_cache: Arc<DashMap<String, f64>>, // token_mint -> usd_price
    helius_api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HeliusTokenBalance {
    mint: String,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: String,
    decimals: u8,
    #[serde(rename = "uiAmount")]
    ui_amount: Option<f64>,
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct AmountVisitor;

    impl<'de> Visitor<'de> for AmountVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or integer representing token amount")
        }

        fn visit_str<E>(self, value: &str) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u32<E>(self, value: u32) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_i32<E>(self, value: i32) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(AmountVisitor)
}

#[derive(Debug, Serialize, Deserialize)]
struct HeliusBalanceResponse {
    tokens: Vec<HeliusTokenBalance>,
    #[serde(rename = "nativeBalance")]
    native_balance: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JupiterPriceResponse {
    data: HashMap<String, JupiterTokenPrice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JupiterTokenPrice {
    id: String,
    #[serde(rename = "mintSymbol")]
    mint_symbol: Option<String>,
    #[serde(rename = "vsToken")]
    vs_token: String,
    #[serde(rename = "vsTokenSymbol")]
    vs_token_symbol: String,
    price: f64,
}

impl BalanceManager {
    pub fn new(
        rpc_url: &str,
        wallet_pubkey: Pubkey,
        helius_api_key: String,
    ) -> TradingResult<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );

        let http_client = Client::new();

        let initial_balance = WalletBalance {
            sol_balance: 0.0,
            token_balances: HashMap::new(),
            total_value_usd: 0.0,
            last_updated: Utc::now(),
        };

        Ok(Self {
            rpc_client,
            http_client,
            wallet_pubkey,
            cached_balances: Arc::new(RwLock::new(initial_balance)),
            locked_amounts: Arc::new(DashMap::new()),
            price_cache: Arc::new(DashMap::new()),
            helius_api_key,
        })
    }

    /// Update all balances from the blockchain
    pub async fn update_balances(&self) -> TradingResult<WalletBalance> {
        info!("Updating wallet balances for {}", self.wallet_pubkey);

        // Get SOL balance
        let sol_balance = self.get_sol_balance().await?;

        // Get token balances using Helius enhanced API
        let token_balances = self.get_token_balances().await?;

        // Update prices for all tokens
        self.update_token_prices(&token_balances).await?;

        // Calculate total value
        let total_value_usd = self.calculate_total_value(sol_balance, &token_balances).await?;

        let wallet_balance = WalletBalance {
            sol_balance,
            token_balances,
            total_value_usd,
            last_updated: Utc::now(),
        };

        // Update cached balance
        {
            let mut cached = self.cached_balances.write().await;
            *cached = wallet_balance.clone();
        }

        debug!("Updated wallet balance: SOL: {}, Total USD: {}", sol_balance, total_value_usd);

        Ok(wallet_balance)
    }

    /// Get current cached balance
    pub async fn get_cached_balance(&self) -> WalletBalance {
        self.cached_balances.read().await.clone()
    }

    /// Check if wallet has sufficient balance for an order
    pub async fn check_sufficient_balance(
        &self,
        token_mint: &str,
        required_amount: f64,
    ) -> TradingResult<bool> {
        let balance = self.get_cached_balance().await;

        if token_mint == "So11111111111111111111111111111111111111112" {
            // SOL balance check
            let available_sol = balance.sol_balance - self.get_locked_amount(token_mint);
            Ok(available_sol >= required_amount)
        } else {
            // Token balance check
            if let Some(token_balance) = balance.token_balances.get(token_mint) {
                let available_amount = token_balance.balance - token_balance.locked_amount;
                Ok(available_amount >= required_amount)
            } else {
                Ok(false) // Token not found
            }
        }
    }

    /// Lock amount for pending order
    pub fn lock_amount(&self, token_mint: &str, amount: f64) {
        let current_locked = self.locked_amounts.get(token_mint)
            .map(|entry| *entry.value())
            .unwrap_or(0.0);
        
        self.locked_amounts.insert(token_mint.to_string(), current_locked + amount);
        
        debug!("Locked {} of token {}", amount, token_mint);
    }

    /// Unlock amount after order completion
    pub fn unlock_amount(&self, token_mint: &str, amount: f64) {
        if let Some(mut entry) = self.locked_amounts.get_mut(token_mint) {
            let new_amount = (*entry.value() - amount).max(0.0);
            *entry.value_mut() = new_amount;
            
            if new_amount == 0.0 {
                drop(entry);
                self.locked_amounts.remove(token_mint);
            }
        }
        
        debug!("Unlocked {} of token {}", amount, token_mint);
    }

    /// Get locked amount for a token
    pub fn get_locked_amount(&self, token_mint: &str) -> f64 {
        self.locked_amounts.get(token_mint)
            .map(|entry| *entry.value())
            .unwrap_or(0.0)
    }

    /// Get available (unlocked) balance for a token
    pub async fn get_available_balance(&self, token_mint: &str) -> f64 {
        let balance = self.get_cached_balance().await;
        let locked = self.get_locked_amount(token_mint);

        if token_mint == "So11111111111111111111111111111111111111112" {
            (balance.sol_balance - locked).max(0.0)
        } else {
            balance.token_balances.get(token_mint)
                .map(|tb| (tb.balance - tb.locked_amount - locked).max(0.0))
                .unwrap_or(0.0)
        }
    }

    async fn get_sol_balance(&self) -> TradingResult<f64> {
        let balance_lamports = self.rpc_client
            .get_balance(&self.wallet_pubkey)
            .map_err(|e| TradingError::RpcError(e.to_string()))?;

        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }

    async fn get_token_balances(&self) -> TradingResult<HashMap<String, TokenBalance>> {
        let url = format!(
            "https://api.helius.xyz/v0/addresses/{}/balances?api-key={}",
            self.wallet_pubkey, self.helius_api_key
        );

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| TradingError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            warn!("Helius API error: {} - {}", status, error_text);
            return Err(TradingError::DataError(format!("Helius API error: {} - {}", status, error_text)));
        }

        // Get response text first for debugging
        let response_text = response.text().await
            .map_err(|e| TradingError::DataError(e.to_string()))?;

        debug!("Helius API response: {}", response_text);

        let balance_response: HeliusBalanceResponse = serde_json::from_str(&response_text)
            .map_err(|e| TradingError::DataError(format!("Failed to parse Helius response: {} - Response: {}", e, response_text)))?;

        let mut token_balances = HashMap::new();

        for token in balance_response.tokens {
            if let Some(ui_amount) = token.ui_amount {
                if ui_amount > 0.0 {
                    let locked_amount = self.get_locked_amount(&token.mint);
                    
                    let token_balance = TokenBalance {
                        mint: token.mint.clone(),
                        symbol: "UNKNOWN".to_string(), // Will be updated with price data
                        balance: ui_amount,
                        decimals: token.decimals,
                        value_usd: None, // Will be calculated
                        locked_amount,
                    };

                    token_balances.insert(token.mint, token_balance);
                }
            }
        }

        Ok(token_balances)
    }

    async fn update_token_prices(&self, token_balances: &HashMap<String, TokenBalance>) -> TradingResult<()> {
        if token_balances.is_empty() {
            return Ok(());
        }

        let token_mints: Vec<String> = token_balances.keys().cloned().collect();
        let mints_param = token_mints.join(",");

        let url = format!(
            "https://price.jup.ag/v4/price?ids={}",
            urlencoding::encode(&mints_param)
        );

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(price_response) = response.json::<JupiterPriceResponse>().await {
                        for (mint, price_data) in price_response.data {
                            self.price_cache.insert(mint, price_data.price);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch token prices: {}", e);
            }
        }

        // Update SOL price
        if let Ok(response) = self.http_client
            .get("https://price.jup.ag/v4/price?ids=So11111111111111111111111111111111111111112")
            .send()
            .await
        {
            if let Ok(price_response) = response.json::<JupiterPriceResponse>().await {
                if let Some(sol_price) = price_response.data.get("So11111111111111111111111111111111111111112") {
                    self.price_cache.insert("So11111111111111111111111111111111111111112".to_string(), sol_price.price);
                }
            }
        }

        Ok(())
    }

    async fn calculate_total_value(
        &self,
        sol_balance: f64,
        token_balances: &HashMap<String, TokenBalance>,
    ) -> TradingResult<f64> {
        let mut total_value = 0.0;

        // Add SOL value
        if let Some(sol_price) = self.price_cache.get("So11111111111111111111111111111111111111112") {
            total_value += sol_balance * sol_price.value();
        }

        // Add token values
        for (mint, token_balance) in token_balances {
            if let Some(price) = self.price_cache.get(mint) {
                total_value += token_balance.balance * price.value();
            }
        }

        Ok(total_value)
    }

    /// Get token price in USD
    pub fn get_token_price(&self, token_mint: &str) -> Option<f64> {
        self.price_cache.get(token_mint).map(|entry| *entry.value())
    }

    /// Calculate order value in USD
    pub fn calculate_order_value_usd(&self, token_mint: &str, amount: f64) -> Option<f64> {
        self.get_token_price(token_mint).map(|price| amount * price)
    }

    /// Check if wallet has sufficient SOL for transaction fees
    pub async fn check_sufficient_sol_for_fees(&self, estimated_fee_sol: f64) -> TradingResult<bool> {
        let available_sol = self.get_available_balance("So11111111111111111111111111111111111111112").await;
        Ok(available_sol >= estimated_fee_sol)
    }

    /// Start background balance update task
    pub fn start_background_updates(&self, update_interval_seconds: u64) -> tokio::task::JoinHandle<()> {
        let balance_manager = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(update_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                if let Err(e) = balance_manager.update_balances().await {
                    error!("Failed to update balances: {}", e);
                }
            }
        })
    }
}

impl Clone for BalanceManager {
    fn clone(&self) -> Self {
        Self {
            rpc_client: RpcClient::new_with_commitment(
                self.rpc_client.url(),
                self.rpc_client.commitment(),
            ),
            http_client: self.http_client.clone(),
            wallet_pubkey: self.wallet_pubkey,
            cached_balances: Arc::clone(&self.cached_balances),
            locked_amounts: Arc::clone(&self.locked_amounts),
            price_cache: Arc::clone(&self.price_cache),
            helius_api_key: self.helius_api_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_balance_manager_creation() {
        let pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let manager = BalanceManager::new(
            "https://api.mainnet-beta.solana.com",
            pubkey,
            "test-api-key".to_string(),
        );
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_lock_unlock_amounts() {
        let pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let manager = BalanceManager::new(
            "https://api.mainnet-beta.solana.com",
            pubkey,
            "test-api-key".to_string(),
        ).unwrap();

        let token_mint = "So11111111111111111111111111111111111111112";
        
        // Initially no locked amount
        assert_eq!(manager.get_locked_amount(token_mint), 0.0);

        // Lock some amount
        manager.lock_amount(token_mint, 1.5);
        assert_eq!(manager.get_locked_amount(token_mint), 1.5);

        // Lock more
        manager.lock_amount(token_mint, 0.5);
        assert_eq!(manager.get_locked_amount(token_mint), 2.0);

        // Unlock some
        manager.unlock_amount(token_mint, 1.0);
        assert_eq!(manager.get_locked_amount(token_mint), 1.0);

        // Unlock all
        manager.unlock_amount(token_mint, 1.0);
        assert_eq!(manager.get_locked_amount(token_mint), 0.0);
    }
}

/*!
💰 Portfolio Manager - Real-time Wallet Monitoring

This module provides real-time monitoring of the bot's Solana wallet,
tracking SOL balance and all token holdings with USD valuations.
*/

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, native_token::LAMPORTS_PER_SOL};

use std::str::FromStr;
use std::time::Duration;
use tokio::time;
use tracing::{info, warn, error, debug};

use crate::config::AppConfig;

/// Complete portfolio status with all holdings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStatus {
    pub wallet_address: String,
    pub network: String,
    pub sol_balance: f64,
    pub sol_price_usd: f64,
    pub total_usd_value: f64,
    pub token_balances: Vec<TokenBalance>,
    pub active_positions_count: usize,
    pub last_updated: DateTime<Utc>,
    pub trading_mode: String,
    pub balance_status: String,
}

/// Individual token balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint_address: String,
    pub symbol: String,
    pub balance: f64,
    pub decimals: u8,
    pub usd_value: Option<f64>,
    pub price_per_token: Option<f64>,
}

/// Portfolio manager for real-time wallet monitoring
pub struct PortfolioManager {
    rpc_client: RpcClient,
    wallet_pubkey: Pubkey,
    config: AppConfig,
    update_interval: Duration,
}

impl PortfolioManager {
    /// Create new portfolio manager
    pub fn new(config: AppConfig) -> Result<Self> {
        let rpc_url = &config.solana.rpc_url;
        let rpc_client = RpcClient::new(rpc_url.clone());

        let public_key = config.solana.public_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SOLANA_PUBLIC_KEY not configured"))?;
        let wallet_pubkey = Pubkey::from_str(public_key)?;

        info!("💰 Portfolio Manager initialized for wallet: {}", public_key);
        info!("🌐 RPC: {}", rpc_url);

        Ok(Self {
            rpc_client,
            wallet_pubkey,
            config,
            update_interval: Duration::from_secs(30), // Update every 30 seconds
        })
    }

    /// Start the portfolio monitoring loop
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("🚀 Starting portfolio monitoring loop (interval: {:?})", self.update_interval);

        let mut interval = time::interval(self.update_interval);
        
        loop {
            interval.tick().await;
            
            match self.update_portfolio_status().await {
                Ok(_) => debug!("✅ Portfolio status updated successfully"),
                Err(e) => error!("❌ Failed to update portfolio status: {:?}", e),
            }
        }
    }

    /// Update and save current portfolio status
    pub async fn update_portfolio_status(&self) -> Result<()> {
        debug!("📊 Updating portfolio status...");

        // Get SOL balance
        let (sol_balance, balance_status) = match self.get_sol_balance().await {
            Ok(balance) => (balance, "✅ Live".to_string()),
            Err(e) => {
                warn!("⚠️ Failed to fetch SOL balance: {:?}", e);
                (0.0, "❌ Error".to_string())
            }
        };

        // Get token balances (simplified for now)
        let token_balances = vec![]; // TODO: Implement token balance fetching

        // Calculate total USD value
        let sol_price_usd = self.get_sol_price_estimate();
        let sol_usd_value = sol_balance * sol_price_usd;
        let total_usd_value = sol_usd_value; // Only SOL for now

        // Get network from environment
        let network = std::env::var("SOLANA_NETWORK").unwrap_or_else(|_| "mainnet".to_string());

        // Get trading mode from environment
        let paper_trading = std::env::var("PAPER_TRADING")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        // Create portfolio status
        let portfolio_status = PortfolioStatus {
            wallet_address: self.config.solana.public_key
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            network,
            sol_balance,
            sol_price_usd,
            total_usd_value,
            token_balances: token_balances.clone(),
            active_positions_count: 0, // TODO: Get from position manager
            last_updated: Utc::now(),
            trading_mode: if paper_trading { "PAPER" } else { "LIVE" }.to_string(),
            balance_status,
        };

        // TODO: Save to DragonflyDB
        // For now, we'll log the status
        info!("💰 Portfolio Status: {} SOL ({}), {} tokens, Total: {}",
              sol_balance, 
              if sol_balance > 0.0 { "✅" } else { "⚠️" },
              token_balances.len(),
              format!("${:.2}", total_usd_value)
        );

        // Store in memory for API access (temporary solution)
        // In production, this would go to DragonflyDB
        self.store_portfolio_status(portfolio_status).await?;

        Ok(())
    }

    /// Get SOL balance from wallet
    async fn get_sol_balance(&self) -> Result<f64> {
        let balance_lamports = self.rpc_client.get_balance(&self.wallet_pubkey)?;
        let balance_sol = balance_lamports as f64 / LAMPORTS_PER_SOL as f64;
        
        debug!("💰 SOL Balance: {} SOL ({} lamports)", balance_sol, balance_lamports);
        Ok(balance_sol)
    }

    /// Get all token balances from wallet (simplified implementation)
    async fn get_token_balances(&self) -> Result<Vec<TokenBalance>> {
        // TODO: Implement proper token balance fetching
        // For now, return empty vector
        debug!("🪙 Token balance fetching not yet implemented");
        Ok(vec![])
    }

    /// Get estimated SOL price (placeholder - in production use real price oracle)
    fn get_sol_price_estimate(&self) -> f64 {
        let network = std::env::var("SOLANA_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
        match network.to_lowercase().as_str() {
            "mainnet" => 150.0, // Real SOL price estimate
            "devnet" | "testnet" => 20.0, // Devnet SOL for display purposes
            _ => 20.0,
        }
    }

    /// Store portfolio status (temporary in-memory, should be DragonflyDB)
    async fn store_portfolio_status(&self, status: PortfolioStatus) -> Result<()> {
        // TODO: Implement DragonflyDB storage
        // For now, we'll use a static variable or file storage
        
        // Serialize to JSON and store
        let json_data = serde_json::to_string_pretty(&status)?;
        tokio::fs::write("/tmp/portfolio_status.json", json_data).await?;
        
        debug!("💾 Portfolio status saved to temporary storage");
        Ok(())
    }

    /// Get current portfolio status (for API endpoint)
    pub async fn get_current_status(&self) -> Result<Option<PortfolioStatus>> {
        // TODO: Read from DragonflyDB
        // For now, read from temporary file
        
        match tokio::fs::read_to_string("/tmp/portfolio_status.json").await {
            Ok(json_data) => {
                let status: PortfolioStatus = serde_json::from_str(&json_data)?;
                Ok(Some(status))
            }
            Err(_) => {
                // File doesn't exist yet, trigger an update
                self.update_portfolio_status().await?;
                Ok(None)
            }
        }
    }
}

/// Create and start portfolio manager
pub async fn start_portfolio_monitoring(config: AppConfig) -> Result<()> {
    let portfolio_manager = PortfolioManager::new(config)?;
    portfolio_manager.start_monitoring().await
}

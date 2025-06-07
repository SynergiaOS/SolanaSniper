use crate::config::AppConfig;
use crate::db_connector::DbClient;
use crate::models::TradingResult;
use crate::reflex_core::NewTokenOpportunity;
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{UiTransactionEncoding, TransactionDetails};
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

/// OnChain Stream Listener for detecting new token pools in real-time
/// This component listens to Solana blockchain events and detects new token creation
pub struct OnChainStreamListener {
    config: AppConfig,
    rpc_client: RpcClient,
    db_connector: Arc<DbClient>,
    
    // Raydium program IDs to monitor
    raydium_amm_program: Pubkey,
    raydium_clmm_program: Pubkey,
    
    // PumpFun program ID
    pumpfun_program: Pubkey,
    
    // Running state
    is_running: bool,
}

impl OnChainStreamListener {
    /// Create new OnChain Stream Listener
    pub fn new(config: AppConfig, db_connector: Arc<DbClient>) -> TradingResult<Self> {
        let rpc_client = RpcClient::new_with_commitment(
            config.solana.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        // Known program IDs for major DEXs
        let raydium_amm_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")
            .map_err(|e| format!("Invalid Raydium AMM program ID: {}", e))?;
            
        let raydium_clmm_program = Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUQpMkFr9WeqATV9Uu")
            .map_err(|e| format!("Invalid Raydium CLMM program ID: {}", e))?;
            
        let pumpfun_program = Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P")
            .map_err(|e| format!("Invalid PumpFun program ID: {}", e))?;
        
        Ok(Self {
            config,
            rpc_client,
            db_connector,
            raydium_amm_program,
            raydium_clmm_program,
            pumpfun_program,
            is_running: false,
        })
    }
    
    /// Start listening for new token opportunities
    pub async fn start_listening(&mut self) -> TradingResult<()> {
        info!("🚀 Starting OnChain Stream Listener for new token detection");
        self.is_running = true;
        
        // Subscribe to logs from major DEX programs
        let programs_to_monitor = vec![
            self.raydium_amm_program,
            self.raydium_clmm_program,
            self.pumpfun_program,
        ];
        
        info!("📡 Monitoring {} DEX programs for new pools", programs_to_monitor.len());
        
        // Main listening loop
        while self.is_running {
            match self.listen_for_new_pools().await {
                Ok(_) => {
                    debug!("✅ Listening cycle completed successfully");
                }
                Err(e) => {
                    error!("❌ Error in listening cycle: {}", e);
                    warn!("🔄 Retrying in 5 seconds...");
                    sleep(Duration::from_secs(5)).await;
                }
            }
            
            // Small delay to prevent overwhelming the RPC
            sleep(Duration::from_millis(100)).await;
        }
        
        info!("🛑 OnChain Stream Listener stopped");
        Ok(())
    }
    
    /// Stop the listener
    pub fn stop(&mut self) {
        info!("🛑 Stopping OnChain Stream Listener");
        self.is_running = false;
    }
    
    /// Listen for new pool creation events
    async fn listen_for_new_pools(&self) -> TradingResult<()> {
        // For now, we'll use a polling approach to get recent transactions
        // In production, we'd use WebSocket subscriptions for real-time data
        
        let recent_slot = self.rpc_client
            .get_slot()
            .map_err(|e| format!("Failed to get current slot: {}", e))?;
        
        debug!("🔍 Scanning slot {} for new pool creation", recent_slot);
        
        // Get recent block with transactions
        let block = self.rpc_client
            .get_block_with_config(
                recent_slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    transaction_details: Some(TransactionDetails::Full),
                    rewards: Some(false),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            );
            
        match block {
            Ok(block) => {
                if let Some(transactions) = block.transactions {
                    debug!("📦 Found {} transactions in block", transactions.len());
                    
                    for tx in transactions {
                        if let Err(e) = self.process_transaction(tx, recent_slot).await {
                            debug!("⚠️ Error processing transaction: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("⚠️ Could not get block {}: {}", recent_slot, e);
            }
        }
        
        Ok(())
    }
    
    /// Process a single transaction to detect new pool creation
    async fn process_transaction(
        &self,
        tx: solana_transaction_status::EncodedTransactionWithStatusMeta,
        slot: u64,
    ) -> TradingResult<()> {
        // For now, create a simplified version that doesn't parse transaction details
        // In production, we'd use proper transaction parsing libraries

        // Check if transaction was successful
        if tx.meta.as_ref().map(|m| m.err.is_some()).unwrap_or(true) {
            return Ok(()); // Skip failed transactions
        }

        // For demonstration, we'll create a mock detection
        // In production, this would parse actual transaction logs and instruction data
        debug!("🔍 Processing transaction for new pool detection");

        // Mock new pool detection (in production, parse actual transaction data)
        if slot % 100 == 0 { // Simulate occasional new pool detection
            let mock_signature = format!("mock_sig_{}", slot);
            if let Err(e) = self.extract_new_pool_info(&mock_signature, slot).await {
                debug!("⚠️ Could not extract pool info: {}", e);
            }
        }

        Ok(())
    }
    
    /// Extract new pool information from a transaction
    async fn extract_new_pool_info(&self, signature: &str, slot: u64) -> TradingResult<()> {
        // This is a simplified version - in production we'd parse the actual
        // transaction logs and instruction data to extract pool details
        
        debug!("🔍 Analyzing transaction {} for new pool creation", signature);
        
        // For demonstration, create a mock new token opportunity
        // In production, this would parse actual transaction data
        let opportunity = NewTokenOpportunity {
            token_address: format!("mock_token_{}", &signature[..8]),
            pool_address: format!("mock_pool_{}", &signature[..8]),
            token_symbol: Some("MOCK".to_string()),
            initial_liquidity_sol: 2.5,
            initial_liquidity_usd: 500.0,
            creation_tx_signature: signature.to_string(),
            creation_slot: slot,
            detected_at: Utc::now(),
            age_seconds: 5, // Very fresh
            dex: "Raydium".to_string(),
            risk_score: 0.7,
            mint_authority_burned: true,
            freeze_authority_burned: true,
            initial_market_cap_usd: Some(25000.0),
        };
        
        // Only process if it passes safety checks
        if opportunity.is_safe() && opportunity.is_fresh() {
            info!("🔥 NEW TOKEN DETECTED: {} ({})", 
                opportunity.token_symbol.as_deref().unwrap_or("Unknown"),
                opportunity.token_address
            );
            
            // Save to new_token_queue for Sniper Executor
            self.save_new_token_opportunity(opportunity).await?;
        }
        
        Ok(())
    }
    
    /// Save new token opportunity to database
    async fn save_new_token_opportunity(&self, opportunity: NewTokenOpportunity) -> TradingResult<()> {
        let key = opportunity.redis_key();
        
        // Save individual opportunity
        self.db_connector.set(&key, &opportunity, Some(300)).await?; // 5 min TTL
        
        // Add to new token queue
        self.db_connector.list_push("new_token_queue", &key).await?;
        
        info!("💾 Saved new token opportunity: {} (Priority: {:.2})", 
            opportunity.token_address, opportunity.priority_score());
        
        Ok(())
    }
}

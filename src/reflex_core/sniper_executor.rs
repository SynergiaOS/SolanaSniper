use crate::config::AppConfig;
use crate::db_connector::DbClient;
use crate::models::{TradingResult, TradingError};
use crate::reflex_core::NewTokenOpportunity;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

/// Ultra-fast executor for new token opportunities
/// This component processes new tokens within milliseconds of detection
pub struct SniperExecutor {
    config: AppConfig,
    db_connector: Arc<DbClient>,
    
    // Execution settings
    max_position_size_sol: f64,
    min_liquidity_sol: f64,
    max_age_seconds: u64,
    
    // Running state
    is_running: bool,
}

impl SniperExecutor {
    /// Create new Sniper Executor
    pub fn new(config: AppConfig, db_connector: Arc<DbClient>) -> Self {
        Self {
            config,
            db_connector,
            max_position_size_sol: 0.05, // Max 0.05 SOL per trade
            min_liquidity_sol: 1.0,      // Min 1 SOL liquidity required
            max_age_seconds: 60,         // Max 60 seconds old
            is_running: false,
        }
    }
    
    /// Start processing new token opportunities
    pub async fn start_processing(&mut self) -> TradingResult<()> {
        info!("⚡ Starting Sniper Executor for new token opportunities");
        self.is_running = true;
        
        while self.is_running {
            match self.process_new_token_queue().await {
                Ok(processed) => {
                    if processed > 0 {
                        debug!("⚡ Processed {} new token opportunities", processed);
                    }
                }
                Err(e) => {
                    error!("❌ Error processing new token queue: {}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
            
            // Very short delay for ultra-fast processing
            sleep(Duration::from_millis(10)).await;
        }
        
        info!("🛑 Sniper Executor stopped");
        Ok(())
    }
    
    /// Stop the executor
    pub fn stop(&mut self) {
        info!("🛑 Stopping Sniper Executor");
        self.is_running = false;
    }
    
    /// Process the new token queue
    async fn process_new_token_queue(&self) -> TradingResult<usize> {
        // Get all items from new_token_queue as raw strings
        let queue_items = self.db_connector.list_range_raw("new_token_queue", 0, -1).await?;
        
        if queue_items.is_empty() {
            return Ok(0);
        }
        
        debug!("⚡ Found {} items in new token queue", queue_items.len());
        
        let mut processed = 0;
        
        for item in queue_items {
            if let Ok(opportunity) = self.load_opportunity(&item).await {
                if self.should_execute(&opportunity) {
                    match self.execute_snipe(&opportunity).await {
                        Ok(_) => {
                            info!("🎯 SNIPED: {} at {} SOL", 
                                opportunity.token_symbol.as_deref().unwrap_or("Unknown"),
                                opportunity.initial_liquidity_sol
                            );
                            processed += 1;
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to snipe {}: {}", 
                                opportunity.token_address, e);
                        }
                    }
                } else {
                    debug!("⛔ Skipped opportunity: {}", opportunity.token_address);
                }
                
                // Remove processed item from queue
                self.db_connector.list_remove("new_token_queue", &item).await?;
            }
        }
        
        Ok(processed)
    }
    
    /// Load opportunity from database
    async fn load_opportunity(&self, key: &str) -> TradingResult<NewTokenOpportunity> {
        match self.db_connector.get::<NewTokenOpportunity>(key).await? {
            Some(opportunity) => Ok(opportunity),
            None => Err(TradingError::DataError(format!("Opportunity not found: {}", key))),
        }
    }
    
    /// Check if we should execute this opportunity
    fn should_execute(&self, opportunity: &NewTokenOpportunity) -> bool {
        // Ultra-fast decision criteria
        
        // Must be fresh
        if !opportunity.is_fresh() || opportunity.age_seconds > self.max_age_seconds {
            debug!("❌ Too old: {} seconds", opportunity.age_seconds);
            return false;
        }
        
        // Must be safe
        if !opportunity.is_safe() {
            debug!("❌ Failed safety checks");
            return false;
        }
        
        // Must have minimum liquidity
        if opportunity.initial_liquidity_sol < self.min_liquidity_sol {
            debug!("❌ Insufficient liquidity: {} SOL", opportunity.initial_liquidity_sol);
            return false;
        }
        
        // Must have good risk score
        if opportunity.risk_score < 0.5 {
            debug!("❌ Risk score too low: {}", opportunity.risk_score);
            return false;
        }
        
        // Check if we already processed this token
        // (This would be stored in a separate "processed_tokens" set)
        
        info!("✅ OPPORTUNITY APPROVED: {} (Risk: {:.2}, Liq: {} SOL, Age: {}s)", 
            opportunity.token_symbol.as_deref().unwrap_or("Unknown"),
            opportunity.risk_score,
            opportunity.initial_liquidity_sol,
            opportunity.age_seconds
        );
        
        true
    }
    
    /// Execute the snipe trade
    async fn execute_snipe(&self, opportunity: &NewTokenOpportunity) -> TradingResult<()> {
        info!("🎯 EXECUTING SNIPE: {}", opportunity.token_address);
        
        // Calculate position size based on liquidity and risk
        let position_size = self.calculate_position_size(opportunity);
        
        info!("💰 Position size: {} SOL", position_size);
        
        // In DRY RUN mode, just log the trade
        if matches!(self.config.trading.mode, crate::config::BotMode::DryRun) {
            info!("🔥 DRY RUN SNIPE EXECUTED:");
            info!("   Token: {}", opportunity.token_symbol.as_deref().unwrap_or("Unknown"));
            info!("   Address: {}", opportunity.token_address);
            info!("   Pool: {}", opportunity.pool_address);
            info!("   Position: {} SOL", position_size);
            info!("   Liquidity: {} SOL", opportunity.initial_liquidity_sol);
            info!("   Age: {} seconds", opportunity.age_seconds);
            info!("   Risk Score: {:.2}", opportunity.risk_score);
            info!("   DEX: {}", opportunity.dex);
            
            // Simulate execution time
            sleep(Duration::from_millis(50)).await;
            
            return Ok(());
        }
        
        // TODO: Implement actual trading execution
        // This would involve:
        // 1. Create Jupiter swap transaction
        // 2. Submit via Jito for MEV protection
        // 3. Monitor transaction status
        // 4. Update position tracking
        
        warn!("🚧 LIVE TRADING NOT YET IMPLEMENTED - Use DRY RUN mode");
        Err(TradingError::StrategyError("Live trading not implemented".to_string()))
    }
    
    /// Calculate optimal position size for this opportunity
    fn calculate_position_size(&self, opportunity: &NewTokenOpportunity) -> f64 {
        // Base position size
        let mut position = self.max_position_size_sol;
        
        // Adjust based on liquidity (higher liquidity = larger position)
        let liquidity_factor = (opportunity.initial_liquidity_sol / 10.0).min(1.0);
        position *= liquidity_factor;
        
        // Adjust based on risk score (higher risk = smaller position)
        position *= opportunity.risk_score;
        
        // Adjust based on age (fresher = larger position)
        let age_factor = 1.0 - (opportunity.age_seconds as f64 / 60.0);
        position *= age_factor.max(0.1);
        
        // Ensure minimum viable position
        position.max(0.01).min(self.max_position_size_sol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_position_size_calculation() {
        let config = AppConfig::default();
        let db_connector = Arc::new(DbConnector::new(config.clone()).unwrap());
        let executor = SniperExecutor::new(config, db_connector);
        
        let opportunity = NewTokenOpportunity {
            token_address: "test123".to_string(),
            pool_address: "pool123".to_string(),
            token_symbol: Some("TEST".to_string()),
            initial_liquidity_sol: 5.0,
            initial_liquidity_usd: 1000.0,
            creation_tx_signature: "sig123".to_string(),
            creation_slot: 12345,
            detected_at: Utc::now(),
            age_seconds: 10,
            dex: "Raydium".to_string(),
            risk_score: 0.8,
            mint_authority_burned: true,
            freeze_authority_burned: true,
            initial_market_cap_usd: Some(50000.0),
        };
        
        let position_size = executor.calculate_position_size(&opportunity);
        assert!(position_size > 0.0);
        assert!(position_size <= executor.max_position_size_sol);
    }
}

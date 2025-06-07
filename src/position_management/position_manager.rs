use crate::config::AppConfig;
use crate::models::{TradingResult, TradingError, StrategySignal};
use crate::position_management::{
    ActivePosition, PositionStatus, PositionManagement, PositionManagementFactory
};
use crate::data_fetcher::jupiter_client::JupiterClient;
use redis::{AsyncCommands, Client as RedisClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};


/// Position Manager - monitors active positions and generates exit signals
/// 
/// This is the "MEMORY AND DISCIPLINE" component of our trading system.
/// It runs in its own tokio task and continuously monitors all active positions,
/// checking for exit conditions (TP/SL/Time) and generating sell signals.
pub struct PositionManager {
    config: AppConfig,
    redis_client: RedisClient,
    jupiter_client: Arc<JupiterClient>,
    signal_sender: mpsc::Sender<StrategySignal>,
    monitoring_interval_seconds: u64,
    position_managers: HashMap<String, Box<dyn PositionManagement>>,
}

impl PositionManager {
    /// Create new position manager
    pub fn new(
        config: AppConfig,
        redis_client: RedisClient,
        jupiter_client: Arc<JupiterClient>,
        signal_sender: mpsc::Sender<StrategySignal>,
    ) -> Self {
        Self {
            config,
            redis_client,
            jupiter_client,
            signal_sender,
            monitoring_interval_seconds: 2, // Check every 2 seconds
            position_managers: HashMap::new(),
        }
    }
    
    /// Start the position monitoring loop
    pub async fn start_monitoring(&self) -> TradingResult<()> {
        info!("🎯 Starting Position Manager - MEMORY AND DISCIPLINE activated");
        info!("📊 Monitoring interval: {} seconds", self.monitoring_interval_seconds);
        
        let mut interval = interval(Duration::from_secs(self.monitoring_interval_seconds));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.monitor_positions().await {
                error!("💥 Error monitoring positions: {}", e);
                // Continue monitoring even if there's an error
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    
    /// Monitor all active positions and generate exit signals
    async fn monitor_positions(&self) -> TradingResult<()> {
        debug!("🔍 Checking active positions...");
        
        // Get all active positions from DragonflyDB
        let active_positions = self.get_all_active_positions().await?;
        
        if active_positions.is_empty() {
            debug!("📭 No active positions to monitor");
            return Ok(());
        }
        
        info!("📊 Monitoring {} active positions", active_positions.len());
        
        // Get unique token mints for price fetching
        let token_mints: Vec<String> = active_positions
            .iter()
            .map(|p| p.token_mint.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        // Fetch current prices for all tokens
        let current_prices = self.fetch_current_prices(&token_mints).await?;
        
        // Check each position for exit conditions
        for mut position in active_positions {
            if let Some(current_price) = current_prices.get(&position.token_mint) {
                if let Err(e) = self.check_position_exit(&mut position, *current_price).await {
                    error!("💥 Error checking position {}: {}", position.id, e);
                }
            } else {
                warn!("⚠️ No price data for token {} in position {}", position.token_mint, position.id);
            }
        }
        
        Ok(())
    }
    
    /// Check if a position should be closed and generate exit signal if needed
    async fn check_position_exit(&self, position: &mut ActivePosition, current_price: f64) -> TradingResult<()> {
        // Skip if position is already closing or closed
        if matches!(position.status, PositionStatus::Closing | PositionStatus::Closed) {
            return Ok(());
        }
        
        // Update position with current price
        position.update_with_price(current_price);
        
        // Get appropriate position manager for this strategy
        let manager = self.get_position_manager(&position.strategy_name);
        
        // Check if position should exit
        if let Some(exit_reason) = manager.should_exit(position, current_price).await? {
            info!("🚨 Position {} should exit: {:?}", position.id, exit_reason);
            
            // Generate exit signal
            let exit_signal = manager.generate_exit_signal(position, exit_reason.clone());
            
            // Mark position as closing
            position.status = PositionStatus::Closing;
            
            // Update position in database
            self.update_position_in_db(position).await?;
            
            // Send exit signal
            if let Err(e) = self.signal_sender.send(exit_signal).await {
                error!("💥 Failed to send exit signal for position {}: {}", position.id, e);
                return Err(TradingError::DataError(format!("Failed to send exit signal: {}", e)));
            }
            
            info!("✅ Exit signal sent for position {} (reason: {:?})", position.id, exit_reason);
        } else {
            // Update position metrics and save to database
            self.update_position_in_db(position).await?;
            
            // Log position status
            let pnl_percent = position.calculate_unrealized_pnl_percent(current_price);
            let age_minutes = position.get_age_seconds() / 60;
            
            debug!("📊 Position {} - Price: {:.6}, P&L: {:.2}%, Age: {}m", 
                position.id, current_price, pnl_percent, age_minutes);
        }
        
        Ok(())
    }
    
    /// Get all active positions from DragonflyDB
    async fn get_all_active_positions(&self) -> TradingResult<Vec<ActivePosition>> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| TradingError::DataError(format!("Redis connection failed: {}", e)))?;
        
        // Get all keys matching active_position:*
        let keys: Vec<String> = conn.keys("active_position:*").await
            .map_err(|e| TradingError::DataError(format!("Failed to get position keys: {}", e)))?;

        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Get all position data
        let position_data: Vec<String> = conn.mget(&keys).await
            .map_err(|e| TradingError::DataError(format!("Failed to get position data: {}", e)))?;
        
        // Deserialize positions
        let mut positions = Vec::new();
        for (key, data) in keys.iter().zip(position_data.iter()) {
            match ActivePosition::from_json(data) {
                Ok(position) => positions.push(position),
                Err(e) => {
                    error!("💥 Failed to deserialize position from key {}: {}", key, e);
                }
            }
        }
        
        Ok(positions)
    }
    
    /// Fetch current prices for multiple tokens
    async fn fetch_current_prices(&self, token_mints: &[String]) -> TradingResult<HashMap<String, f64>> {
        let mut prices = HashMap::new();
        
        // For now, we'll fetch prices one by one
        // TODO: Implement batch price fetching for better performance
        for token_mint in token_mints {
            match self.fetch_token_price(token_mint).await {
                Ok(price) => {
                    prices.insert(token_mint.clone(), price);
                }
                Err(e) => {
                    warn!("⚠️ Failed to fetch price for {}: {}", token_mint, e);
                }
            }
        }
        
        Ok(prices)
    }
    
    /// Fetch current price for a single token
    async fn fetch_token_price(&self, token_mint: &str) -> TradingResult<f64> {
        // Use Jupiter API to get current price
        // This is a simplified implementation - in production we might use multiple sources
        
        // For now, simulate price fetching
        // TODO: Implement actual Jupiter price fetching
        
        // Simulate some price movement for testing
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let base_price = 0.001; // Base price
        let volatility = 0.1;   // 10% volatility
        let price_change = rng.gen_range(-volatility..volatility);
        let current_price = base_price * (1.0 + price_change);
        
        debug!("💰 Fetched price for {}: {:.6}", token_mint, current_price);
        Ok(current_price)
    }
    
    /// Update position in database
    async fn update_position_in_db(&self, position: &ActivePosition) -> TradingResult<()> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| TradingError::DataError(format!("Redis connection failed: {}", e)))?;
        
        let key = position.get_db_key();
        let data = position.to_json()?;
        
        conn.set::<_, _, ()>(&key, data).await
            .map_err(|e| TradingError::DataError(format!("Failed to update position: {}", e)))?;
        
        debug!("💾 Updated position {} in database", position.id);
        Ok(())
    }
    
    /// Get position manager for strategy type
    fn get_position_manager(&self, strategy_name: &str) -> Box<dyn PositionManagement> {
        // Extract strategy type from strategy name
        let strategy_type = if strategy_name.contains("pure_sniper") {
            "pure_sniper"
        } else if strategy_name.contains("cautious_sniper") {
            "cautious_sniper"
        } else if strategy_name.contains("momentum") {
            "momentum_trader"
        } else if strategy_name.contains("dlmm") {
            "dlmm_fee_harvester"
        } else {
            "default"
        };
        
        PositionManagementFactory::create_manager(strategy_type)
    }
    
    /// Add new position to monitoring (called by TradingExecutor after successful buy)
    pub async fn add_position(&self, position: ActivePosition) -> TradingResult<()> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| TradingError::DataError(format!("Redis connection failed: {}", e)))?;

        let key = position.get_db_key();
        let data = position.to_json()?;

        conn.set::<_, _, ()>(&key, data).await
            .map_err(|e| TradingError::DataError(format!("Failed to add position: {}", e)))?;
        
        info!("✅ Added position {} to monitoring (strategy: {}, symbol: {})",
            position.id, position.strategy_name, position.symbol);

        Ok(())
    }

    /// Remove position from monitoring (called after successful exit)
    pub async fn remove_position(&self, position_id: &str) -> TradingResult<()> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| TradingError::DataError(format!("Redis connection failed: {}", e)))?;

        let key = format!("active_position:{}", position_id);

        let deleted: i32 = conn.del(&key).await
            .map_err(|e| TradingError::DataError(format!("Failed to remove position: {}", e)))?;

        if deleted > 0 {
            info!("✅ Removed position {} from monitoring", position_id);
        } else {
            warn!("⚠️ Position {} was not found in database", position_id);
        }

        Ok(())
    }

    /// Get position by ID
    pub async fn get_position(&self, position_id: &str) -> TradingResult<Option<ActivePosition>> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| TradingError::DataError(format!("Redis connection failed: {}", e)))?;

        let key = format!("active_position:{}", position_id);

        let data: Option<String> = conn.get(&key).await
            .map_err(|e| TradingError::DataError(format!("Failed to get position: {}", e)))?;

        match data {
            Some(json) => Ok(Some(ActivePosition::from_json(&json)?)),
            None => Ok(None),
        }
    }

    /// Get all positions for a specific strategy
    pub async fn get_positions_by_strategy(&self, strategy_name: &str) -> TradingResult<Vec<ActivePosition>> {
        let all_positions = self.get_all_active_positions().await?;

        Ok(all_positions
            .into_iter()
            .filter(|p| p.strategy_name == strategy_name)
            .collect())
    }

    /// Get position statistics
    pub async fn get_position_stats(&self) -> TradingResult<PositionStats> {
        let positions = self.get_all_active_positions().await?;

        let total_positions = positions.len();
        let total_invested = positions.iter().map(|p| p.amount_sol_invested).sum();

        let mut strategy_counts = HashMap::new();
        for position in &positions {
            *strategy_counts.entry(position.strategy_name.clone()).or_insert(0) += 1;
        }

        Ok(PositionStats {
            total_positions,
            total_invested_sol: total_invested,
            strategy_breakdown: strategy_counts,
            oldest_position_age_hours: positions
                .iter()
                .map(|p| p.get_age_hours())
                .fold(0.0, f64::max),
        })
    }
}

/// Position statistics for monitoring and reporting
#[derive(Debug, Clone)]
pub struct PositionStats {
    pub total_positions: usize,
    pub total_invested_sol: f64,
    pub strategy_breakdown: HashMap<String, usize>,
    pub oldest_position_age_hours: f64,
}

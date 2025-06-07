pub mod position_manager;
pub mod active_position;

use crate::models::{TradingResult, StrategySignal, SignalType};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub use position_manager::PositionManager;
pub use active_position::ActivePosition;

/// Position exit reason for tracking and analytics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExitReason {
    TakeProfit,
    StopLoss,
    TimeExit,
    ManualClose,
    EmergencyExit,
}

/// Position status for lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionStatus {
    Opening,     // Order placed but not yet filled
    Active,      // Position is open and being monitored
    Closing,     // Exit order placed but not yet filled
    Closed,      // Position fully closed
    Failed,      // Failed to open or close
}

/// Exit strategy configuration for different strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategy {
    pub take_profit_percent: f64,    // e.g., 500.0 for +500%
    pub stop_loss_percent: f64,      // e.g., -90.0 for -90%
    pub time_exit_hours: f64,        // e.g., 1.0 for 1 hour
    pub trailing_stop: Option<f64>,  // Optional trailing stop percentage
    pub partial_exit_levels: Vec<PartialExitLevel>, // For scaling out
}

/// Partial exit configuration for scaling out of positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialExitLevel {
    pub price_percent: f64,    // At what % gain to exit
    pub amount_percent: f64,   // What % of position to exit
}

/// Position performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMetrics {
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
    pub max_profit: f64,
    pub max_profit_percent: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percent: f64,
    pub duration_seconds: i64,
    pub current_price: f64,
    pub price_change_percent: f64,
}

/// Position management trait for different position types
#[async_trait]
pub trait PositionManagement: Send + Sync {
    /// Check if position should be closed based on exit strategy
    async fn should_exit(&self, position: &ActivePosition, current_price: f64) -> TradingResult<Option<ExitReason>>;
    
    /// Calculate position metrics
    fn calculate_metrics(&self, position: &ActivePosition, current_price: f64) -> PositionMetrics;
    
    /// Generate exit signal for position
    fn generate_exit_signal(&self, position: &ActivePosition, exit_reason: ExitReason) -> StrategySignal;
    
    /// Update position with new market data
    async fn update_position(&self, position: &mut ActivePosition, current_price: f64) -> TradingResult<()>;
}

/// Default exit strategy configurations for different strategy types
impl ExitStrategy {
    /// Pure Sniper exit strategy: High risk, high reward
    pub fn pure_sniper() -> Self {
        Self {
            take_profit_percent: 300.0,  // +300%
            stop_loss_percent: -80.0,    // -80%
            time_exit_hours: 1.0,        // 1 hour
            trailing_stop: None,
            partial_exit_levels: vec![],
        }
    }
    
    /// Cautious Sniper exit strategy: Moderate risk
    pub fn cautious_sniper() -> Self {
        Self {
            take_profit_percent: 200.0,  // +200%
            stop_loss_percent: -60.0,    // -60%
            time_exit_hours: 2.0,        // 2 hours
            trailing_stop: None,
            partial_exit_levels: vec![],
        }
    }
    
    /// Momentum Trader exit strategy: Trailing stop
    pub fn momentum_trader() -> Self {
        Self {
            take_profit_percent: 0.0,    // No fixed TP
            stop_loss_percent: -20.0,    // -20% initial SL
            time_exit_hours: 24.0,       // 24 hours max
            trailing_stop: Some(20.0),   // 20% trailing stop
            partial_exit_levels: vec![
                PartialExitLevel { price_percent: 50.0, amount_percent: 25.0 },  // Take 25% at +50%
                PartialExitLevel { price_percent: 100.0, amount_percent: 25.0 }, // Take 25% at +100%
            ],
        }
    }
    
    /// DLMM Fee Harvester exit strategy: Conservative
    pub fn dlmm_fee_harvester() -> Self {
        Self {
            take_profit_percent: 0.0,    // No TP (LP position)
            stop_loss_percent: -30.0,    // -30% emergency exit
            time_exit_hours: 168.0,      // 1 week
            trailing_stop: None,
            partial_exit_levels: vec![],
        }
    }
}

/// Position management factory for creating appropriate managers
pub struct PositionManagementFactory;

impl PositionManagementFactory {
    pub fn create_manager(strategy_type: &str) -> Box<dyn PositionManagement> {
        match strategy_type {
            "pure_sniper" | "cautious_sniper" => {
                Box::new(SniperPositionManager::new())
            }
            "momentum_trader" => {
                Box::new(MomentumPositionManager::new())
            }
            "dlmm_fee_harvester" => {
                Box::new(DLMMPositionManager::new())
            }
            _ => {
                Box::new(DefaultPositionManager::new())
            }
        }
    }
}

/// Default position manager for basic strategies
pub struct DefaultPositionManager;

impl DefaultPositionManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PositionManagement for DefaultPositionManager {
    async fn should_exit(&self, position: &ActivePosition, current_price: f64) -> TradingResult<Option<ExitReason>> {
        let now = Utc::now();
        
        // Check time exit
        if now >= position.time_exit_at {
            return Ok(Some(ExitReason::TimeExit));
        }
        
        // Check take profit
        if let Some(tp_price) = position.take_profit_price {
            if current_price >= tp_price {
                return Ok(Some(ExitReason::TakeProfit));
            }
        }
        
        // Check stop loss
        if let Some(sl_price) = position.stop_loss_price {
            if current_price <= sl_price {
                return Ok(Some(ExitReason::StopLoss));
            }
        }
        
        Ok(None)
    }
    
    fn calculate_metrics(&self, position: &ActivePosition, current_price: f64) -> PositionMetrics {
        let price_change = current_price - position.entry_price;
        let price_change_percent = (price_change / position.entry_price) * 100.0;
        
        let unrealized_pnl = price_change * position.amount_tokens;
        let unrealized_pnl_percent = price_change_percent;
        
        let duration_seconds = (Utc::now() - position.created_at).num_seconds();
        
        PositionMetrics {
            unrealized_pnl,
            unrealized_pnl_percent,
            max_profit: position.max_profit,
            max_profit_percent: position.max_profit_percent,
            max_drawdown: position.max_drawdown,
            max_drawdown_percent: position.max_drawdown_percent,
            duration_seconds,
            current_price,
            price_change_percent,
        }
    }
    
    fn generate_exit_signal(&self, position: &ActivePosition, exit_reason: ExitReason) -> StrategySignal {
        StrategySignal {
            strategy: position.strategy_name.clone(),
            symbol: position.symbol.clone(),
            signal_type: SignalType::Sell,
            strength: 1.0, // Max strength for exit signals
            price: 0.0,    // Market price
            size: position.amount_tokens,
            metadata: serde_json::json!({
                "position_id": position.id,
                "exit_reason": exit_reason,
                "entry_price": position.entry_price,
                "strategy_type": "position_exit",
                "use_mev_protection": true,
                "priority": "high"
            }),
            timestamp: Utc::now(),
        }
    }
    
    async fn update_position(&self, position: &mut ActivePosition, current_price: f64) -> TradingResult<()> {
        let price_change_percent = ((current_price - position.entry_price) / position.entry_price) * 100.0;
        
        // Update max profit
        if price_change_percent > position.max_profit_percent {
            position.max_profit_percent = price_change_percent;
            position.max_profit = (current_price - position.entry_price) * position.amount_tokens;
        }
        
        // Update max drawdown
        if price_change_percent < position.max_drawdown_percent {
            position.max_drawdown_percent = price_change_percent;
            position.max_drawdown = (current_price - position.entry_price) * position.amount_tokens;
        }
        
        position.last_price = current_price;
        position.updated_at = Utc::now();
        
        Ok(())
    }
}

/// Sniper-specific position manager (Pure Sniper, Cautious Sniper)
pub struct SniperPositionManager;

impl SniperPositionManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PositionManagement for SniperPositionManager {
    async fn should_exit(&self, position: &ActivePosition, current_price: f64) -> TradingResult<Option<ExitReason>> {
        // Use default implementation for now
        DefaultPositionManager::new().should_exit(position, current_price).await
    }
    
    fn calculate_metrics(&self, position: &ActivePosition, current_price: f64) -> PositionMetrics {
        DefaultPositionManager::new().calculate_metrics(position, current_price)
    }
    
    fn generate_exit_signal(&self, position: &ActivePosition, exit_reason: ExitReason) -> StrategySignal {
        DefaultPositionManager::new().generate_exit_signal(position, exit_reason)
    }
    
    async fn update_position(&self, position: &mut ActivePosition, current_price: f64) -> TradingResult<()> {
        DefaultPositionManager::new().update_position(position, current_price).await
    }
}

/// Momentum-specific position manager with trailing stops
pub struct MomentumPositionManager;

impl MomentumPositionManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PositionManagement for MomentumPositionManager {
    async fn should_exit(&self, position: &ActivePosition, current_price: f64) -> TradingResult<Option<ExitReason>> {
        // TODO: Implement trailing stop logic
        DefaultPositionManager::new().should_exit(position, current_price).await
    }
    
    fn calculate_metrics(&self, position: &ActivePosition, current_price: f64) -> PositionMetrics {
        DefaultPositionManager::new().calculate_metrics(position, current_price)
    }
    
    fn generate_exit_signal(&self, position: &ActivePosition, exit_reason: ExitReason) -> StrategySignal {
        DefaultPositionManager::new().generate_exit_signal(position, exit_reason)
    }
    
    async fn update_position(&self, position: &mut ActivePosition, current_price: f64) -> TradingResult<()> {
        DefaultPositionManager::new().update_position(position, current_price).await
    }
}

/// DLMM-specific position manager for liquidity positions
pub struct DLMMPositionManager;

impl DLMMPositionManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PositionManagement for DLMMPositionManager {
    async fn should_exit(&self, position: &ActivePosition, current_price: f64) -> TradingResult<Option<ExitReason>> {
        // TODO: Implement DLMM-specific exit logic
        DefaultPositionManager::new().should_exit(position, current_price).await
    }
    
    fn calculate_metrics(&self, position: &ActivePosition, current_price: f64) -> PositionMetrics {
        DefaultPositionManager::new().calculate_metrics(position, current_price)
    }
    
    fn generate_exit_signal(&self, position: &ActivePosition, exit_reason: ExitReason) -> StrategySignal {
        DefaultPositionManager::new().generate_exit_signal(position, exit_reason)
    }
    
    async fn update_position(&self, position: &mut ActivePosition, current_price: f64) -> TradingResult<()> {
        DefaultPositionManager::new().update_position(position, current_price).await
    }
}

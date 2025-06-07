use crate::models::{Order, StrategySignal, TradingResult};
use crate::position_management::{ExitStrategy, PositionStatus};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Active position representing an open trade being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePosition {
    /// Unique position identifier
    pub id: Uuid,
    
    /// Position identification
    pub symbol: String,              // e.g., "TOKEN/SOL"
    pub token_mint: String,          // Token mint address
    pub sol_mint: String,            // SOL mint address (usually constant)
    pub strategy_name: String,       // e.g., "pure_sniper"
    
    /// Entry information
    pub entry_price: f64,            // Price at which we bought
    pub entry_timestamp: DateTime<Utc>,
    pub amount_sol_invested: f64,    // How much SOL we spent
    pub amount_tokens: f64,          // How many tokens we received
    
    /// Exit strategy configuration
    pub take_profit_price: Option<f64>,  // Calculated TP price
    pub stop_loss_price: Option<f64>,    // Calculated SL price
    pub time_exit_at: DateTime<Utc>,     // When to exit by time
    pub exit_strategy: ExitStrategy,     // Full exit strategy config
    
    /// Position tracking
    pub status: PositionStatus,
    pub last_price: f64,             // Last known price
    pub max_profit: f64,             // Maximum profit achieved (SOL)
    pub max_profit_percent: f64,     // Maximum profit percentage
    pub max_drawdown: f64,           // Maximum drawdown (SOL)
    pub max_drawdown_percent: f64,   // Maximum drawdown percentage
    
    /// Transaction tracking
    pub entry_order_id: Option<Uuid>,   // ID of the buy order
    pub entry_transaction_signature: Option<String>, // Solana transaction signature
    pub exit_order_id: Option<Uuid>,    // ID of the sell order (when closing)
    pub exit_transaction_signature: Option<String>,  // Exit transaction signature
    
    /// Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,     // Strategy-specific metadata
}

impl ActivePosition {
    /// Create new active position from entry order and signal
    pub fn from_entry_order(
        order: &Order,
        signal: &StrategySignal,
        entry_price: f64,
        amount_tokens: f64,
        exit_strategy: ExitStrategy,
    ) -> TradingResult<Self> {
        let now = Utc::now();
        let time_exit_at = now + Duration::seconds((exit_strategy.time_exit_hours * 3600.0) as i64);
        
        // Calculate TP and SL prices
        let take_profit_price = if exit_strategy.take_profit_percent > 0.0 {
            Some(entry_price * (1.0 + exit_strategy.take_profit_percent / 100.0))
        } else {
            None
        };
        
        let stop_loss_price = if exit_strategy.stop_loss_percent < 0.0 {
            Some(entry_price * (1.0 + exit_strategy.stop_loss_percent / 100.0))
        } else {
            None
        };
        
        Ok(Self {
            id: Uuid::new_v4(),
            symbol: signal.symbol.clone(),
            token_mint: Self::extract_token_mint_from_metadata(&signal.metadata)?,
            sol_mint: "So11111111111111111111111111111111111111112".to_string(),
            strategy_name: signal.strategy.clone(),
            
            entry_price,
            entry_timestamp: now,
            amount_sol_invested: order.size,
            amount_tokens,
            
            take_profit_price,
            stop_loss_price,
            time_exit_at,
            exit_strategy,
            
            status: PositionStatus::Active,
            last_price: entry_price,
            max_profit: 0.0,
            max_profit_percent: 0.0,
            max_drawdown: 0.0,
            max_drawdown_percent: 0.0,
            
            entry_order_id: Some(order.id),
            entry_transaction_signature: order.transaction_signature.clone(),
            exit_order_id: None,
            exit_transaction_signature: None,
            
            created_at: now,
            updated_at: now,
            metadata: signal.metadata.clone(),
        })
    }
    
    /// Create position for Pure Sniper strategy
    pub fn pure_sniper(
        order: &Order,
        signal: &StrategySignal,
        entry_price: f64,
        amount_tokens: f64,
    ) -> TradingResult<Self> {
        Self::from_entry_order(
            order,
            signal,
            entry_price,
            amount_tokens,
            ExitStrategy::pure_sniper(),
        )
    }
    
    /// Create position for Cautious Sniper strategy
    pub fn cautious_sniper(
        order: &Order,
        signal: &StrategySignal,
        entry_price: f64,
        amount_tokens: f64,
    ) -> TradingResult<Self> {
        Self::from_entry_order(
            order,
            signal,
            entry_price,
            amount_tokens,
            ExitStrategy::cautious_sniper(),
        )
    }
    
    /// Create position for Momentum Trader strategy
    pub fn momentum_trader(
        order: &Order,
        signal: &StrategySignal,
        entry_price: f64,
        amount_tokens: f64,
    ) -> TradingResult<Self> {
        Self::from_entry_order(
            order,
            signal,
            entry_price,
            amount_tokens,
            ExitStrategy::momentum_trader(),
        )
    }
    
    /// Create position for DLMM Fee Harvester strategy
    pub fn dlmm_fee_harvester(
        order: &Order,
        signal: &StrategySignal,
        entry_price: f64,
        amount_tokens: f64,
    ) -> TradingResult<Self> {
        Self::from_entry_order(
            order,
            signal,
            entry_price,
            amount_tokens,
            ExitStrategy::dlmm_fee_harvester(),
        )
    }
    
    /// Calculate current unrealized P&L in SOL
    pub fn calculate_unrealized_pnl(&self, current_price: f64) -> f64 {
        (current_price - self.entry_price) * self.amount_tokens
    }
    
    /// Calculate current unrealized P&L percentage
    pub fn calculate_unrealized_pnl_percent(&self, current_price: f64) -> f64 {
        ((current_price - self.entry_price) / self.entry_price) * 100.0
    }
    
    /// Calculate current position value in SOL
    pub fn calculate_current_value(&self, current_price: f64) -> f64 {
        current_price * self.amount_tokens
    }
    
    /// Check if position should be closed based on current price
    pub fn should_close(&self, current_price: f64) -> Option<String> {
        let now = Utc::now();
        
        // Time exit check
        if now >= self.time_exit_at {
            return Some("time_exit".to_string());
        }
        
        // Take profit check
        if let Some(tp_price) = self.take_profit_price {
            if current_price >= tp_price {
                return Some("take_profit".to_string());
            }
        }
        
        // Stop loss check
        if let Some(sl_price) = self.stop_loss_price {
            if current_price <= sl_price {
                return Some("stop_loss".to_string());
            }
        }
        
        None
    }
    
    /// Update position with new price data
    pub fn update_with_price(&mut self, current_price: f64) {
        self.last_price = current_price;
        self.updated_at = Utc::now();
        
        // Update max profit tracking
        let current_pnl_percent = self.calculate_unrealized_pnl_percent(current_price);
        if current_pnl_percent > self.max_profit_percent {
            self.max_profit_percent = current_pnl_percent;
            self.max_profit = self.calculate_unrealized_pnl(current_price);
        }
        
        // Update max drawdown tracking
        if current_pnl_percent < self.max_drawdown_percent {
            self.max_drawdown_percent = current_pnl_percent;
            self.max_drawdown = self.calculate_unrealized_pnl(current_price);
        }
    }
    
    /// Mark position as closing (exit order placed)
    pub fn mark_closing(&mut self, exit_order_id: Uuid) {
        self.status = PositionStatus::Closing;
        self.exit_order_id = Some(exit_order_id);
        self.updated_at = Utc::now();
    }
    
    /// Mark position as closed (exit order filled)
    pub fn mark_closed(&mut self, exit_transaction_signature: String) {
        self.status = PositionStatus::Closed;
        self.exit_transaction_signature = Some(exit_transaction_signature);
        self.updated_at = Utc::now();
    }
    
    /// Get DragonflyDB key for this position
    pub fn get_db_key(&self) -> String {
        format!("active_position:{}", self.id)
    }
    
    /// Get position age in seconds
    pub fn get_age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }
    
    /// Get position age in hours
    pub fn get_age_hours(&self) -> f64 {
        self.get_age_seconds() as f64 / 3600.0
    }
    
    /// Check if position is expired (past time exit)
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.time_exit_at
    }
    
    /// Get time remaining until exit in seconds
    pub fn time_remaining_seconds(&self) -> i64 {
        (self.time_exit_at - Utc::now()).num_seconds().max(0)
    }
    
    /// Extract token mint from signal metadata
    fn extract_token_mint_from_metadata(metadata: &serde_json::Value) -> TradingResult<String> {
        metadata
            .get("token_mint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                crate::models::TradingError::DataError(
                    "Missing token_mint in signal metadata".to_string()
                )
            })
    }
    
    /// Convert to JSON for DragonflyDB storage
    pub fn to_json(&self) -> TradingResult<String> {
        serde_json::to_string(self).map_err(|e| {
            crate::models::TradingError::DeserializationError(format!("Failed to serialize position: {}", e))
        })
    }
    
    /// Create from JSON stored in DragonflyDB
    pub fn from_json(json: &str) -> TradingResult<Self> {
        serde_json::from_str(json).map_err(|e| {
            crate::models::TradingError::DeserializationError(format!("Failed to deserialize position: {}", e))
        })
    }

    /// Create ActivePosition from successful order execution
    pub fn from_execution(
        order: &Order,
        signal: &StrategySignal,
        execution_result: &crate::models::ExecutionResult,
    ) -> TradingResult<Self> {
        // Calculate entry price and amount from execution result
        let entry_price = execution_result.filled_price.unwrap_or(signal.price);
        let amount_tokens = execution_result.filled_size / entry_price;

        // Create position based on strategy type
        match signal.strategy.as_str() {
            "pure_sniper" => {
                Self::pure_sniper(order, signal, entry_price, amount_tokens)
            }
            "cautious_sniper" => {
                Self::cautious_sniper(order, signal, entry_price, amount_tokens)
            }
            "momentum_trader" => {
                Self::momentum_trader(order, signal, entry_price, amount_tokens)
            }
            "dlmm_fee_harvester" => {
                Self::dlmm_fee_harvester(order, signal, entry_price, amount_tokens)
            }
            _ => {
                // Default to pure sniper for unknown strategies
                tracing::warn!("⚠️ Unknown strategy '{}', defaulting to pure_sniper", signal.strategy);
                Self::pure_sniper(order, signal, entry_price, amount_tokens)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OrderSide, OrderType, OrderStatus, TimeInForce, ExecutionParams, SignalType};
    
    fn create_test_order() -> Order {
        Order {
            id: Uuid::new_v4(),
            exchange_order_id: None,
            symbol: "TEST/SOL".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            size: 0.05,
            price: Some(0.001),
            filled_size: 0.05,
            average_fill_price: Some(0.001),
            status: OrderStatus::Filled,
            exchange: "jupiter".to_string(),
            strategy: "pure_sniper".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            time_in_force: TimeInForce::IOC,
            execution_params: ExecutionParams::default(),
            stop_loss: None,
            take_profit: None,
            max_slippage_bps: 300,
            actual_slippage_bps: None,
            fees_paid: 0.0,
            transaction_signature: Some("test_signature".to_string()),
            bundle_id: None,
        }
    }
    
    fn create_test_signal() -> StrategySignal {
        StrategySignal {
            strategy: "pure_sniper".to_string(),
            symbol: "TEST/SOL".to_string(),
            signal_type: SignalType::Buy,
            strength: 0.95,
            price: 0.001,
            size: 0.05,
            metadata: serde_json::json!({
                "token_mint": "test_token_mint_123",
                "strategy_type": "pure_sniper"
            }),
            timestamp: Utc::now(),
        }
    }
    
    #[test]
    fn test_pure_sniper_position_creation() {
        let order = create_test_order();
        let signal = create_test_signal();
        
        let position = ActivePosition::pure_sniper(&order, &signal, 0.001, 50.0).unwrap();
        
        assert_eq!(position.strategy_name, "pure_sniper");
        assert_eq!(position.entry_price, 0.001);
        assert_eq!(position.amount_tokens, 50.0);
        assert_eq!(position.amount_sol_invested, 0.05);
        
        // Check Pure Sniper exit strategy
        assert_eq!(position.exit_strategy.take_profit_percent, 300.0);
        assert_eq!(position.exit_strategy.stop_loss_percent, -80.0);
        assert_eq!(position.exit_strategy.time_exit_hours, 1.0);
        
        // Check calculated prices
        assert_eq!(position.take_profit_price, Some(0.004)); // +300%
        assert_eq!(position.stop_loss_price, Some(0.0002));  // -80%
    }
    
    #[test]
    fn test_position_pnl_calculation() {
        let order = create_test_order();
        let signal = create_test_signal();
        let position = ActivePosition::pure_sniper(&order, &signal, 0.001, 50.0).unwrap();
        
        // Test profit scenario
        let profit_pnl = position.calculate_unrealized_pnl(0.002);
        assert_eq!(profit_pnl, 0.05); // (0.002 - 0.001) * 50 = 0.05 SOL
        
        let profit_percent = position.calculate_unrealized_pnl_percent(0.002);
        assert_eq!(profit_percent, 100.0); // 100% gain
        
        // Test loss scenario
        let loss_pnl = position.calculate_unrealized_pnl(0.0005);
        assert_eq!(loss_pnl, -0.025); // (0.0005 - 0.001) * 50 = -0.025 SOL
        
        let loss_percent = position.calculate_unrealized_pnl_percent(0.0005);
        assert_eq!(loss_percent, -50.0); // 50% loss
    }
    
    #[test]
    fn test_position_exit_conditions() {
        let order = create_test_order();
        let signal = create_test_signal();
        let position = ActivePosition::pure_sniper(&order, &signal, 0.001, 50.0).unwrap();
        
        // Test take profit
        assert_eq!(position.should_close(0.004), Some("take_profit".to_string()));
        
        // Test stop loss
        assert_eq!(position.should_close(0.0002), Some("stop_loss".to_string()));
        
        // Test no exit
        assert_eq!(position.should_close(0.002), None);
    }
}

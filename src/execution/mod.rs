pub mod jupiter_executor;
pub mod jito_executor;
pub mod balance_manager;
pub mod enhanced_executor;

use crate::models::{ExecutionResult, Order, OrderStatus, TradingResult};
use async_trait::async_trait;
use tracing::info;

pub use jupiter_executor::JupiterExecutor;
pub use jito_executor::JitoExecutor;
pub use enhanced_executor::{SniperBotExecutor, ExecutorFactory};
pub use balance_manager::BalanceManager;

#[async_trait]
pub trait OrderExecutor: Send + Sync {
    async fn submit_order(&self, order: &Order) -> TradingResult<String>;
    async fn cancel_order(&self, order_id: &str) -> TradingResult<()>;
    async fn get_order_status(&self, order_id: &str) -> TradingResult<OrderStatus>;
    async fn get_open_orders(&self) -> TradingResult<Vec<Order>>;
    fn get_exchange_name(&self) -> &str;
}

/// Enhanced executor trait for advanced execution features
#[async_trait]
pub trait EnhancedOrderExecutor: Send + Sync {
    /// Execute order with full result details
    async fn execute_order(&self, order: &Order) -> TradingResult<ExecutionResult>;

    /// Execute order with MEV protection
    async fn execute_order_with_mev_protection(&self, order: &Order) -> TradingResult<ExecutionResult>;

    /// Get execution capabilities
    fn supports_mev_protection(&self) -> bool;
    fn supports_bundles(&self) -> bool;
    fn get_executor_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct MockOrderExecutor {
    exchange_name: String,
    dry_run: bool,
}

impl MockOrderExecutor {
    pub fn new(exchange_name: String, dry_run: bool) -> Self {
        Self {
            exchange_name,
            dry_run,
        }
    }
}

#[async_trait]
impl OrderExecutor for MockOrderExecutor {
    async fn submit_order(&self, order: &Order) -> TradingResult<String> {
        if self.dry_run {
            info!(
                "DRY RUN: Would submit order {:?} {} {} @ {} on {}",
                order.side,
                order.size,
                order.symbol,
                order.price.unwrap_or(0.0),
                self.exchange_name
            );
            Ok(format!("dry_run_{}", order.id))
        } else {
            // In a real implementation, this would make actual API calls
            info!(
                "MOCK: Submitting order {:?} {} {} @ {} on {}",
                order.side,
                order.size,
                order.symbol,
                order.price.unwrap_or(0.0),
                self.exchange_name
            );
            Ok(format!("mock_{}", order.id))
        }
    }

    async fn cancel_order(&self, order_id: &str) -> TradingResult<()> {
        if self.dry_run {
            info!("DRY RUN: Would cancel order {} on {}", order_id, self.exchange_name);
        } else {
            info!("MOCK: Cancelling order {} on {}", order_id, self.exchange_name);
        }
        Ok(())
    }

    async fn get_order_status(&self, order_id: &str) -> TradingResult<OrderStatus> {
        info!("Getting status for order {} on {}", order_id, self.exchange_name);
        // Mock implementation - in reality this would query the exchange
        Ok(OrderStatus::Open)
    }

    async fn get_open_orders(&self) -> TradingResult<Vec<Order>> {
        info!("Getting open orders from {}", self.exchange_name);
        // Mock implementation
        Ok(Vec::new())
    }

    fn get_exchange_name(&self) -> &str {
        &self.exchange_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ExecutionParams, OrderSide, OrderType, TimeInForce};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_mock_order_executor() {
        let executor = MockOrderExecutor::new("TestExchange".to_string(), true);
        
        let order = Order {
            id: Uuid::new_v4(),
            exchange_order_id: None,
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            size: 100.0,
            price: Some(50000.0),
            filled_size: 0.0,
            average_fill_price: None,
            status: OrderStatus::Pending,
            exchange: "TestExchange".to_string(),
            strategy: "test_strategy".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            time_in_force: TimeInForce::GTC,
            execution_params: ExecutionParams::default(),
            stop_loss: None,
            take_profit: None,
            max_slippage_bps: 300,
            actual_slippage_bps: None,
            fees_paid: 0.0,
            transaction_signature: None,
            bundle_id: None,
        };

        let result = executor.submit_order(&order).await;
        assert!(result.is_ok());
        
        let order_id = result.unwrap();
        assert!(order_id.starts_with("dry_run_"));
    }
}

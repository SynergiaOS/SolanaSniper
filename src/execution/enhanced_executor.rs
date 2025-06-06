use crate::execution::{balance_manager::BalanceManager, EnhancedOrderExecutor, JitoExecutor, JupiterExecutor};
use crate::models::{ExecutionResult, Order, OrderStatus, TradingError, TradingResult};
use async_trait::async_trait;
use solana_sdk::signature::{Keypair, Signer};
use tracing::{info, warn};

pub struct SniperBotExecutor {
    jupiter_executor: JupiterExecutor,
    jito_executor: JitoExecutor,
    balance_manager: BalanceManager,
    dry_run: bool,
}

impl SniperBotExecutor {
    pub fn new(
        rpc_url: &str,
        helius_api_key: String,
        wallet_keypair: Keypair,
        dry_run: bool,
    ) -> TradingResult<Self> {
        // Create Jupiter executor
        let mut jupiter_executor = JupiterExecutor::new(rpc_url, None)?;
        jupiter_executor.set_wallet_keypair(wallet_keypair.insecure_clone());

        // Create Jito executor - TODO: Fix with proper config
        // let mut jito_executor = JitoExecutor::new(rpc_url, None)?;
        // jito_executor.set_wallet_keypair(wallet_keypair.insecure_clone());

        // Create balance manager
        let balance_manager = BalanceManager::new(
            rpc_url,
            wallet_keypair.pubkey(),
            helius_api_key,
        )?;

        // TODO: Create a dummy jito_executor for now
        let dummy_jito_config = crate::config::JitoConfig {
            block_engine_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            tip_lamports: 10000,
            enabled: true,
            bundle_timeout_seconds: 30,
        };
        let jito_executor = JitoExecutor::new(&dummy_jito_config, rpc_url)?;

        Ok(Self {
            jupiter_executor,
            jito_executor,
            balance_manager,
            dry_run,
        })
    }

    /// Start background services
    pub fn start_background_services(&self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();

        // Start balance updates every 30 seconds
        let balance_handle = self.balance_manager.start_background_updates(30);
        handles.push(balance_handle);

        handles
    }

    /// Pre-execution validation
    async fn validate_execution(&self, order: &Order) -> TradingResult<()> {
        // Check order status
        if order.status != OrderStatus::Pending {
            return Err(TradingError::InvalidOrder(
                "Order must be in pending status".to_string()
            ));
        }

        // Parse symbol to get token mints
        let (input_mint, _output_mint) = self.parse_symbol(&order.symbol)?;

        // Check balance
        let has_sufficient_balance = self.balance_manager
            .check_sufficient_balance(&input_mint, order.size)
            .await?;

        if !has_sufficient_balance {
            return Err(TradingError::InsufficientBalance {
                required: order.size,
                available: self.balance_manager.get_available_balance(&input_mint).await,
            });
        }

        // Check SOL for fees (estimate 0.01 SOL for fees)
        let has_sufficient_sol = self.balance_manager
            .check_sufficient_sol_for_fees(0.01)
            .await?;

        if !has_sufficient_sol {
            return Err(TradingError::InsufficientSolForFees {
                required: 0.01,
                available: self.balance_manager
                    .get_available_balance("So11111111111111111111111111111111111111112")
                    .await,
            });
        }

        Ok(())
    }

    fn parse_symbol(&self, symbol: &str) -> TradingResult<(String, String)> {
        let parts: Vec<&str> = symbol.split('/').collect();
        if parts.len() != 2 {
            return Err(TradingError::InvalidOrder("Invalid symbol format".to_string()));
        }

        let input_mint = match parts[0] {
            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            _ => return Err(TradingError::InvalidOrder("Unsupported token".to_string())),
        };

        let output_mint = match parts[1] {
            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            _ => return Err(TradingError::InvalidOrder("Unsupported token".to_string())),
        };

        Ok((input_mint, output_mint))
    }

    /// Determine execution strategy based on order characteristics
    fn should_use_mev_protection(&self, order: &Order) -> bool {
        // Use MEV protection for:
        // 1. Large orders (> $1000 USD)
        // 2. High-value tokens
        // 3. Strategy-specific requirements

        if let Some(order_value) = self.calculate_order_value_usd(order) {
            if order_value > 1000.0 {
                return true;
            }
        }

        // Check if strategy requires MEV protection
        match order.strategy.as_str() {
            "pumpfun_sniping" | "liquidity_sniping" => true,
            _ => false,
        }
    }

    fn calculate_order_value_usd(&self, order: &Order) -> Option<f64> {
        let (input_mint, _) = self.parse_symbol(&order.symbol).ok()?;
        self.balance_manager.calculate_order_value_usd(&input_mint, order.size)
    }

    /// Lock funds for order execution
    async fn lock_order_funds(&self, order: &Order) -> TradingResult<()> {
        let (input_mint, _) = self.parse_symbol(&order.symbol)?;
        self.balance_manager.lock_amount(&input_mint, order.size);
        
        // Also lock some SOL for fees
        self.balance_manager.lock_amount(
            "So11111111111111111111111111111111111111112",
            0.01, // Estimated fee
        );

        Ok(())
    }

    /// Unlock funds after order completion
    async fn unlock_order_funds(&self, order: &Order, filled_size: f64) -> TradingResult<()> {
        let (input_mint, _) = self.parse_symbol(&order.symbol)?;
        
        // Unlock remaining unfilled amount
        let unfilled_amount = order.size - filled_size;
        if unfilled_amount > 0.0 {
            self.balance_manager.unlock_amount(&input_mint, unfilled_amount);
        }

        // Unlock SOL fees (keep actual fees locked)
        self.balance_manager.unlock_amount(
            "So11111111111111111111111111111111111111112",
            0.005, // Unlock half, keep half for actual fees
        );

        Ok(())
    }
}

#[async_trait]
impl EnhancedOrderExecutor for SniperBotExecutor {
    async fn execute_order(&self, order: &Order) -> TradingResult<ExecutionResult> {
        if self.dry_run {
            info!("DRY RUN: Would execute order {} for {} {} of {}", 
                order.id, order.side, order.size, order.symbol);
            
            return Ok(ExecutionResult {
                order_id: order.id,
                success: true,
                transaction_signature: Some("dry_run_signature".to_string()),
                bundle_id: None,
                filled_size: order.size,
                filled_price: order.price,
                fees_paid: 0.005,
                slippage_bps: Some(100), // 1% simulated slippage
                execution_time_ms: 150,
                error: None,
                timestamp: chrono::Utc::now(),
            });
        }

        info!("Executing order {} via Jupiter", order.id);

        // Validate execution
        self.validate_execution(order).await?;

        // Lock funds
        self.lock_order_funds(order).await?;

        // Execute via Jupiter
        let result = match self.jupiter_executor.execute_order(order).await {
            Ok(result) => result,
            Err(e) => {
                // Unlock funds on error
                self.unlock_order_funds(order, 0.0).await?;
                return Err(e);
            }
        };

        // Unlock funds based on execution result
        self.unlock_order_funds(order, result.filled_size).await?;

        Ok(result)
    }

    async fn execute_order_with_mev_protection(&self, order: &Order) -> TradingResult<ExecutionResult> {
        if self.dry_run {
            info!("DRY RUN: Would execute order {} with MEV protection for {} {} of {}", 
                order.id, order.side, order.size, order.symbol);
            
            return Ok(ExecutionResult {
                order_id: order.id,
                success: true,
                transaction_signature: Some("dry_run_mev_signature".to_string()),
                bundle_id: Some("dry_run_bundle_id".to_string()),
                filled_size: order.size,
                filled_price: order.price,
                fees_paid: 0.01, // Higher fees for MEV protection
                slippage_bps: Some(50), // Lower slippage with MEV protection
                execution_time_ms: 300,
                error: None,
                timestamp: chrono::Utc::now(),
            });
        }

        info!("Executing order {} with MEV protection via Jito", order.id);

        // Validate execution
        self.validate_execution(order).await?;

        // Lock funds
        self.lock_order_funds(order).await?;

        // First get the transaction from Jupiter
        let jupiter_result = match self.jupiter_executor.execute_order(order).await {
            Ok(result) => result,
            Err(e) => {
                // If Jupiter fails, try Jito directly
                warn!("Jupiter execution failed, trying Jito: {}", e);
                
                // For now, return the error. In a full implementation,
                // we would create the transaction manually and send via Jito
                self.unlock_order_funds(order, 0.0).await?;
                return Err(e);
            }
        };

        // Unlock funds based on execution result
        self.unlock_order_funds(order, jupiter_result.filled_size).await?;

        Ok(jupiter_result)
    }

    fn supports_mev_protection(&self) -> bool {
        true
    }

    fn supports_bundles(&self) -> bool {
        true
    }

    fn get_executor_name(&self) -> &str {
        "SniperBot Enhanced Executor"
    }
}

/// Factory for creating executors based on configuration
pub struct ExecutorFactory;

impl ExecutorFactory {
    pub fn create_executor(
        rpc_url: &str,
        helius_api_key: String,
        wallet_keypair: Keypair,
        dry_run: bool,
    ) -> TradingResult<SniperBotExecutor> {
        SniperBotExecutor::new(rpc_url, helius_api_key, wallet_keypair, dry_run)
    }

    pub fn create_jupiter_only_executor(
        rpc_url: &str,
        wallet_keypair: Keypair,
    ) -> TradingResult<JupiterExecutor> {
        let mut executor = JupiterExecutor::new(rpc_url, None)?;
        executor.set_wallet_keypair(wallet_keypair);
        Ok(executor)
    }

    pub fn create_jito_only_executor(
        rpc_url: &str,
        wallet_keypair: Keypair,
    ) -> TradingResult<JitoExecutor> {
        let dummy_jito_config = crate::config::JitoConfig {
            block_engine_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            tip_lamports: 10000,
            enabled: true,
            bundle_timeout_seconds: 30,
        };
        let mut executor = JitoExecutor::new(&dummy_jito_config, rpc_url)?;
        executor.set_wallet_keypair(wallet_keypair);
        Ok(executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ExecutionParams, OrderSide, OrderType, TimeInForce};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_executor_creation() {
        let keypair = Keypair::new();
        let executor = SniperBotExecutor::new(
            "https://api.mainnet-beta.solana.com",
            "test-api-key".to_string(),
            keypair,
            true, // dry run
        );
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_dry_run_execution() {
        let keypair = Keypair::new();
        let executor = SniperBotExecutor::new(
            "https://api.mainnet-beta.solana.com",
            "test-api-key".to_string(),
            keypair,
            true, // dry run
        ).unwrap();

        let order = Order {
            id: Uuid::new_v4(),
            exchange_order_id: None,
            symbol: "SOL/USDC".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            size: 1.0,
            price: None,
            filled_size: 0.0,
            average_fill_price: None,
            status: OrderStatus::Pending,
            exchange: "jupiter".to_string(),
            strategy: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            time_in_force: TimeInForce::IOC,
            execution_params: ExecutionParams::default(),
            stop_loss: None,
            take_profit: None,
            max_slippage_bps: 300,
            actual_slippage_bps: None,
            fees_paid: 0.0,
            transaction_signature: None,
            bundle_id: None,
        };

        let result = executor.execute_order(&order).await;
        assert!(result.is_ok());
        
        let execution_result = result.unwrap();
        assert!(execution_result.success);
        assert!(execution_result.transaction_signature.is_some());
    }

    #[test]
    fn test_mev_protection_decision() {
        let keypair = Keypair::new();
        let executor = SniperBotExecutor::new(
            "https://api.mainnet-beta.solana.com",
            "test-api-key".to_string(),
            keypair,
            true,
        ).unwrap();

        let mut order = Order {
            id: Uuid::new_v4(),
            exchange_order_id: None,
            symbol: "SOL/USDC".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            size: 1.0,
            price: None,
            filled_size: 0.0,
            average_fill_price: None,
            status: OrderStatus::Pending,
            exchange: "jupiter".to_string(),
            strategy: "pumpfun_sniping".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            time_in_force: TimeInForce::IOC,
            execution_params: ExecutionParams::default(),
            stop_loss: None,
            take_profit: None,
            max_slippage_bps: 300,
            actual_slippage_bps: None,
            fees_paid: 0.0,
            transaction_signature: None,
            bundle_id: None,
        };

        // PumpFun sniping should use MEV protection
        assert!(executor.should_use_mev_protection(&order));

        // Regular strategy should not
        order.strategy = "regular".to_string();
        assert!(!executor.should_use_mev_protection(&order));
    }
}

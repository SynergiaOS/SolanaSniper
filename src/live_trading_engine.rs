//! Live Trading Engine for SniperBot 2.0
//! 
//! This module coordinates real-time market data, strategy execution, and order management.
//! It serves as the central orchestrator for live trading operations.
//! 
//! The LiveTradingEngine is now a proper library component that can be imported
//! and used by binary executables. It follows the correct Rust architecture pattern.

use crate::{
    config::AppConfig,
    models::{TradingResult, TradingError, StrategySignal, SignalType, Order},
    execution::{SniperBotExecutor, EnhancedOrderExecutor},
};
use crate::position_management::{PositionManager, ActivePosition};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, debug, instrument};

/// Live Trading Engine - The Heart of SniperBot 2.0
/// 
/// This is the central orchestrator that receives trading decisions and executes them.
/// It follows the proper Rust library pattern - it's a component that can be imported
/// and used by binary executables.
pub struct LiveTradingEngine {
    /// Channel for receiving trading decisions from strategies
    decision_receiver: mpsc::Receiver<StrategySignal>,
    /// Trading executor for order execution
    trading_executor: SniperBotExecutor,
    /// Position manager for tracking active positions
    position_manager: PositionManager,
    /// Configuration
    config: AppConfig,
    /// Dry run mode flag
    dry_run: bool,
}

impl LiveTradingEngine {
    /// Create new live trading engine with dependency injection
    pub fn new(
        decision_receiver: mpsc::Receiver<StrategySignal>,
        trading_executor: SniperBotExecutor,
        position_manager: PositionManager,
        config: AppConfig,
        dry_run: bool,
    ) -> Self {
        Self {
            decision_receiver,
            trading_executor,
            position_manager,
            config,
            dry_run,
        }
    }

    /// Main execution loop - this is the heart of the trading engine
    /// 
    /// This method should be called from a binary executable and will run indefinitely,
    /// processing trading signals and executing orders.
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> TradingResult<()> {
        info!("🚀 Live Trading Engine: AKTYWNY. Oczekiwanie na decyzje handlowe...");
        info!("💰 Mode: {}", if self.dry_run { "DRY RUN" } else { "LIVE TRADING" });
        
        while let Some(signal) = self.decision_receiver.recv().await {
            info!("🎯 Otrzymano nową decyzję handlową: {} {} {} @ ${:.6} (strength: {:.2})",
                signal.strategy, signal.signal_type, signal.symbol, signal.price, signal.strength);

            // Process the trading signal
            if let Err(e) = self.process_trading_signal(&signal).await {
                error!("❌ Błąd podczas przetwarzania sygnału: {}", e);
                continue;
            }
        }
        
        info!("🛑 Kanał decyzyjny zamknięty. Zamykanie Live Trading Engine.");
        Ok(())
    }

    /// Process a single trading signal
    async fn process_trading_signal(&mut self, signal: &StrategySignal) -> TradingResult<()> {
        // Convert signal to order
        let order = Self::signal_to_order(signal)?;
        
        if self.dry_run {
            info!("🔍 DRY RUN: Would execute order {} for {} {} of {}",
                order.id, order.side, order.size, order.symbol);
            return Ok(());
        }

        // Execute the order
        match self.trading_executor.execute_order(&order).await {
            Ok(execution_result) => {
                info!("✅ Transakcja wykonana pomyślnie. ID: {:?}", execution_result.transaction_signature);
                
                // Handle position management based on signal type
                match signal.signal_type {
                    SignalType::Buy => {
                        // After successful BUY, add position to monitoring
                        let active_position = ActivePosition::from_execution(&order, signal, &execution_result)?;
                        if let Err(e) = self.position_manager.add_position(active_position).await {
                            error!("❌ Krytyczny błąd: Nie udało się zapisać aktywnej pozycji do bazy! Błąd: {}", e);
                        } else {
                            info!("✅ Position added to monitoring for order {}", order.id);
                        }
                    }
                    SignalType::Sell => {
                        // After successful SELL, remove position from monitoring
                        if let Some(position_id) = signal.metadata.get("position_id").and_then(|v| v.as_str()) {
                            if let Err(e) = self.position_manager.remove_position(position_id).await {
                                error!("❌ Failed to remove position from monitoring: {}", e);
                            } else {
                                info!("✅ Position {} removed from monitoring", position_id);
                            }
                        }
                    }
                    _ => {
                        debug!("🔍 Signal type {:?} does not require position management", signal.signal_type);
                    }
                }
            }
            Err(e) => {
                error!("❌ Błąd podczas wykonywania transakcji: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Convert strategy signal to order
    fn signal_to_order(signal: &StrategySignal) -> TradingResult<Order> {
        use crate::models::{OrderSide, OrderType, OrderStatus, TimeInForce, ExecutionParams};
        use uuid::Uuid;
        use chrono::Utc;

        let order_side = match signal.signal_type {
            SignalType::Buy => OrderSide::Buy,
            SignalType::Sell => OrderSide::Sell,
            _ => return Err(TradingError::InvalidOrder("Unsupported signal type".to_string())),
        };

        Ok(Order {
            id: Uuid::new_v4(),
            exchange_order_id: None,
            symbol: signal.symbol.clone(),
            side: order_side,
            order_type: OrderType::Market,
            size: signal.size,
            price: Some(signal.price),
            filled_size: 0.0,
            average_fill_price: None,
            status: OrderStatus::Pending,
            exchange: "jupiter".to_string(),
            strategy: signal.strategy.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            time_in_force: TimeInForce::IOC,
            execution_params: ExecutionParams::default(),
            stop_loss: None,
            take_profit: None,
            max_slippage_bps: 300, // 3% max slippage
            actual_slippage_bps: None,
            fees_paid: 0.0,
            transaction_signature: None,
            bundle_id: None,
        })
    }

    /// Get engine status for monitoring
    pub fn get_status(&self) -> EngineStatus {
        EngineStatus {
            is_running: true, // Simplified - in real implementation this would track actual state
            dry_run: self.dry_run,
            processed_signals: 0, // Would be tracked in real implementation
            successful_trades: 0, // Would be tracked in real implementation
            failed_trades: 0, // Would be tracked in real implementation
            active_strategies: 0, // Would be tracked in real implementation
            portfolio_value: 0.0, // Would be tracked in real implementation
        }
    }
}

/// Trading engine status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct EngineStatus {
    pub is_running: bool,
    pub dry_run: bool,
    pub processed_signals: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub active_strategies: u32,
    pub portfolio_value: f64,
}

/// Factory for creating LiveTradingEngine instances
pub struct LiveTradingEngineFactory;

impl LiveTradingEngineFactory {
    /// Create a new LiveTradingEngine with all dependencies
    pub async fn create(
        config: AppConfig,
        dry_run: bool,
    ) -> TradingResult<(LiveTradingEngine, mpsc::Sender<StrategySignal>)> {
        info!("🏭 Creating LiveTradingEngine with all dependencies");

        // Create communication channel
        let (signal_sender, signal_receiver) = mpsc::channel::<StrategySignal>(100);

        // Create trading executor
        let trading_executor = Self::create_trading_executor(&config, dry_run).await?;

        // Create position manager
        let position_manager = Self::create_position_manager(&config, signal_sender.clone()).await?;

        // Create the engine
        let engine = LiveTradingEngine::new(
            signal_receiver,
            trading_executor,
            position_manager,
            config,
            dry_run,
        );

        info!("✅ LiveTradingEngine created successfully");
        Ok((engine, signal_sender))
    }

    /// Create trading executor
    async fn create_trading_executor(config: &AppConfig, dry_run: bool) -> TradingResult<SniperBotExecutor> {
        use crate::execution::ExecutorFactory;
        use solana_sdk::signature::Keypair;

        info!("🔧 Creating trading executor");

        // Get wallet keypair from config
        let wallet_keypair = if let Some(private_key) = &config.solana.private_key {
            let keypair_bytes = bs58::decode(private_key)
                .into_vec()
                .map_err(|e| TradingError::ConfigError(format!("Invalid private key: {}", e)))?;

            Keypair::from_bytes(&keypair_bytes)
                .map_err(|e| TradingError::ConfigError(format!("Failed to create keypair: {}", e)))?
        } else {
            return Err(TradingError::ConfigError("No wallet private key configured".to_string()));
        };

        // Get Helius API key
        let helius_api_key = config.solana.api_key.as_ref()
            .ok_or_else(|| TradingError::ConfigError("No Helius API key configured".to_string()))?
            .clone();

        // Create executor
        ExecutorFactory::create_executor(
            &config.solana.rpc_url,
            helius_api_key,
            wallet_keypair,
            &config.jito,
            dry_run,
        )
    }

    /// Create position manager
    async fn create_position_manager(
        config: &AppConfig,
        signal_sender: mpsc::Sender<StrategySignal>,
    ) -> TradingResult<PositionManager> {
        use crate::data_fetcher::jupiter_client::JupiterClient;
        use redis::Client as RedisClient;

        info!("🔧 Creating position manager");

        // Create Redis client for DragonflyDB
        let redis_client = RedisClient::open(config.database.dragonfly_url.clone())
            .map_err(|e| TradingError::DataError(format!("Failed to create Redis client: {}", e)))?;

        // Create Jupiter client for price fetching
        let jupiter_client = Arc::new(JupiterClient::new(&config.jupiter)?);

        // Create Position Manager
        Ok(PositionManager::new(
            config.clone(),
            redis_client,
            jupiter_client,
            signal_sender,
        ))
    }
}

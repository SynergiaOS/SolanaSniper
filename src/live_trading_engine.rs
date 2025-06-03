use crate::config::Config;
use crate::data_fetcher::realtime_websocket_manager::RealtimeWebSocketManager;
use crate::data_fetcher::data_aggregator::AggregatedMarketData;
use crate::models::{MarketEvent, StrategySignal, Portfolio, TradingResult, TradingError, MarketData, DataSource};
use crate::strategy::{
    StrategyManager, StrategyContext, PumpFunSnipingStrategy, LiquidityPoolSnipingStrategy,
    MarketConditions, VolumeTrend, PriceMomentum
};
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tracing::{info, warn, error, debug, instrument};

/// Live Trading Engine that coordinates WebSocket data, strategies, and execution
pub struct LiveTradingEngine {
    config: Config,
    websocket_manager: RealtimeWebSocketManager,
    strategy_manager: StrategyManager,
    portfolio: Arc<RwLock<Portfolio>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
    dry_run: bool,
}

impl LiveTradingEngine {
    /// Create new live trading engine
    pub fn new(config: Config, dry_run: bool) -> TradingResult<Self> {
        // Create channels for communication
        let (market_event_sender, market_event_receiver) = mpsc::channel::<MarketEvent>(1000);
        let (signal_sender, signal_receiver) = mpsc::channel::<StrategySignal>(100);

        // Create WebSocket manager
        let websocket_manager = RealtimeWebSocketManager::new(
            config.websocket.clone(),
            market_event_sender,
        );

        // Create strategy manager
        let strategy_manager = StrategyManager::new(signal_sender);

        // Initialize portfolio
        let portfolio = Arc::new(RwLock::new(Portfolio {
            total_value: config.trading.initial_balance,
            available_balance: config.trading.initial_balance,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: vec![],
            daily_pnl: 0.0,
            max_drawdown: 0.0,
            updated_at: Utc::now(),
        }));

        Ok(Self {
            config,
            websocket_manager,
            strategy_manager,
            portfolio,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            dry_run,
        })
    }

    /// Start the live trading engine
    #[instrument(skip(self))]
    pub async fn start(&self) -> TradingResult<()> {
        if self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(TradingError::ConfigError("Engine already running".to_string()));
        }

        self.is_running.store(true, std::sync::atomic::Ordering::SeqCst);

        info!("🚀 Starting SniperBot 2.0 Live Trading Engine");
        info!("💰 Mode: {}", if self.dry_run { "DRY RUN" } else { "LIVE TRADING" });
        info!("💵 Initial Balance: ${:.2}", self.config.trading.initial_balance);

        // Initialize strategies
        self.initialize_strategies().await?;

        // Create channels for internal communication
        let (market_event_sender, mut market_event_receiver) = mpsc::channel::<MarketEvent>(1000);
        let (signal_sender, mut signal_receiver) = mpsc::channel::<StrategySignal>(100);

        // Start WebSocket manager
        let ws_manager = RealtimeWebSocketManager::new(
            self.config.websocket.clone(),
            market_event_sender,
        );

        let ws_handle = tokio::spawn(async move {
            if let Err(e) = ws_manager.start().await {
                error!("WebSocket manager failed: {}", e);
            }
        });

        // Start strategy manager with new signal sender
        let strategy_manager = Arc::new(StrategyManager::new(signal_sender));
        self.initialize_strategies_for_manager(&strategy_manager).await?;

        // Start periodic analysis task
        let periodic_analysis_handle = self.start_periodic_analysis(strategy_manager.clone()).await;

        // Start market event processing task
        let portfolio_clone = self.portfolio.clone();
        let strategy_manager_clone = strategy_manager.clone();
        let market_event_handle = tokio::spawn(async move {
            Self::process_market_events(
                &mut market_event_receiver,
                &strategy_manager_clone,
                &portfolio_clone,
            ).await;
        });

        // Start signal processing task
        let signal_processing_handle = tokio::spawn(async move {
            Self::process_trading_signals(&mut signal_receiver).await;
        });

        info!("✅ Live Trading Engine started successfully");

        // Wait for all tasks to complete (they should run indefinitely)
        tokio::select! {
            _ = ws_handle => warn!("WebSocket manager stopped"),
            _ = market_event_handle => warn!("Market event processor stopped"),
            _ = signal_processing_handle => warn!("Signal processor stopped"),
            _ = periodic_analysis_handle => warn!("Periodic analysis stopped"),
        }

        Ok(())
    }

    /// Stop the live trading engine
    pub fn stop(&self) {
        info!("🛑 Stopping Live Trading Engine");
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        self.websocket_manager.stop();
    }

    /// Initialize trading strategies
    pub async fn initialize_strategies(&self) -> TradingResult<()> {
        info!("📊 Initializing trading strategies");

        // Add PumpFun sniping strategy
        let pumpfun_strategy = Box::new(PumpFunSnipingStrategy::new("pumpfun_sniping".to_string()));
        self.strategy_manager.add_strategy(pumpfun_strategy).await?;

        // Add Liquidity Pool sniping strategy
        let liquidity_strategy = Box::new(LiquidityPoolSnipingStrategy::new("liquidity_sniping".to_string()));
        self.strategy_manager.add_strategy(liquidity_strategy).await?;

        info!("✅ Strategies initialized: {:?}", self.strategy_manager.get_all_strategies().await);
        Ok(())
    }

    /// Initialize strategies for a specific strategy manager
    async fn initialize_strategies_for_manager(&self, strategy_manager: &Arc<StrategyManager>) -> TradingResult<()> {
        info!("📊 Initializing strategies for manager");

        // Add PumpFun sniping strategy
        let pumpfun_strategy = Box::new(PumpFunSnipingStrategy::new("pumpfun_sniping".to_string()));
        strategy_manager.add_strategy(pumpfun_strategy).await?;

        // Add Liquidity Pool sniping strategy
        let liquidity_strategy = Box::new(LiquidityPoolSnipingStrategy::new("liquidity_sniping".to_string()));
        strategy_manager.add_strategy(liquidity_strategy).await?;

        info!("✅ Strategies initialized for manager");
        Ok(())
    }

    /// Start periodic analysis task
    async fn start_periodic_analysis(&self, strategy_manager: Arc<StrategyManager>) -> tokio::task::JoinHandle<()> {
        let portfolio = self.portfolio.clone();
        let is_running = self.is_running.clone();
        let analysis_interval = Duration::from_secs(self.config.trading.analysis_interval_seconds);

        tokio::spawn(async move {
            let mut interval = interval(analysis_interval);
            
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                interval.tick().await;
                
                debug!("🔄 Running periodic analysis");
                
                // Create mock context for periodic analysis
                let context = Self::create_mock_strategy_context(&portfolio).await;
                
                match strategy_manager.run_periodic_analysis(&context).await {
                    Ok(signals) => {
                        if !signals.is_empty() {
                            info!("📈 Periodic analysis generated {} signals", signals.len());
                        }
                    }
                    Err(e) => {
                        warn!("Periodic analysis error: {}", e);
                    }
                }
            }
            
            info!("⏹️ Periodic analysis task stopped");
        })
    }

    /// Process market events and generate trading signals
    async fn process_market_events(
        event_receiver: &mut mpsc::Receiver<MarketEvent>,
        strategy_manager: &Arc<StrategyManager>,
        portfolio: &Arc<RwLock<Portfolio>>,
    ) {
        info!("📡 Starting market event processor");
        
        while let Some(event) = event_receiver.recv().await {
            debug!("📨 Processing market event: {:?}", std::mem::discriminant(&event));
            
            // Create strategy context from current portfolio state
            let context = Self::create_mock_strategy_context(portfolio).await;
            
            // Process event through strategy manager
            match strategy_manager.process_market_event(&event, &context).await {
                Ok(signals) => {
                    if !signals.is_empty() {
                        info!("🎯 Market event generated {} signals", signals.len());
                        for signal in signals {
                            info!("📊 Signal: {} {} {} @ ${:.6} (strength: {:.2})", 
                                signal.strategy, signal.signal_type, signal.symbol, signal.price, signal.strength);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error processing market event: {}", e);
                }
            }
        }
        
        info!("📡 Market event processor stopped");
    }

    /// Process trading signals (placeholder for execution)
    async fn process_trading_signals(signal_receiver: &mut mpsc::Receiver<StrategySignal>) {
        info!("⚡ Starting signal processor");
        
        while let Some(signal) = signal_receiver.recv().await {
            info!("🎯 Processing signal: {} {} {} @ ${:.6} (strength: {:.2})", 
                signal.strategy, signal.signal_type, signal.symbol, signal.price, signal.strength);
            
            // TODO: Integrate with execution engine
            // For now, just log the signal
            info!("💡 Signal would be executed here (DRY RUN mode)");
        }
        
        info!("⚡ Signal processor stopped");
    }

    /// Create mock strategy context for testing
    async fn create_mock_strategy_context(portfolio: &Arc<RwLock<Portfolio>>) -> StrategyContext {
        let portfolio_state = portfolio.read().await.clone();
        
        // Create mock market data
        let market_data = MarketData {
            symbol: "SOL/USDC".to_string(),
            price: 100.0,
            volume: 1000000.0,
            bid: Some(99.95),
            ask: Some(100.05),
            timestamp: Utc::now(),
            source: DataSource::Solana,
        };
        
        let aggregated_data = AggregatedMarketData {
            primary_data: market_data,
            secondary_data: vec![],
            sources_count: 1,
            confidence_score: 0.8,
            latency_ms: 100,
        };
        
        let market_conditions = MarketConditions {
            volatility: 0.15,
            volume_trend: VolumeTrend::Increasing,
            price_momentum: PriceMomentum::Bullish,
            liquidity_depth: 50000.0,
            market_cap: Some(1000000.0),
            age_hours: Some(12.0),
        };
        
        StrategyContext::new(aggregated_data, portfolio_state, market_conditions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_live_trading_engine_creation() {
        let config = Config::default();
        let engine = LiveTradingEngine::new(config, true);
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_strategy_initialization() {
        let config = Config::default();
        let engine = LiveTradingEngine::new(config, true).unwrap();
        
        // Test that we can initialize strategies
        let result = engine.initialize_strategies().await;
        assert!(result.is_ok());
    }
}

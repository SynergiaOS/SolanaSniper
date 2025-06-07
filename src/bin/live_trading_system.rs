//! Live Trading System - SniperBot 2.0 Main Binary
//! 
//! This is the main executable that orchestrates all components of SniperBot 2.0.
//! It follows the correct Rust architecture pattern where the binary imports
//! and uses components from the library.

use sniper_bot::{
    config::AppConfig,
    live_trading_engine::LiveTradingEngineFactory,
    strategy::{StrategyManager, PureSniperStrategy},
    data_fetcher::realtime_websocket_manager::RealtimeWebSocketManager,
    models::MarketEvent,
};
use tokio::sync::mpsc;
use tracing::{info, error, warn, debug};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 SniperBot 2.0 - Live Trading System Starting...");
    info!("🧠 MEMORY AND DISCIPLINE - Position Management Active");
    info!("⚡ REFLEX CORE - Ultra-Fast Execution Ready");

    // Load configuration
    let config = AppConfig::from_env();

    // Determine if this is a dry run
    let dry_run = std::env::var("DRY_RUN")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if dry_run {
        warn!("🔍 RUNNING IN DRY RUN MODE - No real trades will be executed");
    } else {
        info!("💰 LIVE TRADING MODE - Real money at risk!");
    }

    // Create shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    // Handle Ctrl+C gracefully
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        info!("🛑 Shutdown signal received. Gracefully shutting down...");
        shutdown_clone.store(true, Ordering::Relaxed);
    });

    // Create the main system
    let result = run_trading_system(config, dry_run, shutdown).await;

    match result {
        Ok(_) => {
            info!("✅ SniperBot 2.0 shut down gracefully");
            Ok(())
        }
        Err(e) => {
            error!("❌ SniperBot 2.0 encountered an error: {}", e);
            Err(e)
        }
    }
}

/// Main trading system orchestrator
async fn run_trading_system(
    config: AppConfig,
    dry_run: bool,
    shutdown: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🏗️ Initializing SniperBot 2.0 components...");

    // Create communication channels
    let (market_event_sender, mut market_event_receiver) = mpsc::channel::<MarketEvent>(1000);

    // Create LiveTradingEngine with all dependencies
    let (mut live_trading_engine, signal_sender) = LiveTradingEngineFactory::create(config.clone(), dry_run).await
        .map_err(|e| {
            error!("❌ Failed to create LiveTradingEngine: {}", e);
            e
        })?;

    info!("✅ LiveTradingEngine created successfully");

    // Create Strategy Manager
    let strategy_manager = Arc::new(StrategyManager::new(signal_sender.clone()));
    
    // Initialize strategies
    initialize_strategies(&strategy_manager, &config).await?;

    // Create WebSocket Manager for real-time data
    let websocket_manager = RealtimeWebSocketManager::new(
        config.websocket.clone(),
        market_event_sender.clone(),
    );

    info!("✅ All components initialized successfully");

    // Start WebSocket data streaming
    let websocket_handle = tokio::spawn(async move {
        if let Err(e) = websocket_manager.start().await {
            error!("❌ WebSocket streaming failed: {}", e);
        }
    });

    // Start market event processing
    let strategy_manager_clone = Arc::clone(&strategy_manager);
    let market_event_handle = tokio::spawn(async move {
        process_market_events(&mut market_event_receiver, &strategy_manager_clone).await;
    });

    // Start the LiveTradingEngine
    let trading_engine_handle = tokio::spawn(async move {
        if let Err(e) = live_trading_engine.run().await {
            error!("❌ LiveTradingEngine failed: {}", e);
        }
    });

    info!("🎯 SniperBot 2.0 is now FULLY OPERATIONAL!");
    info!("👁️  Monitoring markets for opportunities...");
    info!("🧠 Position management active");
    info!("⚡ Ready for ultra-fast execution");

    // Main monitoring loop
    while !shutdown.load(Ordering::Relaxed) {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Here we could add periodic health checks, metrics reporting, etc.
        debug!("💓 System heartbeat - all components running");
    }

    info!("🛑 Shutdown initiated. Stopping all components...");

    // Cancel all tasks
    websocket_handle.abort();
    market_event_handle.abort();
    trading_engine_handle.abort();

    // Wait a moment for graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    info!("✅ All components stopped. SniperBot 2.0 shutdown complete.");
    Ok(())
}

/// Initialize trading strategies
async fn initialize_strategies(
    strategy_manager: &Arc<StrategyManager>,
    _config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🧠 Initializing trading strategies...");

    // Initialize Pure Sniper Strategy
    let pure_sniper = PureSniperStrategy::new("pure_sniper".to_string());
    strategy_manager.add_strategy(Box::new(pure_sniper)).await
        .map_err(|e| {
            error!("❌ Failed to add Pure Sniper strategy: {}", e);
            e
        })?;

    info!("✅ Pure Sniper Strategy initialized");

    // TODO: Add other strategies here:
    // - Cautious Sniper Strategy
    // - Momentum Trader Strategy  
    // - DLMM Fee Harvester Strategy

    info!("✅ All strategies initialized successfully");
    Ok(())
}

/// Process market events and generate trading signals
async fn process_market_events(
    market_event_receiver: &mut mpsc::Receiver<MarketEvent>,
    strategy_manager: &Arc<StrategyManager>,
) {
    info!("👁️ Market event processor started");

    while let Some(market_event) = market_event_receiver.recv().await {
        debug!("📊 Processing market event: {:?}", market_event);

        // Create mock strategy context for event processing
        let context = create_mock_strategy_context();

        // Process the event with all strategies
        if let Err(e) = strategy_manager.process_market_event(&market_event, &context).await {
            error!("❌ Failed to process market event: {}", e);
            continue;
        }

        // Log successful processing
        match &market_event {
            MarketEvent::NewPoolCreated { pool_address, .. } => {
                info!("🎯 New pool detected: {} - strategies notified", pool_address);
            }
            MarketEvent::PriceUpdate { symbol, price, .. } => {
                debug!("💰 Price update: {} @ ${:.6}", symbol, price);
            }
            MarketEvent::NewTokenListing { token_address, symbol, .. } => {
                info!("📈 New token listing detected: {} ({:?}) - strategies notified",
                    token_address, symbol);
            }
            MarketEvent::LiquidityUpdate { pool_address, liquidity_a, .. } => {
                debug!("💧 Liquidity update: {} - liquidity: {}", pool_address, liquidity_a);
            }
            MarketEvent::WhaleAlert { amount_usd, .. } => {
                info!("🐋 Whale alert: ${:.2} transaction detected", amount_usd);
            }
            _ => {
                debug!("📊 Market event processed: {:?}", std::mem::discriminant(&market_event));
            }
        }
    }

    warn!("📡 Market event channel closed. Event processing stopped.");
}

/// Create mock strategy context for event processing
fn create_mock_strategy_context() -> sniper_bot::strategy::StrategyContext {
    use sniper_bot::{
        strategy::{StrategyContext, MarketConditions, VolumeTrend, PriceMomentum},
        data_fetcher::data_aggregator::AggregatedMarketData,
        models::{Portfolio, MarketData, DataSource},
    };
    use chrono::Utc;

    // Create mock market data
    let market_data = MarketData {
        symbol: "MOCK/SOL".to_string(),
        price: 0.001,
        volume: 50000.0,
        bid: Some(0.0009),
        ask: Some(0.0011),
        timestamp: Utc::now(),
        source: DataSource::Solana,
    };

    let aggregated_data = AggregatedMarketData {
        primary_data: market_data,
        secondary_data: vec![],
        sources_count: 1,
        confidence_score: 0.8,
        latency_ms: 150,
    };

    let portfolio = Portfolio {
        total_value: 10000.0,
        total_value_usd: Some(10000.0),
        available_balance: 5000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
        positions: vec![],
        daily_pnl: 0.0,
        max_drawdown: 0.0,
        updated_at: Utc::now(),
    };

    let market_conditions = MarketConditions {
        volatility: 0.15,
        volume_trend: VolumeTrend::Increasing,
        price_momentum: PriceMomentum::Bullish,
        liquidity_depth: 50000.0,
        market_cap: Some(500000.0),
        age_hours: Some(2.0),
    };

    StrategyContext::new(aggregated_data, portfolio, market_conditions)
}

/// Display system status and statistics
fn display_startup_banner() {
    println!("
╔══════════════════════════════════════════════════════════════╗
║                    🎯 SNIPERBOT 2.0 🎯                      ║
║                  LIVE TRADING SYSTEM                         ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  🧠 INTELLIGENCE BRAIN: Advanced Analysis & Decision Making  ║
║  ⚡ REFLEX CORE: Ultra-Fast New Token Detection             ║
║  💾 MEMORY & DISCIPLINE: Position Management System         ║
║  🛡️ MEV PROTECTION: Jito Bundle Integration                ║
║                                                              ║
║  🎯 STRATEGIES:                                             ║
║    • Pure Sniper (0.05 SOL, +300% TP, -80% SL, 1h)        ║
║    • Cautious Sniper (0.1 SOL, +200% TP, -60% SL, 2h)     ║
║    • Momentum Trader (Dynamic, Trailing SL, 24h)           ║
║    • DLMM Fee Harvester (Passive Income, 1 week)           ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
");
}

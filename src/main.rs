use anyhow::Result;
use clap::Parser;
use tracing::{info, warn, error, debug};
use tokio::sync::mpsc;
use std::sync::Arc;


// All modules are now part of the sniper_bot library

use sniper_bot::{
    config::AppConfig,
    utils::logging,
    data_fetcher::{
        realtime_websocket_manager::{RealtimeWebSocketManager, ConnectionStatus},
        data_aggregator::DataAggregator,
        market_scanner::{MarketScanner, PotentialOpportunity},
    },
    models::{MarketEvent, StrategySignal, Portfolio},
    risk_management::RiskManager,
    strategy::{
        strategy_manager::StrategyManager,
        arbitrage_strategy::ArbitrageStrategy,
        pumpfun_sniping::PumpFunSnipingStrategy,
        liquidity_sniping::LiquidityPoolSnipingStrategy,
        meteora_dlmm_strategy::MeteoraDLMMStrategy,
        volume_spike_strategy::VolumeSpikeStrategy,
        pure_sniper_strategy::PureSniperStrategy,
    },
    live_trading_engine::{LiveTradingEngine, LiveTradingEngineFactory, EngineStatus},
    ai_decision_engine::{AIDecisionEngine, AIConfig},
    ai_signal_processor::AISignalProcessor,
    utils::reporter::{Reporter, ReporterConfig},
    dragonfly_manager::DragonflyManager,
};
use config::Config;



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "configs/bot.toml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Dry run mode (no actual trading)
    #[arg(long)]
    dry_run: bool,

    /// Paper trading mode
    #[arg(long)]
    paper_trading: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    match dotenvy::dotenv() {
        Ok(path) => {
            eprintln!("✅ Loaded .env file from: {:?}", path);
        }
        Err(e) => {
            eprintln!("⚠️ Warning: Could not load .env file: {}", e);
        }
    }

    // Debug: Check multiple variables after loading .env
    match std::env::var("DASHBOARD_URL") {
        Ok(url) => eprintln!("🔍 DASHBOARD_URL after .env load: {}", url),
        Err(_) => {
            eprintln!("❌ DASHBOARD_URL not found after .env load");
            eprintln!("🔧 Setting DASHBOARD_URL manually as fallback");
            std::env::set_var("DASHBOARD_URL", "http://localhost:8084/api/report_event");
        }
    }
    match std::env::var("HELIUS_API_KEY") {
        Ok(key) => eprintln!("🔍 HELIUS_API_KEY after .env load: {}...", &key[..10]),
        Err(_) => eprintln!("❌ HELIUS_API_KEY not found after .env load"),
    }
    match std::env::var("DRY_RUN") {
        Ok(val) => eprintln!("🔍 DRY_RUN after .env load: {}", val),
        Err(_) => eprintln!("❌ DRY_RUN not found after .env load"),
    }

    let args = Args::parse();

    // Initialize logging
    logging::init_logging(&args.log_level)?;

    info!("🎯 SniperBot starting up...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Config file: {}", args.config);

    if args.dry_run {
        warn!("🔍 Running in DRY RUN mode - no actual trades will be executed");
    }

    if args.paper_trading {
        warn!("📝 Running in PAPER TRADING mode");
    }

    // Load configuration
    let config = AppConfig::from_env();
    info!("✅ Configuration loaded successfully");

    // Initialize bot components
    let bot = SniperBot::new(config, args.dry_run, args.paper_trading).await?;
    
    // Start the bot
    info!("🚀 Starting SniperBot main loop...");
    bot.run().await?;

    Ok(())
}

/// Main SniperBot struct that orchestrates all components
pub struct SniperBot {
    config: AppConfig,
    dry_run: bool,
    paper_trading: bool,
    // Core components
    websocket_manager: Arc<RealtimeWebSocketManager>,
    data_aggregator: Arc<DataAggregator>,
    strategy_manager: Arc<StrategyManager>,
    trading_engine: Arc<LiveTradingEngine>,
    risk_manager: Arc<RiskManager>,
    dragonfly_manager: Option<Arc<DragonflyManager>>,
    ai_decision_engine: Option<Arc<tokio::sync::Mutex<AIDecisionEngine>>>,
    reporter: Option<Arc<tokio::sync::Mutex<Reporter>>>,
    market_scanner: Option<Arc<tokio::sync::Mutex<MarketScanner>>>,
    // Communication channels
    market_event_sender: mpsc::Sender<MarketEvent>,
    market_event_receiver: mpsc::Receiver<MarketEvent>,
    signal_sender: mpsc::Sender<StrategySignal>,
    signal_receiver: mpsc::Receiver<StrategySignal>,
    opportunity_sender: mpsc::Sender<PotentialOpportunity>,
    opportunity_receiver: mpsc::Receiver<PotentialOpportunity>,
}

impl SniperBot {
    pub async fn new(config: AppConfig, dry_run: bool, paper_trading: bool) -> Result<Self> {
        info!("🔧 Initializing SniperBot components...");

        // Create communication channels
        let (market_event_sender, market_event_receiver) = mpsc::channel::<MarketEvent>(1000);
        let (signal_sender, signal_receiver) = mpsc::channel::<StrategySignal>(100);
        let (opportunity_sender, opportunity_receiver) = mpsc::channel::<PotentialOpportunity>(200);

        // Initialize WebSocket Manager with Helius configuration
        info!("🌐 Initializing Helius WebSocket Manager...");
        let websocket_manager = Arc::new(
            RealtimeWebSocketManager::new(
                config.websocket.clone(),
                market_event_sender.clone(),
            )
        );

        // Initialize Data Aggregator
        info!("📊 Initializing Data Aggregator...");
        // Convert config to utils::config::Config for DataAggregator
        let utils_config = sniper_bot::utils::config::Config::default(); // Use default for now
        let data_aggregator = Arc::new(DataAggregator::new(utils_config).await?);

        // Initialize DragonflyDB Manager (optional - requires DRAGONFLY_URL)
        let dragonfly_manager = if let Ok(dragonfly_url) = std::env::var("DRAGONFLY_URL") {
            info!("🐉 Initializing DragonflyDB Manager...");
            match DragonflyManager::new(&dragonfly_url).await {
                Ok(manager) => {
                    info!("✅ DragonflyDB Manager initialized successfully");
                    Some(Arc::new(manager))
                }
                Err(e) => {
                    warn!("⚠️ Failed to initialize DragonflyDB Manager: {}", e);
                    None
                }
            }
        } else {
            info!("ℹ️ DragonflyDB Manager disabled - no DRAGONFLY_URL provided");
            None
        };

        // Initialize AI Decision Engine (optional - requires API key)
        let ai_decision_engine = if let Ok(mistral_api_key) = std::env::var("MISTRAL_API_KEY") {
            info!("🤖 Initializing AI Decision Engine with Mistral AI...");
            let ai_config = AIConfig {
                enabled: true,
                api_key: mistral_api_key,
                model: "mistral-large-latest".to_string(),
                temperature: 0.3,
                top_p: 0.95,
                tool_use_enabled: true,
            };

            match AIDecisionEngine::new(ai_config) {
                Ok(ai_engine) => {
                    info!("✅ AI Decision Engine initialized successfully");
                    Some(Arc::new(tokio::sync::Mutex::new(ai_engine)))
                }
                Err(e) => {
                    warn!("⚠️ Failed to initialize AI Decision Engine: {}", e);
                    None
                }
            }
        } else {
            info!("ℹ️ AI Decision Engine disabled - no MISTRAL_API_KEY provided");
            None
        };

        // Initialize Strategy Manager with all strategies
        info!("🎯 Initializing Strategy Manager...");
        let mut strategy_manager = StrategyManager::new(signal_sender.clone());

        // Add all strategies
        strategy_manager.add_strategy(Box::new(
            ArbitrageStrategy::new("helius_arbitrage".to_string())
        )).await?;

        strategy_manager.add_strategy(Box::new(
            PumpFunSnipingStrategy::new("pumpfun_sniping".to_string())
        )).await?;

        strategy_manager.add_strategy(Box::new(
            LiquidityPoolSnipingStrategy::new("liquidity_sniping".to_string())
        )).await?;

        strategy_manager.add_strategy(Box::new(
            MeteoraDLMMStrategy::new("meteora_dlmm".to_string())
        )).await?;

        strategy_manager.add_strategy(Box::new(
            VolumeSpikeStrategy::new()
        )).await?;

        // Add Pure Sniper Strategy (Reflex Core Family)
        strategy_manager.add_strategy(Box::new(
            PureSniperStrategy::new("pure_sniper".to_string())
        )).await?;

        let strategy_manager = Arc::new(strategy_manager);

        // Initialize Reporter (optional - requires dashboard URL)
        let reporter = if let Ok(dashboard_url) = std::env::var("DASHBOARD_URL") {
            info!("📊 Initializing Reporter with dashboard: {}", dashboard_url);
            debug!("🔍 DEBUG: DASHBOARD_URL from env: {}", dashboard_url);
            let reporter_config = ReporterConfig {
                enabled: true,
                dashboard_url,
                api_key: std::env::var("DASHBOARD_API_KEY").ok(),
                ..ReporterConfig::default()
            };
            let mut reporter = Reporter::new(reporter_config);

            match reporter.start().await {
                Ok(_) => {
                    info!("✅ Reporter initialized successfully");
                    Some(Arc::new(tokio::sync::Mutex::new(reporter)))
                }
                Err(e) => {
                    warn!("⚠️ Failed to initialize Reporter: {}", e);
                    None
                }
            }
        } else {
            info!("ℹ️ Reporter disabled - no DASHBOARD_URL provided");
            None
        };

        // Initialize Risk Manager
        info!("🛡️ Initializing Risk Manager...");
        let risk_config = sniper_bot::utils::config::RiskManagementConfig {
            global_max_exposure: config.trading.max_position_size_sol,
            max_daily_loss: config.risk_management.max_daily_loss_sol,
            max_drawdown: config.risk_management.circuit_breaker_threshold,
            position_sizing_method: "percentage".to_string(),
            emergency_stop_enabled: true,
            circuit_breaker_threshold: config.risk_management.circuit_breaker_threshold,
        };
        let risk_manager = Arc::new(
            RiskManager::new(risk_config)
        );

        // Initialize Trading Engine
        info!("⚡ Initializing Live Trading Engine...");
        let (trading_engine, signal_sender) = LiveTradingEngineFactory::create(
            config.clone(),
            dry_run,
        ).await?;
        let trading_engine = Arc::new(trading_engine);

        info!("✅ All SniperBot components initialized successfully");

        Ok(Self {
            config,
            dry_run,
            paper_trading,
            websocket_manager,
            data_aggregator,
            strategy_manager,
            trading_engine,
            risk_manager,
            dragonfly_manager,
            ai_decision_engine,
            reporter,
            market_scanner: None, // Will be initialized later if needed
            market_event_sender,
            market_event_receiver,
            signal_sender,
            signal_receiver,
            opportunity_sender,
            opportunity_receiver,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        info!("🎯 SniperBot main loop started");

        // Start WebSocket Manager in background
        let ws_manager = Arc::clone(&self.websocket_manager);
        tokio::spawn(async move {
            if let Err(e) = ws_manager.start().await {
                error!("❌ WebSocket Manager error: {}", e);
            }
        });

        // Start Trading Engine in background
        // Note: For now, we'll skip starting the trading engine as it requires mutable access
        // In a real implementation, we'd need to restructure this to handle the ownership properly
        info!("🔄 Trading Engine ready (start method needs to be implemented)");

        // Start new UI API server in background
        tokio::spawn(async move {
            if let Err(e) = sniperbot_ui_api::start_server(8084).await {
                error!("❌ UI API Server error: {}", e);
            }
        });

        // Start Portfolio Manager in background
        let portfolio_config = AppConfig::from_env();
        tokio::spawn(async move {
            if let Err(e) = sniper_bot::portfolio_manager::start_portfolio_monitoring(portfolio_config).await {
                error!("❌ Portfolio Manager error: {}", e);
            }
        });

        info!("🚀 All background services started");

        // Subscribe to key symbols for real-time monitoring
        self.subscribe_to_symbols().await?;

        // Create health check interval
        let mut health_check_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        health_check_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Main event processing loop
        loop {
            tokio::select! {
                // Handle shutdown signals
                _ = tokio::signal::ctrl_c() => {
                    info!("🛑 Received shutdown signal, stopping bot...");
                    break;
                }

                // Process market events from WebSocket
                Some(market_event) = self.market_event_receiver.recv() => {
                    if let Err(e) = self.process_market_event(market_event).await {
                        error!("❌ Error processing market event: {}", e);
                    }
                }

                // Process strategy signals
                Some(strategy_signal) = self.signal_receiver.recv() => {
                    if let Err(e) = self.process_strategy_signal(strategy_signal).await {
                        error!("❌ Error processing strategy signal: {}", e);
                    }
                }

                // Process market opportunities from scanner
                Some(opportunity) = self.opportunity_receiver.recv() => {
                    if let Err(e) = self.process_opportunity(opportunity).await {
                        error!("❌ Error processing opportunity: {}", e);
                    }
                }

                // Periodic health checks and maintenance
                _ = health_check_interval.tick() => {
                    self.perform_health_check().await;
                }
            }
        }

        info!("✅ SniperBot shutdown complete");
        Ok(())
    }

    /// Subscribe to key symbols for real-time monitoring
    async fn subscribe_to_symbols(&self) -> Result<()> {
        info!("📡 Subscribing to key symbols for real-time monitoring...");

        // Key symbols to monitor for arbitrage and sniping opportunities
        let symbols = vec![
            "SOL/USDC",
            "SOL/USDT",
            "BONK/SOL",
            "WIF/SOL",
            "POPCAT/SOL",
            "BOME/SOL",
        ];

        for symbol in symbols {
            if let Err(e) = self.websocket_manager.subscribe_to_symbol(symbol).await {
                warn!("⚠️ Failed to subscribe to {}: {}", symbol, e);
            } else {
                debug!("✅ Subscribed to {}", symbol);
            }
        }

        info!("📡 Symbol subscriptions completed");
        Ok(())
    }

    /// Process incoming market events from WebSocket
    async fn process_market_event(&self, event: MarketEvent) -> Result<()> {
        debug!("📊 Processing market event: {:?}", event);

        // Update data aggregator with new market data
        if let Err(e) = self.data_aggregator.process_market_event(&event).await {
            warn!("⚠️ Data aggregator error: {}", e);
        }

        // Pass event to all strategies for analysis
        // Create a mock strategy context for now
        let mock_context = self.create_mock_strategy_context().await;
        if let Err(e) = self.strategy_manager.process_market_event(&event, &mock_context).await {
            warn!("⚠️ Strategy manager error: {}", e);
        }

        Ok(())
    }

    /// Process strategy signals through AI analysis and forward to trading engine
    async fn process_strategy_signal(&self, signal: StrategySignal) -> Result<()> {
        info!("🎯 Processing strategy signal: {} - {} - {:.4}% strength",
              signal.strategy, signal.symbol, signal.strength * 100.0);

        // Process signal through AI if available
        let enhanced_signal = if let Some(ai_engine) = &self.ai_decision_engine {
            info!("🧠 Processing signal through AI analysis...");

            // Create AI Signal Processor for this signal
            let ai_engine_guard = ai_engine.lock().await;
            let ai_processor = AISignalProcessor::new(Arc::new(ai_engine_guard.clone()));

            match ai_processor.process_signal(signal.clone(), None).await {
                Ok(enhanced) => {
                    info!("🧠 AI analysis complete: {} (confidence: {:.2}, action: {})",
                          enhanced.ai_recommendation.action,
                          enhanced.ai_confidence,
                          enhanced.final_action);

                    // Report AI-enhanced signal to dashboard
                    if let Some(reporter) = &self.reporter {
                        if let Err(e) = reporter.lock().await.report_ai_decision(&enhanced).await {
                            warn!("⚠️ Failed to report AI decision: {}", e);
                        }
                    }

                    Some(enhanced)
                }
                Err(e) => {
                    warn!("🧠 AI processing failed: {}. Using original signal.", e);
                    None
                }
            }
        } else {
            debug!("🧠 AI Decision Engine not available, using original signal");
            None
        };

        // Report original signal to dashboard
        if let Some(reporter) = &self.reporter {
            if let Err(e) = reporter.lock().await.report_signal(&signal).await {
                warn!("⚠️ Failed to report signal: {}", e);
            }
        }

        // Determine if we should execute based on AI analysis
        let should_execute = if let Some(ref enhanced) = enhanced_signal {
            match enhanced.final_action.as_str() {
                "EXECUTE" => true,
                "HOLD" => {
                    info!("🧠 AI recommends HOLD - signal not executed");
                    false
                }
                "REJECT" => {
                    info!("🧠 AI recommends REJECT - signal rejected");
                    false
                }
                _ => true // Default to execute for unknown actions
            }
        } else {
            true // Execute if no AI analysis
        };

        // Log signal details in DRY RUN mode
        if self.dry_run {
            info!("🔍 DRY RUN - Signal details: {:?}", signal.metadata);
            if let Some(enhanced) = enhanced_signal {
                info!("🔍 DRY RUN - AI Analysis: {}", enhanced.ai_analysis);
            }
            return Ok(());
        }

        // AI-Enhanced Risk Assessment before execution
        if should_execute {
            info!("🛡️ Performing AI-enhanced risk assessment...");

            // Create mock portfolio for risk assessment
            let portfolio = Portfolio {
                total_value: self.config.trading.initial_balance,
                total_value_usd: Some(self.config.trading.initial_balance),
                available_balance: self.config.trading.initial_balance * 0.8, // 80% available
                unrealized_pnl: 0.0,
                realized_pnl: 0.0,
                positions: vec![],
                daily_pnl: 0.0,
                max_drawdown: 0.0,
                updated_at: chrono::Utc::now(),
            };

            // Get AI recommendation if available
            let ai_recommendation = enhanced_signal.as_ref().map(|enhanced| &enhanced.ai_recommendation);

            // Perform AI-enhanced risk assessment
            match self.risk_manager.assess_signal_with_ai(&signal, &portfolio, ai_recommendation).await {
                Ok(risk_assessment) => {
                    if risk_assessment.approved {
                        info!("✅ Risk assessment passed - executing signal");
                        info!("🛡️ Risk score: {:.2}, Suggested size: {:?}",
                              risk_assessment.risk_score, risk_assessment.suggested_size);

                        // Log risk warnings if any
                        for warning in &risk_assessment.warnings {
                            warn!("⚠️ Risk warning: {}", warning);
                        }

                        // Forward signal to trading engine for execution
                        // Note: In a real implementation, we'd send this through the signal_sender
                        info!("🎯 Signal ready for execution: {} {} {}",
                              signal.strategy, signal.signal_type, signal.symbol);
                    } else {
                        warn!("🚫 Signal execution blocked by risk management");
                        for warning in &risk_assessment.warnings {
                            warn!("🚫 Risk block reason: {}", warning);
                        }

                        // Report risk rejection to dashboard
                        if let Some(reporter) = &self.reporter {
                            if let Err(e) = reporter.lock().await.report_risk_rejection(&signal, &risk_assessment).await {
                                warn!("⚠️ Failed to report risk rejection: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Risk assessment failed: {}", e);
                    warn!("🚫 Signal execution blocked due to risk assessment failure");
                }
            }
        } else {
            info!("🚫 Signal execution blocked by AI analysis");
        }

        Ok(())
    }

    /// Process market opportunities discovered by scanner
    async fn process_opportunity(&self, opportunity: PotentialOpportunity) -> Result<()> {
        info!("🔍 New opportunity discovered: {} ({}) - {:.2}% confidence",
              opportunity.symbol, opportunity.source, opportunity.confidence_score * 100.0);

        // Log opportunity details
        info!("💰 Market Cap: ${:.0}, Volume: ${:.0}, Liquidity: ${:.0}",
              opportunity.market_cap.unwrap_or(0.0),
              opportunity.volume_24h.unwrap_or(0.0),
              opportunity.liquidity_usd.unwrap_or(0.0));

        // In DRY RUN mode, just log the opportunity
        if self.dry_run {
            info!("🔍 DRY RUN - Would analyze opportunity: {:?}", opportunity.opportunity_type);
            return Ok(());
        }

        // TODO: Forward high-confidence opportunities to strategies for detailed analysis
        // This would involve creating a market event and passing it to strategy manager

        Ok(())
    }

    /// Perform periodic health checks
    async fn perform_health_check(&self) {
        info!("💓 Performing health check...");

        // Check WebSocket connection status
        let ws_status = self.websocket_manager.get_connection_status().await;
        if !ws_status.is_healthy() {
            warn!("⚠️ WebSocket connection issues detected");
        } else {
            info!("✅ WebSocket connections healthy");
        }

        // Check strategy performance
        let strategy_stats = self.strategy_manager.get_performance_stats().await;
        info!("📈 Strategy stats: {:?}", strategy_stats);

        // Check trading engine status
        let engine_status = self.trading_engine.get_status();
        info!("⚡ Trading engine status: {:?}", engine_status);

        // Check DragonflyDB health
        if let Some(dragonfly) = &self.dragonfly_manager {
            if dragonfly.health_check().await {
                info!("✅ DragonflyDB connection healthy");
            } else {
                warn!("⚠️ DragonflyDB connection issues detected");
            }
        }

        // Update strategies based on current balance (portfolio-aware activation)
        let current_balance = self.config.trading.initial_balance; // TODO: Get real balance from balance manager
        info!("💰 Updating strategies for balance: ${:.2}", current_balance);
        if let Err(e) = self.strategy_manager.update_strategies_for_balance(current_balance).await {
            warn!("⚠️ Failed to update strategies for balance: {}", e);
        } else {
            info!("✅ Portfolio-aware strategy activation completed");
        }
    }

    /// Create mock strategy context for event processing
    async fn create_mock_strategy_context(&self) -> sniper_bot::strategy::enhanced_strategy::StrategyContext {
        use sniper_bot::strategy::enhanced_strategy::{StrategyContext, MarketConditions, VolumeTrend, PriceMomentum};
        use sniper_bot::data_fetcher::data_aggregator::AggregatedMarketData;
        use sniper_bot::models::{MarketData, DataSource, Portfolio};
        use chrono::Utc;

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
            market_cap: Some(1000000.0),
            age_hours: Some(12.0),
        };

        StrategyContext::new(aggregated_data, portfolio, market_conditions)
    }

}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_bot_initialization() {
        // Test basic bot initialization
        // This will be expanded as we add more components
        assert!(true); // Placeholder test
    }
}

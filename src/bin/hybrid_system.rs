use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{Duration, interval};
use tracing::{info, warn, error};

use sniper_bot::{
    config::AppConfig,
    db_connector::{DbClient, DbConfig},
    reflex_core::{OnChainStreamListener, SniperExecutor, NewTokenOpportunity},
    data_fetcher::soul_meteor_scanner::{SoulMeteorScanner, HotCandidate},
    pipeline::{PipelineController, ValidatedOpportunity},
    models::persistent_state::TradingDecision,
    utils::reporter::{Reporter, ReporterConfig},
};

/// Hybrid Trading System - First-of-its-kind dual-mode trading bot
/// 
/// This system combines:
/// - Reflex Core: Ultra-fast new token detection and execution (0-60s)
/// - Intelligence Brain: Quality analysis for established tokens
/// 
/// Architecture:
/// - Shared DragonflyDB for coordination
/// - Dual queues: new_token_queue vs raw_opportunities
/// - Parallel processing with conflict resolution
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

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🌟 Starting HYBRID TRADING SYSTEM - World's First Dual-Mode Bot!");
    info!("⚡ Reflex Core: Ultra-fast new tokens (0-60s)");
    info!("🧠 Intelligence Brain: Quality analysis (established tokens)");

    // Load configuration
    let config = AppConfig::from_env();
    info!("✅ Configuration loaded");

    // Initialize shared database
    let db_config = DbConfig::default();
    let db_client = Arc::new(DbClient::new(db_config).await?);
    info!("✅ Shared DragonflyDB initialized");

    // Initialize reporter for dashboard
    let reporter = Arc::new(Mutex::new(initialize_reporter().await?));
    info!("✅ Reporter initialized");

    // Create communication channels
    let (new_token_tx, new_token_rx) = mpsc::channel::<NewTokenOpportunity>(1000);
    let (raw_opportunity_tx, raw_opportunity_rx) = mpsc::channel::<HotCandidate>(500);
    let (validated_opportunity_tx, validated_opportunity_rx) = mpsc::channel::<ValidatedOpportunity>(200);
    let (trading_decision_tx, trading_decision_rx) = mpsc::channel::<TradingDecision>(100);

    info!("✅ Communication channels established");

    // Initialize Hybrid System
    let hybrid_system = HybridTradingSystem::new(
        config,
        db_client.clone(),
        reporter.clone(),
        new_token_tx,
        raw_opportunity_tx,
        validated_opportunity_tx,
        trading_decision_tx,
    ).await?;

    info!("🎯 Hybrid Trading System initialized successfully!");

    // Start all components
    hybrid_system.start_all_components(
        new_token_rx,
        raw_opportunity_rx,
        validated_opportunity_rx,
        trading_decision_rx,
    ).await?;

    // Keep the system running
    info!("🚀 Hybrid system is now running...");
    info!("Press Ctrl+C to stop");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("🛑 Shutdown signal received, stopping hybrid system...");

    Ok(())
}

/// Main Hybrid Trading System struct
pub struct HybridTradingSystem {
    config: AppConfig,
    db_client: Arc<DbClient>,
    reporter: Arc<Mutex<Reporter>>,
    
    // Reflex Core components
    onchain_listener: OnChainStreamListener,
    sniper_executor: SniperExecutor,
    
    // Intelligence Brain components
    soul_meteor_scanner: SoulMeteorScanner,
    pipeline_controller: PipelineController,
    
    // Communication channels (senders)
    new_token_sender: mpsc::Sender<NewTokenOpportunity>,
    raw_opportunity_sender: mpsc::Sender<HotCandidate>,
    validated_opportunity_sender: mpsc::Sender<ValidatedOpportunity>,
    trading_decision_sender: mpsc::Sender<TradingDecision>,
}

impl HybridTradingSystem {
    pub async fn new(
        config: AppConfig,
        db_client: Arc<DbClient>,
        reporter: Arc<Mutex<Reporter>>,
        new_token_sender: mpsc::Sender<NewTokenOpportunity>,
        raw_opportunity_sender: mpsc::Sender<HotCandidate>,
        validated_opportunity_sender: mpsc::Sender<ValidatedOpportunity>,
        trading_decision_sender: mpsc::Sender<TradingDecision>,
    ) -> Result<Self> {
        info!("🔧 Initializing Hybrid Trading System components...");

        // Initialize Reflex Core
        info!("⚡ Initializing Reflex Core...");
        let onchain_listener = OnChainStreamListener::new(config.clone(), db_client.clone())?;
        let sniper_executor = SniperExecutor::new(config.clone(), db_client.clone());
        info!("✅ Reflex Core initialized");

        // Initialize Intelligence Brain
        info!("🧠 Initializing Intelligence Brain...");
        let soul_meteor_scanner = SoulMeteorScanner::new();
        // Create a static config for PipelineController
        let static_config = Box::leak(Box::new(config.clone()));
        let pipeline_controller = PipelineController::new((*db_client).clone(), static_config);
        info!("✅ Intelligence Brain initialized");

        Ok(Self {
            config,
            db_client,
            reporter,
            onchain_listener,
            sniper_executor,
            soul_meteor_scanner,
            pipeline_controller,
            new_token_sender,
            raw_opportunity_sender,
            validated_opportunity_sender,
            trading_decision_sender,
        })
    }

    pub async fn start_all_components(
        &self,
        new_token_rx: mpsc::Receiver<NewTokenOpportunity>,
        raw_opportunity_rx: mpsc::Receiver<HotCandidate>,
        validated_opportunity_rx: mpsc::Receiver<ValidatedOpportunity>,
        trading_decision_rx: mpsc::Receiver<TradingDecision>,
    ) -> Result<()> {
        info!("🚀 Starting all hybrid system components...");

        // Start Reflex Core components
        self.start_reflex_core().await?;
        
        // Start Intelligence Brain components
        self.start_intelligence_brain().await?;
        
        // Start processing pipelines
        self.start_processing_pipelines(
            new_token_rx,
            raw_opportunity_rx,
            validated_opportunity_rx,
            trading_decision_rx,
        ).await?;

        // Start monitoring and health checks
        self.start_monitoring().await?;

        info!("✅ All hybrid system components started successfully!");
        Ok(())
    }

    async fn start_reflex_core(&self) -> Result<()> {
        info!("⚡ Starting Reflex Core components...");

        // Start OnChain Stream Listener
        let new_token_sender = self.new_token_sender.clone();
        let db_client = self.db_client.clone();

        tokio::spawn(async move {
            info!("📡 OnChain Stream Listener started");
            
            // Simulate new token detection (in real implementation, this would listen to Solana)
            let mut detection_interval = interval(Duration::from_secs(30));
            
            loop {
                detection_interval.tick().await;
                
                // Simulate detecting a new token
                if let Ok(new_token) = simulate_new_token_detection().await {
                    info!("🎯 NEW TOKEN DETECTED: {} (age: {}s, risk: {:.2})", 
                          new_token.token_address, 
                          new_token.age_seconds,
                          new_token.risk_score);
                    
                    // Save to database
                    let key = new_token.redis_key();
                    if let Err(e) = db_client.set(&key, &new_token, Some(300)).await {
                        error!("❌ Failed to save new token to DB: {}", e);
                        continue;
                    }
                    
                    // Add to queue
                    if let Err(e) = db_client.list_push("new_token_queue", &key).await {
                        error!("❌ Failed to add to new_token_queue: {}", e);
                        continue;
                    }
                    
                    // Send to processing channel
                    if let Err(e) = new_token_sender.send(new_token).await {
                        error!("❌ Failed to send new token to channel: {}", e);
                    }
                }
            }
        });

        info!("✅ Reflex Core started");
        Ok(())
    }

    async fn start_intelligence_brain(&self) -> Result<()> {
        info!("🧠 Starting Intelligence Brain components...");

        // Start Soul Meteor Scanner
        let raw_opportunity_sender = self.raw_opportunity_sender.clone();
        let db_client = self.db_client.clone();

        tokio::spawn(async move {
            info!("🔍 Soul Meteor Scanner started");
            
            let mut scan_interval = interval(Duration::from_secs(60));
            
            loop {
                scan_interval.tick().await;
                
                // Simulate scanning for opportunities
                match simulate_soul_meteor_scan().await {
                    Ok(candidates) => {
                        info!("📊 Soul Meteor found {} candidates", candidates.len());
                        
                        for candidate in candidates {
                            // Save to database
                            let key = format!("raw_opportunity:{}", candidate.address);
                            if let Err(e) = db_client.set(&key, &candidate, Some(600)).await {
                                error!("❌ Failed to save candidate to DB: {}", e);
                                continue;
                            }
                            
                            // Add to queue
                            if let Err(e) = db_client.list_push("raw_opportunities", &key).await {
                                error!("❌ Failed to add to raw_opportunities: {}", e);
                                continue;
                            }
                            
                            // Send to processing channel
                            if let Err(e) = raw_opportunity_sender.send(candidate).await {
                                error!("❌ Failed to send candidate to channel: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Soul Meteor scan failed: {}", e);
                    }
                }
            }
        });

        info!("✅ Intelligence Brain started");
        Ok(())
    }

    async fn start_processing_pipelines(
        &self,
        mut new_token_rx: mpsc::Receiver<NewTokenOpportunity>,
        mut raw_opportunity_rx: mpsc::Receiver<HotCandidate>,
        mut validated_opportunity_rx: mpsc::Receiver<ValidatedOpportunity>,
        mut trading_decision_rx: mpsc::Receiver<TradingDecision>,
    ) -> Result<()> {
        info!("🔄 Starting processing pipelines...");

        // Pipeline 1: Reflex Core - New Token Processing
        let reporter = self.reporter.clone();
        
        tokio::spawn(async move {
            info!("⚡ Reflex Core processing pipeline started");
            
            while let Some(new_token) = new_token_rx.recv().await {
                info!("🎯 Processing new token: {}", new_token.token_address);
                
                // Ultra-fast validation and execution
                if new_token.is_fresh() && new_token.is_safe() {
                    info!("✅ Token passed Reflex Core validation - executing trade");
                    
                    // In DRY RUN mode, just log the decision
                    let position_size = calculate_position_size(&new_token);
                    info!("🔥 REFLEX CORE TRADE: {} SOL for {}", position_size, new_token.token_address);
                    
                    // Report to dashboard
                    let reporter_guard = reporter.lock().await;
                    let signal = create_mock_signal(&new_token);
                    if let Err(e) = reporter_guard.report_signal(&signal).await {
                        warn!("⚠️ Failed to report Reflex Core trade: {}", e);
                    }
                } else {
                    info!("⛔ Token rejected by Reflex Core validation");
                }
            }
        });

        // Pipeline 2: Intelligence Brain - Quality Analysis
        let validated_sender = self.validated_opportunity_sender.clone();
        
        tokio::spawn(async move {
            info!("🧠 Intelligence Brain processing pipeline started");
            
            while let Some(candidate) = raw_opportunity_rx.recv().await {
                info!("🔍 Analyzing candidate: {}", candidate.address);

                // For now, simulate validation (in real implementation, use PipelineController)
                match simulate_validation(candidate).await {
                    Ok(validated) => {
                        info!("✅ Candidate validated by Intelligence Brain");
                        if let Err(e) = validated_sender.send(validated).await {
                            error!("❌ Failed to send validated opportunity: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Candidate rejected by Intelligence Brain: {}", e);
                    }
                }
            }
        });

        // Pipeline 3: Decision Engine - Process Validated Opportunities
        let trading_decision_sender = self.trading_decision_sender.clone();

        tokio::spawn(async move {
            info!("🤖 Decision Engine: AKTYWNY. Oczekiwanie na zwalidowane okazje...");

            while let Some(opportunity) = validated_opportunity_rx.recv().await {
                info!("💡 Otrzymano zwalidowaną okazję od Intelligence Brain: {}", opportunity.candidate.name);
                info!("   📊 Combined Score: {:.1}", opportunity.combined_score);
                info!("   🎯 Strategy: {:?}", opportunity.recommended_strategy);
                info!("   ⚠️ Risk Level: {:?}", opportunity.risk_level);

                // For now, automatically approve high-quality opportunities
                if opportunity.combined_score >= 6.0 && opportunity.is_actionable() {
                    info!("✅ Opportunity approved by Decision Engine");

                    // Create trading decision
                    let decision = TradingDecision {
                        action: "BUY".to_string(),
                        amount: 0.1, // Conservative amount for testing
                        confidence: opportunity.sentiment.confidence,
                    };

                    if let Err(e) = trading_decision_sender.send(decision).await {
                        error!("❌ Failed to send trading decision: {}", e);
                    } else {
                        info!("📤 Trading decision sent for execution");
                    }
                } else {
                    info!("⛔ Opportunity rejected by Decision Engine (score: {:.1})", opportunity.combined_score);
                }
            }
            info!("🤖 Kanał dla zwalidowanych okazji został zamknięty. Zamykanie Decision Engine.");
        });

        // Pipeline 4: Trading Executor - Execute Final Decisions
        tokio::spawn(async move {
            info!("⚡ Trading Executor: AKTYWNY. Oczekiwanie na decyzje handlowe...");

            while let Some(decision) = trading_decision_rx.recv().await {
                info!("🎯 Otrzymano decyzję handlową: {}", decision.action);
                info!("   💰 Amount: {} SOL", decision.amount);
                info!("   🎯 Confidence: {:.2}", decision.confidence);

                // In DRY RUN mode, just simulate execution
                info!("🔥 SIMULATED TRADE EXECUTION:");
                info!("   📊 Action: {}", decision.action);
                info!("   💰 Amount: {} SOL", decision.amount);
                info!("   ✅ Status: EXECUTED (DRY RUN)");

                // TODO: In live mode, execute real trade through Jupiter/Jito
                // let execution_result = execute_trade(&decision).await;
            }
            info!("⚡ Kanał dla decyzji handlowych został zamknięty. Zamykanie Trading Executor.");
        });

        info!("✅ Processing pipelines started");
        Ok(())
    }

    async fn start_monitoring(&self) -> Result<()> {
        info!("📊 Starting monitoring and health checks...");

        let db_client = self.db_client.clone();
        let reporter = self.reporter.clone();

        tokio::spawn(async move {
            let mut health_interval = interval(Duration::from_secs(30));
            
            loop {
                health_interval.tick().await;
                
                // Check system health
                let new_token_queue_size = db_client.list_range_raw("new_token_queue", 0, -1).await.unwrap_or_default().len();
                let raw_opportunities_size = db_client.list_range_raw("raw_opportunities", 0, -1).await.unwrap_or_default().len();
                
                info!("💓 System Health Check:");
                info!("   📋 New Token Queue: {} items", new_token_queue_size);
                info!("   📋 Raw Opportunities: {} items", raw_opportunities_size);
                
                // Report system stats
                let _reporter_guard = reporter.lock().await;
                // Create performance metric event
                let stats = format!("new_tokens:{},raw_opportunities:{}",
                                  new_token_queue_size, raw_opportunities_size);
                info!("📊 System stats: {}", stats);
            }
        });

        info!("✅ Monitoring started");
        Ok(())
    }
}

// Helper functions for simulation
async fn simulate_new_token_detection() -> Result<NewTokenOpportunity> {
    use chrono::Utc;
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let token_id = rng.gen::<u32>();
    
    Ok(NewTokenOpportunity {
        token_address: format!("NEW_TOKEN_{}", token_id),
        pool_address: format!("POOL_{}", token_id),
        token_symbol: Some(format!("NT{}", token_id % 1000)),
        initial_liquidity_sol: rng.gen_range(1.0..10.0),
        initial_liquidity_usd: rng.gen_range(200.0..2000.0),
        creation_tx_signature: format!("sig_{}", token_id),
        creation_slot: rng.gen_range(100000..200000),
        detected_at: Utc::now(),
        age_seconds: rng.gen_range(5..45),
        dex: "Raydium".to_string(),
        risk_score: rng.gen_range(0.3..0.95),
        mint_authority_burned: rng.gen_bool(0.7),
        freeze_authority_burned: rng.gen_bool(0.8),
        initial_market_cap_usd: Some(rng.gen_range(10000.0..100000.0)),
    })
}

async fn simulate_soul_meteor_scan() -> Result<Vec<HotCandidate>> {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(1..4);
    let mut candidates = Vec::new();
    
    for i in 0..count {
        candidates.push(HotCandidate {
            name: format!("SOUL{}", i),
            address: format!("SOUL_TOKEN_{}", rng.gen::<u32>()),
            liquidity_usd: rng.gen_range(20000.0..200000.0),
            volume_24h: rng.gen_range(10000.0..100000.0),
            fees_24h: rng.gen_range(100.0..1000.0),
            fee_tvl_ratio_24h: rng.gen_range(1.0..5.0),
            apr: rng.gen_range(10.0..100.0),
            apy: rng.gen_range(15.0..150.0),
            opportunity_score: rng.gen_range(0.5..0.9),
            mint_x: format!("MINT_X_{}", rng.gen::<u32>()),
            mint_y: format!("MINT_Y_{}", rng.gen::<u32>()),
            current_price: rng.gen_range(0.1..10.0),
            is_blacklisted: false,
            hide: false,
        });
    }
    
    Ok(candidates)
}

async fn simulate_validation(candidate: HotCandidate) -> Result<ValidatedOpportunity> {
    use sniper_bot::pipeline::opportunity::{SentimentReport, OpportunityStatus, StrategyType, RiskLevel};
    use chrono::Utc;

    // Simulate validation logic
    if candidate.liquidity_usd >= 20000.0 && candidate.opportunity_score >= 0.6 {
        let sentiment = SentimentReport {
            aggregated_score: 0.5,
            confidence: 0.8,
            patterns: vec!["POSITIVE".to_string()],
            sources_count: 3,
            textual_data: None,
            analyzed_at: Utc::now(),
        };

        Ok(ValidatedOpportunity {
            id: format!("{}_{}", candidate.address, Utc::now().timestamp()),
            candidate,
            sentiment,
            status: OpportunityStatus::Validated,
            combined_score: 7.5,
            recommended_strategy: StrategyType::TokenSniper,
            risk_level: RiskLevel::Low,
            discovered_at: Utc::now(),
            updated_at: Utc::now(),
        })
    } else {
        Err(anyhow::anyhow!("Candidate failed validation"))
    }
}

fn calculate_position_size(opportunity: &NewTokenOpportunity) -> f64 {
    let base_size = 0.05; // 0.05 SOL base
    let risk_factor = opportunity.risk_score;
    let liquidity_factor = (opportunity.initial_liquidity_sol / 10.0).min(1.0);
    let age_factor = 1.0 - (opportunity.age_seconds as f64 / 60.0);
    
    base_size * risk_factor * liquidity_factor * age_factor.max(0.1)
}

fn create_mock_signal(opportunity: &NewTokenOpportunity) -> sniper_bot::models::StrategySignal {
    use sniper_bot::models::{StrategySignal, SignalType};
    use chrono::Utc;


    StrategySignal {
        strategy: "reflex_core".to_string(),
        symbol: opportunity.token_address.clone(),
        signal_type: SignalType::Buy,
        strength: opportunity.risk_score,
        price: opportunity.initial_liquidity_sol,
        size: calculate_position_size(opportunity),
        metadata: serde_json::json!({}),
        timestamp: Utc::now(),
    }
}

async fn initialize_reporter() -> Result<Reporter> {
    let config = ReporterConfig {
        enabled: true,
        dashboard_url: std::env::var("DASHBOARD_URL")
            .unwrap_or_else(|_| "http://localhost:8084/api/report_event".to_string()),
        api_key: std::env::var("DASHBOARD_API_KEY").ok(),
        ..ReporterConfig::default()
    };
    
    let mut reporter = Reporter::new(config);
    reporter.start().await?;
    Ok(reporter)
}

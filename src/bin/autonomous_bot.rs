/*!
🧠 SniperBot 2.0 - Autonomous Trading Organism

The Persistent Brain - Hub-and-Spoke Architecture with DragonflyDB

This is the autonomous heart that beats continuously, orchestrating:
- DragonflyDB as the central nervous system
- Pipeline Controller as the processing brain
- Continuous operation without manual intervention
- Graceful shutdown and restart capabilities
*/

use sniper_bot::config::AppConfig;
use sniper_bot::db_connector::{DbClient, DbConfig};
use sniper_bot::pipeline::controller::PipelineController;
use sniper_bot::models::TradingResult;
use clap::{Arg, Command};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;
use tokio::signal;
use tokio::time::interval;
use tracing::{info, error, warn, debug};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> TradingResult<()> {
    // === PHASE 1: SYSTEM INITIALIZATION ===
    
    // Load environment variables
    dotenv().ok();
    
    // Initialize logging with environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sniper_bot=debug"));
    
    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(true)
        .init();
    
    info!("🧠 === SniperBot 2.0: The Persistent Brain === ");
    info!("🚀 Autonomous Trading Organism - Starting...");
    
    // Parse command line arguments
    let matches = Command::new("SniperBot 2.0 - The Persistent Brain")
        .version("2.0.0-phase6")
        .author("SynergiaOS")
        .about("Autonomous AI-powered Solana trading organism with DragonflyDB brain")
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name("MODE")
                .help("Trading mode: dry-run, pilot, live")
                .default_value("dry-run")
        )
        .arg(
            Arg::new("interval")
                .long("interval")
                .value_name("SECONDS")
                .help("Processing interval in seconds")
                .default_value("300")
        )
        .arg(
            Arg::new("health-check")
                .long("health-check")
                .help("Perform health check and exit")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Initialize global configuration
    let config = AppConfig::init();
    
    // Validate configuration
    if let Err(e) = config.validate() {
        error!("❌ Configuration validation failed: {}", e);
        return Err(e.into());
    }

    // Override mode if specified
    let mode = matches.get_one::<String>("mode").unwrap();
    info!("🎯 Trading mode: {}", mode);

    // Safety check for live trading
    if mode == "live" {
        warn!("⚠️ LIVE TRADING MODE ENABLED!");
        warn!("⚠️ Real money will be used for trading!");
        warn!("⚠️ Make sure you understand the risks!");
        
        if env::var("CONFIRM_LIVE_TRADING").unwrap_or_default() != "yes" {
            error!("❌ Live trading requires CONFIRM_LIVE_TRADING=yes environment variable");
            return Err("Live trading not confirmed".to_string().into());
        }
    }

    // === PHASE 2: DRAGONFLY DB CONNECTION ===
    
    info!("🐉 Connecting to DragonflyDB (The Brain)...");
    
    let db_config = DbConfig::from_env()?;
    let db_client = DbClient::new(db_config).await?;
    
    info!("✅ Connected to DragonflyDB successfully");
    
    // Perform health check
    if !db_client.health_check().await? {
        error!("❌ DragonflyDB health check failed");
        return Err("Database health check failed".to_string().into());
    }
    
    info!("✅ DragonflyDB health check passed");

    // If health check mode, exit here
    if matches.get_flag("health-check") {
        info!("🏥 Health check completed successfully");
        return Ok(());
    }

    // === PHASE 3: PIPELINE CONTROLLER INITIALIZATION ===

    info!("🧠 Initializing Pipeline Controller...");

    let mut pipeline_controller = PipelineController::new(db_client.clone(), config);

    // Check if pipeline is ready
    if !pipeline_controller.is_ready().await {
        error!("❌ Pipeline Controller is not ready");
        return Err("Pipeline not ready".to_string().into());
    }

    info!("✅ Pipeline Controller initialized and ready");

    // === PHASE 4: AUTONOMOUS OPERATION LOOP ===
    
    let processing_interval = Duration::from_secs(
        matches.get_one::<String>("interval")
            .unwrap()
            .parse()
            .unwrap_or(config.main_loop.processing_interval_seconds)
    );
    
    info!("⚙️ Processing interval: {} seconds", processing_interval.as_secs());
    info!("🔄 Maximum opportunities per cycle: {}", config.main_loop.max_opportunities_per_cycle);
    info!("⏱️ Cycle timeout: {} seconds", config.main_loop.cycle_timeout_seconds);
    
    let mut interval_timer = interval(processing_interval);
    let mut cycle_count = 0u64;
    
    info!("✅ AUTONOMOUS ORGANISM READY - Entering continuous operation mode");
    info!("🧠 The Persistent Brain is now active and processing...");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    loop {
        tokio::select! {
            // Main processing cycle
            _ = interval_timer.tick() => {
                cycle_count += 1;
                
                info!("🔄 === AUTONOMOUS CYCLE #{} STARTING ===", cycle_count);
                
                let cycle_start = std::time::Instant::now();
                
                match pipeline_controller.process_opportunities_from_db().await {
                    Ok(processed_count) => {
                        let duration = cycle_start.elapsed();
                        
                        if processed_count > 0 {
                            info!("🎉 CYCLE #{} COMPLETED: {} opportunities processed in {:.2}s", 
                                  cycle_count, processed_count, duration.as_secs_f64());
                        } else {
                            debug!("🏁 Cycle #{} completed: No new opportunities (Duration: {:.2}s)", 
                                   cycle_count, duration.as_secs_f64());
                        }
                    }
                    Err(e) => {
                        error!("💀 Cycle #{} FAILED: {} (Continuing to next cycle)", cycle_count, e);
                    }
                }
                
                info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
            
            // Graceful shutdown on SIGINT/SIGTERM
            _ = signal::ctrl_c() => {
                info!("🛑 Shutdown signal received");
                break;
            }
        }
    }

    // === PHASE 5: GRACEFUL SHUTDOWN ===
    
    info!("🛑 Initiating graceful shutdown...");
    
    // Perform final health check
    let health_status = pipeline_controller.health_check().await?;
    info!("🏥 Final health check: {:?}", health_status);
    
    // Get final statistics
    let stats = pipeline_controller.stats();
    info!("📊 Final Statistics:");
    info!("  • Total cycles: {}", stats.cycles_completed);
    info!("  • Total candidates: {}", stats.total_candidates_found);
    info!("  • Total validated: {}", stats.total_candidates_validated);
    info!("  • Total decisions: {}", stats.total_decisions_made);
    info!("  • Average cycle time: {}ms", stats.average_cycle_duration_ms);
    
    info!("✅ SniperBot 2.0 - The Persistent Brain shutdown complete");
    info!("🧠 Autonomous organism has been safely deactivated");
    
    Ok(())
}

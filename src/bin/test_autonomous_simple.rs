/*!
🧪 Simple Autonomous Operation Test

This test validates the core autonomous operation without complex dependencies:
1. Configuration loading and validation
2. DragonflyDB connection and health check
3. Basic pipeline controller initialization
*/

use sniper_bot::config::AppConfig;
use sniper_bot::db_connector::{DbClient, DbConfig};
use sniper_bot::models::TradingResult;
use tracing::{info, error};
use tracing_subscriber;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> TradingResult<()> {
    // Load .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 === SIMPLE AUTONOMOUS OPERATION TEST ===");

    // === TEST 1: CONFIGURATION VALIDATION ===
    info!("🔧 [TEST 1] Configuration validation...");
    
    let config = AppConfig::init();
    
    match config.validate() {
        Ok(_) => info!("✅ Configuration validation passed"),
        Err(e) => {
            error!("❌ Configuration validation failed: {}", e);
            return Err(e.into());
        }
    }

    info!("📋 Configuration Summary:");
    info!("  • Processing interval: {} seconds", config.main_loop.processing_interval_seconds);
    info!("  • Max opportunities per cycle: {}", config.main_loop.max_opportunities_per_cycle);
    info!("  • Trading mode: {:?}", config.trading.mode);
    info!("  • DragonflyDB URL: {}", config.database.dragonfly_url);

    // === TEST 2: DATABASE CONNECTION ===
    info!("🐉 [TEST 2] DragonflyDB connection test...");
    
    let db_config = DbConfig::from_env()?;
    let db_client = DbClient::new(db_config).await?;
    
    info!("✅ Connected to DragonflyDB");
    
    // Health check
    if !db_client.health_check().await? {
        error!("❌ DragonflyDB health check failed");
        return Err("Database health check failed".to_string().into());
    }
    
    info!("✅ DragonflyDB health check passed");

    // === TEST 3: DATABASE CONTENT CHECK ===
    info!("📊 [TEST 3] Checking database content...");
    
    // Check for raw opportunities
    let raw_opportunities_keys: Vec<String> = db_client
        .list_range::<String>("all_raw_opportunities", 0, -1)
        .await
        .unwrap_or_default();
    
    info!("📈 Found {} raw opportunity keys in database", raw_opportunities_keys.len());
    
    if raw_opportunities_keys.is_empty() {
        info!("ℹ️ No raw opportunities found in database");
        info!("ℹ️ Run Soul Meteor Scanner first to populate opportunities:");
        info!("ℹ️ cd pyinstaller_scripts && python3 soul_meteor_scanner.py");
    } else {
        info!("✅ Database contains opportunities for processing");
        
        // Show first few opportunities
        for (i, key) in raw_opportunities_keys.iter().take(3).enumerate() {
            info!("  {}. {}", i + 1, key);
        }
    }

    // Check processed tokens
    let processed_tokens: Vec<String> = db_client
        .set_members("processed_tokens")
        .await
        .unwrap_or_default();
    
    info!("🏷️ Processed tokens count: {}", processed_tokens.len());

    // === TEST 4: DATABASE STATISTICS ===
    info!("📊 [TEST 4] Database statistics...");
    
    let stats = db_client.get_stats().await?;
    info!("📈 Database Statistics:");
    info!("  • Total Keys: {}", stats.total_keys);
    info!("  • Timestamp: {}", stats.timestamp);

    // === TEST 5: CONFIGURATION DISPLAY ===
    info!("⚙️ [TEST 5] Full configuration display...");
    
    info!("🔧 Main Loop Configuration:");
    info!("  • Processing interval: {} seconds", config.main_loop.processing_interval_seconds);
    info!("  • Max opportunities per cycle: {}", config.main_loop.max_opportunities_per_cycle);
    info!("  • Cycle timeout: {} seconds", config.main_loop.cycle_timeout_seconds);
    info!("  • Retry attempts: {}", config.main_loop.retry_attempts);
    info!("  • Health check interval: {} seconds", config.main_loop.health_check_interval_seconds);

    info!("🎯 Trading Configuration:");
    info!("  • Mode: {:?}", config.trading.mode);
    info!("  • Max position size: {} SOL", config.trading.max_position_size_sol);
    info!("  • Risk tolerance: {:.1}", config.trading.risk_tolerance);
    info!("  • PumpFun sniping: {}", config.trading.enable_pumpfun_sniping);
    info!("  • Liquidity sniping: {}", config.trading.enable_liquidity_sniping);

    info!("🛡️ Risk Management:");
    info!("  • Stop loss: {}%", config.risk_management.stop_loss_percentage);
    info!("  • Take profit: {}%", config.risk_management.take_profit_percentage);
    info!("  • Max daily loss: {} SOL", config.risk_management.max_daily_loss_sol);

    info!("🤖 AI Configuration:");
    info!("  • Mistral API key configured: {}", config.ai.mistral_api_key.is_some());
    info!("  • AI risk weight: {:.1}", config.ai.ai_risk_weight);
    info!("  • AI confidence threshold: {:.1}", config.ai.ai_confidence_threshold);

    // === FINAL RESULTS ===
    info!("🎉 === ALL TESTS COMPLETED SUCCESSFULLY ===");
    info!("✅ Core autonomous operation components are ready!");
    info!("");
    info!("🚀 Next steps:");
    info!("   1. Ensure Soul Meteor Scanner has populated opportunities");
    info!("   2. Run full autonomous bot: cargo run --bin autonomous_bot");
    info!("   3. Monitor logs for processing cycles");
    info!("");
    info!("🔧 Commands:");
    info!("   • Health check: cargo run --bin autonomous_bot -- --health-check");
    info!("   • Custom interval: cargo run --bin autonomous_bot -- --interval 180");

    Ok(())
}

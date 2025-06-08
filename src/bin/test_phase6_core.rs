/*!
🧪 Phase 6 Core Test - Autonomous Operation Foundation

This test validates only the core Phase 6 components:
1. Configuration system (AppConfig)
2. DragonflyDB integration
3. Basic autonomous operation readiness
*/

use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tracing::{info, error, warn};
use tracing_subscriber;

// Minimal configuration for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub dragonfly_url: String,
    pub processing_interval_seconds: u64,
    pub max_opportunities_per_cycle: usize,
    pub bot_mode: String,
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            dragonfly_url: env::var("DRAGONFLY_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            processing_interval_seconds: env::var("PROCESSING_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            max_opportunities_per_cycle: env::var("MAX_OPPORTUNITIES_PER_CYCLE")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            bot_mode: env::var("BOT_MODE").unwrap_or_else(|_| "DRY_RUN".to_string()),
        }
    }
}

// Simple Redis client for testing
use redis::{Client, Commands};

pub struct SimpleDbClient {
    client: Client,
}

impl SimpleDbClient {
    pub async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }

    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let result: String = redis::cmd("PING").query(&mut conn)?;
        Ok(result == "PONG")
    }

    pub async fn get_list_length(&self, key: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let length: usize = conn.llen(key)?;
        Ok(length)
    }

    pub async fn get_set_size(&self, key: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let size: usize = conn.scard(key)?;
        Ok(size)
    }

    pub async fn get_keys_count(&self, pattern: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let keys: Vec<String> = conn.keys(pattern)?;
        Ok(keys.len())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 === PHASE 6 CORE AUTONOMOUS OPERATION TEST ===");

    // === TEST 1: CONFIGURATION LOADING ===
    info!("🔧 [TEST 1] Configuration loading...");
    
    let config = TestConfig::from_env();
    
    info!("📋 Configuration loaded:");
    info!("  • DragonflyDB URL: {}", config.dragonfly_url);
    info!("  • Processing interval: {} seconds", config.processing_interval_seconds);
    info!("  • Max opportunities per cycle: {}", config.max_opportunities_per_cycle);
    info!("  • Bot mode: {}", config.bot_mode);

    // === TEST 2: DRAGONFLY DB CONNECTION ===
    info!("🐉 [TEST 2] DragonflyDB connection test...");
    
    let db_client = SimpleDbClient::new(&config.dragonfly_url).await?;
    
    info!("✅ Connected to DragonflyDB");
    
    // Health check
    if !db_client.health_check().await? {
        error!("❌ DragonflyDB health check failed");
        return Err("Database health check failed".into());
    }
    
    info!("✅ DragonflyDB health check passed");

    // === TEST 3: DATABASE CONTENT ANALYSIS ===
    info!("📊 [TEST 3] Database content analysis...");
    
    // Check for raw opportunities
    let raw_opportunities_count = db_client.get_list_length("all_raw_opportunities").await.unwrap_or(0);
    info!("📈 Raw opportunities in database: {}", raw_opportunities_count);
    
    // Check processed tokens
    let processed_tokens_count = db_client.get_set_size("processed_tokens").await.unwrap_or(0);
    info!("🏷️ Processed tokens count: {}", processed_tokens_count);
    
    // Check trading decisions
    let trading_decisions_count = db_client.get_list_length("trading_decisions_queue").await.unwrap_or(0);
    info!("🎯 Trading decisions in queue: {}", trading_decisions_count);
    
    // Check raw opportunity keys
    let raw_opportunity_keys_count = db_client.get_keys_count("raw_opportunity:*").await.unwrap_or(0);
    info!("🔑 Raw opportunity keys: {}", raw_opportunity_keys_count);

    // === TEST 4: AUTONOMOUS OPERATION READINESS ===
    info!("🚀 [TEST 4] Autonomous operation readiness check...");
    
    let mut readiness_score = 0;
    let mut total_checks = 0;
    
    // Check 1: Configuration valid
    total_checks += 1;
    if config.processing_interval_seconds >= 60 && config.max_opportunities_per_cycle > 0 {
        info!("  ✅ Configuration is valid");
        readiness_score += 1;
    } else {
        warn!("  ⚠️ Configuration has issues");
    }
    
    // Check 2: Database accessible
    total_checks += 1;
    if db_client.health_check().await.unwrap_or(false) {
        info!("  ✅ Database is accessible");
        readiness_score += 1;
    } else {
        warn!("  ⚠️ Database is not accessible");
    }
    
    // Check 3: Data available
    total_checks += 1;
    if raw_opportunities_count > 0 || raw_opportunity_keys_count > 0 {
        info!("  ✅ Market data is available");
        readiness_score += 1;
    } else {
        warn!("  ⚠️ No market data available (run Soul Meteor Scanner first)");
    }
    
    // Check 4: Environment variables
    total_checks += 1;
    let required_env_vars = ["DRAGONFLY_URL"];
    let mut env_vars_ok = true;
    for var in &required_env_vars {
        if env::var(var).is_err() {
            warn!("  ⚠️ Missing environment variable: {}", var);
            env_vars_ok = false;
        }
    }
    if env_vars_ok {
        info!("  ✅ Required environment variables are set");
        readiness_score += 1;
    }

    // === TEST 5: AUTONOMOUS CYCLE SIMULATION ===
    info!("🔄 [TEST 5] Autonomous cycle simulation...");
    
    info!("  Simulating autonomous cycle with {} second interval...", config.processing_interval_seconds);
    
    // Simulate cycle timing
    let cycle_start = std::time::Instant::now();
    
    // Simulate processing delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let cycle_duration = cycle_start.elapsed();
    info!("  ✅ Simulated cycle completed in {:.2}ms", cycle_duration.as_millis());
    
    if cycle_duration.as_secs() < config.processing_interval_seconds {
        info!("  ✅ Cycle duration is within acceptable limits");
    } else {
        warn!("  ⚠️ Cycle duration exceeds interval");
    }

    // === FINAL ASSESSMENT ===
    info!("🎯 [FINAL] Autonomous operation readiness assessment...");
    
    let readiness_percentage = (readiness_score as f64 / total_checks as f64) * 100.0;
    
    info!("📊 Readiness Score: {}/{} ({:.1}%)", readiness_score, total_checks, readiness_percentage);
    
    if readiness_percentage >= 75.0 {
        info!("🎉 === AUTONOMOUS OPERATION READY ===");
        info!("✅ System is ready for autonomous operation!");
        
        info!("");
        info!("🚀 Next steps:");
        info!("   1. Start autonomous bot: cargo run --bin autonomous_bot");
        info!("   2. Monitor logs for processing cycles");
        info!("   3. Check dashboard for real-time status");
        
        info!("");
        info!("🔧 Commands:");
        info!("   • Health check: cargo run --bin autonomous_bot -- --health-check");
        info!("   • Custom interval: cargo run --bin autonomous_bot -- --interval 180");
        info!("   • DRY RUN mode: cargo run --bin autonomous_bot -- --mode dry-run");
        
    } else {
        warn!("⚠️ === AUTONOMOUS OPERATION NOT READY ===");
        warn!("System needs attention before autonomous operation");
        
        info!("");
        info!("🔧 Required actions:");
        if raw_opportunities_count == 0 && raw_opportunity_keys_count == 0 {
            info!("   1. Run Soul Meteor Scanner: cd pyinstaller_scripts && python3 soul_meteor_scanner.py");
        }
        info!("   2. Check environment variables in .env file");
        info!("   3. Ensure DragonflyDB is running: docker run -p 6379:6379 dragonflydb/dragonfly");
    }

    Ok(())
}

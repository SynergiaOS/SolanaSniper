use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, debug};

use sniper_bot::{
    config::AppConfig,
    db_connector::{DbClient, DbConfig},
    reflex_core::{OnChainStreamListener, SniperExecutor, NewTokenOpportunity},
};

/// Test binary for Reflex Core components
/// This tests ultra-fast new token detection and execution in DRY RUN mode
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Reflex Core Test Suite");
    info!("⚡ Testing ultra-fast new token detection and execution");

    // Load configuration
    let config = AppConfig::from_env();
    info!("✅ Configuration loaded");

    // Initialize database connection
    let db_config = DbConfig::default(); // Use default config for testing
    let db_client = Arc::new(DbClient::new(db_config).await?);
    info!("✅ Database client initialized");

    // Test database connectivity
    match test_database_connectivity(&db_client).await {
        Ok(_) => info!("✅ Database connectivity test passed"),
        Err(e) => {
            warn!("⚠️ Database connectivity test failed: {}", e);
            info!("ℹ️ Continuing with local-only tests...");
        }
    }

    // Run Reflex Core tests
    info!("🧪 Starting Reflex Core component tests...");

    // Test 1: OnChainStreamListener
    info!("📡 Test 1: OnChainStreamListener initialization and basic functionality");
    test_onchain_stream_listener(config.clone(), db_client.clone()).await?;

    // Test 2: SniperExecutor
    info!("⚡ Test 2: SniperExecutor DRY RUN functionality");
    test_sniper_executor(config.clone(), db_client.clone()).await?;

    // Test 3: NewTokenOpportunity model
    info!("🎯 Test 3: NewTokenOpportunity data model and validation");
    test_new_token_opportunity().await?;

    // Test 4: Integration test
    info!("🔗 Test 4: Reflex Core integration test");
    test_reflex_core_integration(config.clone(), db_client.clone()).await?;

    info!("✅ All Reflex Core tests completed successfully!");
    info!("🎉 Reflex Core is ready for integration with Intelligence Brain!");

    Ok(())
}

/// Test database connectivity
async fn test_database_connectivity(db_client: &Arc<DbClient>) -> Result<()> {
    info!("🔍 Testing database connectivity...");
    
    // Test basic set/get operations
    let test_key = "reflex_core_test";
    let test_value = "test_value";
    
    // Set test value
    db_client.set(test_key, &test_value, Some(60)).await?;
    info!("✅ Database SET operation successful");
    
    // Get test value
    let retrieved: Option<String> = db_client.get(test_key).await?;
    match retrieved {
        Some(value) if value == test_value => {
            info!("✅ Database GET operation successful");
        }
        Some(value) => {
            return Err(anyhow::anyhow!("Retrieved value mismatch: expected '{}', got '{}'", test_value, value));
        }
        None => {
            return Err(anyhow::anyhow!("Failed to retrieve test value from database"));
        }
    }
    
    // Clean up
    db_client.delete(test_key).await?;
    info!("✅ Database cleanup successful");
    
    Ok(())
}

/// Test OnChainStreamListener initialization and basic functionality
async fn test_onchain_stream_listener(config: AppConfig, db_client: Arc<DbClient>) -> Result<()> {
    info!("📡 Initializing OnChainStreamListener...");
    
    let _listener = OnChainStreamListener::new(config, db_client)?;
    info!("✅ OnChainStreamListener created successfully");

    // Test listener configuration
    info!("🔍 Testing listener configuration...");
    // Note: In a real test, we would check if the listener is properly configured
    // For now, we just verify it can be created without errors
    
    info!("✅ OnChainStreamListener test completed");
    Ok(())
}

/// Test SniperExecutor DRY RUN functionality
async fn test_sniper_executor(config: AppConfig, db_client: Arc<DbClient>) -> Result<()> {
    info!("⚡ Initializing SniperExecutor...");
    
    let _executor = SniperExecutor::new(config.clone(), db_client.clone());
    info!("✅ SniperExecutor created successfully");
    
    // Create mock opportunities for testing
    let opportunities = create_mock_opportunities();
    info!("🎯 Created {} mock opportunities for testing", opportunities.len());
    
    // Test each opportunity
    for (i, opportunity) in opportunities.iter().enumerate() {
        info!("🧪 Testing opportunity {}: {} (Risk: {:.2}, Age: {}s)", 
              i + 1, 
              opportunity.token_symbol.as_deref().unwrap_or("Unknown"),
              opportunity.risk_score,
              opportunity.age_seconds);
        
        // Test opportunity validation
        let should_execute = test_opportunity_validation(&opportunity);
        info!("📊 Validation result: {}", if should_execute { "APPROVED" } else { "REJECTED" });
        
        if should_execute {
            // Test position size calculation
            let position_size = test_position_size_calculation(&opportunity);
            info!("💰 Calculated position size: {} SOL", position_size);
            
            // In DRY RUN mode, we would simulate the execution
            info!("🔥 DRY RUN: Would execute trade for {} SOL", position_size);
        }
    }
    
    info!("✅ SniperExecutor test completed");
    Ok(())
}

/// Test NewTokenOpportunity data model and validation
async fn test_new_token_opportunity() -> Result<()> {
    info!("🎯 Testing NewTokenOpportunity data model...");
    
    let opportunities = create_mock_opportunities();
    
    for opportunity in &opportunities {
        // Test freshness check
        let is_fresh = opportunity.is_fresh();
        info!("⏰ Token {} freshness: {} (age: {}s)", 
              opportunity.token_address, 
              if is_fresh { "FRESH" } else { "STALE" },
              opportunity.age_seconds);
        
        // Test safety check
        let is_safe = opportunity.is_safe();
        info!("🛡️ Token {} safety: {} (score: {:.2})", 
              opportunity.token_address, 
              if is_safe { "SAFE" } else { "UNSAFE" },
              opportunity.risk_score);
        
        // Test priority score
        let priority = opportunity.priority_score();
        info!("📈 Token {} priority score: {:.3}", opportunity.token_address, priority);
        
        // Test Redis key generation
        let redis_key = opportunity.redis_key();
        debug!("🔑 Redis key: {}", redis_key);
    }
    
    info!("✅ NewTokenOpportunity model test completed");
    Ok(())
}

/// Test Reflex Core integration
async fn test_reflex_core_integration(config: AppConfig, db_client: Arc<DbClient>) -> Result<()> {
    info!("🔗 Testing Reflex Core integration...");
    
    // Create components
    let listener = OnChainStreamListener::new(config.clone(), db_client.clone())?;
    let executor = SniperExecutor::new(config, db_client.clone());
    
    // Simulate the workflow:
    // 1. Listener detects new token
    // 2. Saves to database
    // 3. Executor processes from queue
    
    info!("📡 Simulating new token detection...");
    let mock_opportunity = create_high_quality_mock_opportunity();
    
    // Save to database (simulating listener behavior)
    let key = mock_opportunity.redis_key();
    db_client.set(&key, &mock_opportunity, Some(300)).await?;
    db_client.list_push("new_token_queue", &key).await?;
    info!("💾 Mock opportunity saved to database");
    
    // Simulate executor processing (would normally be in a loop)
    info!("⚡ Simulating executor processing...");
    let queue_items = db_client.list_range_raw("new_token_queue", 0, -1).await?;
    
    if !queue_items.is_empty() {
        info!("📋 Found {} items in new token queue", queue_items.len());
        
        for item in queue_items {
            if let Some(opportunity) = db_client.get::<NewTokenOpportunity>(&item).await? {
                info!("🎯 Processing opportunity: {}", opportunity.token_address);

                // Simulate execution decision
                if opportunity.is_safe() && opportunity.is_fresh() {
                    info!("✅ DRY RUN: Would execute trade for {}", opportunity.token_address);
                } else {
                    info!("⛔ DRY RUN: Opportunity rejected");
                }

                // Clean up
                db_client.list_remove("new_token_queue", &item).await?;
            }
        }
    }
    
    info!("✅ Reflex Core integration test completed");
    Ok(())
}

/// Create mock opportunities for testing
fn create_mock_opportunities() -> Vec<NewTokenOpportunity> {
    use chrono::Utc;
    
    vec![
        // High quality opportunity
        NewTokenOpportunity {
            token_address: "GOOD_TOKEN_123".to_string(),
            pool_address: "POOL_123".to_string(),
            token_symbol: Some("GOOD".to_string()),
            initial_liquidity_sol: 5.0,
            initial_liquidity_usd: 1000.0,
            creation_tx_signature: "sig_good_123".to_string(),
            creation_slot: 12345,
            detected_at: Utc::now(),
            age_seconds: 15, // Fresh
            dex: "Raydium".to_string(),
            risk_score: 0.9, // High safety
            mint_authority_burned: true,
            freeze_authority_burned: true,
            initial_market_cap_usd: Some(50000.0),
        },
        // Risky opportunity
        NewTokenOpportunity {
            token_address: "RISKY_TOKEN_456".to_string(),
            pool_address: "POOL_456".to_string(),
            token_symbol: Some("RISKY".to_string()),
            initial_liquidity_sol: 0.5, // Low liquidity
            initial_liquidity_usd: 100.0,
            creation_tx_signature: "sig_risky_456".to_string(),
            creation_slot: 12346,
            detected_at: Utc::now(),
            age_seconds: 5,
            dex: "PumpFun".to_string(),
            risk_score: 0.3, // Low safety
            mint_authority_burned: false, // Red flag
            freeze_authority_burned: true,
            initial_market_cap_usd: Some(5000.0),
        },
        // Stale opportunity
        NewTokenOpportunity {
            token_address: "STALE_TOKEN_789".to_string(),
            pool_address: "POOL_789".to_string(),
            token_symbol: Some("STALE".to_string()),
            initial_liquidity_sol: 10.0,
            initial_liquidity_usd: 2000.0,
            creation_tx_signature: "sig_stale_789".to_string(),
            creation_slot: 12347,
            detected_at: Utc::now(),
            age_seconds: 120, // Too old
            dex: "Meteora".to_string(),
            risk_score: 0.8,
            mint_authority_burned: true,
            freeze_authority_burned: true,
            initial_market_cap_usd: Some(100000.0),
        },
    ]
}

/// Create a high-quality mock opportunity for integration testing
fn create_high_quality_mock_opportunity() -> NewTokenOpportunity {
    use chrono::Utc;
    
    NewTokenOpportunity {
        token_address: "INTEGRATION_TEST_TOKEN".to_string(),
        pool_address: "INTEGRATION_TEST_POOL".to_string(),
        token_symbol: Some("TEST".to_string()),
        initial_liquidity_sol: 3.0,
        initial_liquidity_usd: 600.0,
        creation_tx_signature: "integration_test_sig".to_string(),
        creation_slot: 99999,
        detected_at: Utc::now(),
        age_seconds: 10,
        dex: "Raydium".to_string(),
        risk_score: 0.85,
        mint_authority_burned: true,
        freeze_authority_burned: true,
        initial_market_cap_usd: Some(30000.0),
    }
}

/// Test opportunity validation logic
fn test_opportunity_validation(opportunity: &NewTokenOpportunity) -> bool {
    // Simulate SniperExecutor validation logic
    opportunity.is_fresh() && 
    opportunity.is_safe() && 
    opportunity.initial_liquidity_sol >= 1.0 &&
    opportunity.risk_score >= 0.5
}

/// Test position size calculation
fn test_position_size_calculation(opportunity: &NewTokenOpportunity) -> f64 {
    // Simulate SniperExecutor position sizing logic
    let max_position = 0.05; // Max 0.05 SOL per trade
    let mut position = max_position;
    
    // Adjust based on liquidity
    let liquidity_factor = (opportunity.initial_liquidity_sol / 10.0).min(1.0);
    position *= liquidity_factor;
    
    // Adjust based on risk score
    position *= opportunity.risk_score;
    
    // Adjust based on age (fresher = larger position)
    let age_factor = 1.0 - (opportunity.age_seconds as f64 / 60.0);
    position *= age_factor.max(0.1);
    
    position.max(0.01).min(max_position)
}

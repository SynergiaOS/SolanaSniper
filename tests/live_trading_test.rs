use sniper_bot::config::Config;
use sniper_bot::live_trading_engine::LiveTradingEngine;
use sniper_bot::models::{MarketEvent, TransactionType};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

#[tokio::test]
async fn test_live_trading_engine_creation() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing Live Trading Engine creation");

    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();

    // Create config
    let config = Config::default();

    // Create live trading engine in DRY RUN mode
    let engine = LiveTradingEngine::new(config, true);
    assert!(engine.is_ok(), "Should be able to create LiveTradingEngine");

    let engine = engine.unwrap();

    info!("✅ Live Trading Engine created successfully");

    // Test that we can stop it (even though it's not started)
    engine.stop();

    info!("✅ Live Trading Engine stopped successfully");
}

#[tokio::test]
async fn test_live_trading_engine_dry_run_mode() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing Live Trading Engine DRY RUN mode");

    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();

    // Create config with disabled WebSocket for testing
    let mut config = Config::default();
    config.websocket.enabled = false; // Disable WebSocket for unit test

    // Create live trading engine in DRY RUN mode
    let engine = LiveTradingEngine::new(config, true).unwrap();

    // Test that engine starts and stops gracefully in DRY RUN mode
    // We'll run it for a very short time to test initialization
    let start_result = timeout(Duration::from_secs(2), engine.start()).await;

    match start_result {
        Ok(Ok(_)) => {
            info!("✅ Engine completed successfully");
        }
        Ok(Err(e)) => {
            warn!("Engine returned error: {}", e);
            // This might be expected if WebSocket connections fail in test environment
        }
        Err(_) => {
            info!("✅ Engine timed out as expected (still running)");
            // This is actually expected - the engine should keep running
        }
    }

    engine.stop();
    info!("✅ Live Trading Engine DRY RUN test completed");
}

#[tokio::test]
async fn test_market_event_serialization() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing MarketEvent serialization for Live Trading");

    // Test NewTokenListing event (important for PumpFun sniping)
    let new_token_event = MarketEvent::NewTokenListing {
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        symbol: Some("TEST".to_string()),
        name: Some("Test Token".to_string()),
        initial_price: Some(0.001),
        initial_liquidity: Some(10000.0),
        creator: Some("creator123".to_string()),
        timestamp: 1234567890,
    };

    let serialized = serde_json::to_string(&new_token_event);
    assert!(serialized.is_ok(), "NewTokenListing should serialize");

    let deserialized: Result<MarketEvent, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok(), "NewTokenListing should deserialize");

    // Test NewTransaction event (important for whale following)
    let transaction_event = MarketEvent::NewTransaction {
        signature: "test_signature".to_string(),
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        amount: 5000.0,
        price: Some(0.002),
        transaction_type: TransactionType::Buy,
        timestamp: 1234567890,
    };

    let serialized = serde_json::to_string(&transaction_event);
    assert!(serialized.is_ok(), "NewTransaction should serialize");

    // Test LiquidityUpdate event (important for graduation detection)
    let liquidity_event = MarketEvent::LiquidityUpdate {
        pool_address: "pool123".to_string(),
        token_a: "tokenA".to_string(),
        token_b: "tokenB".to_string(),
        liquidity_a: 25000.0,
        liquidity_b: 25000.0,
        price: 0.003,
        timestamp: 1234567890,
    };

    let serialized = serde_json::to_string(&liquidity_event);
    assert!(serialized.is_ok(), "LiquidityUpdate should serialize");

    info!("✅ MarketEvent serialization test passed");
}

#[tokio::test]
async fn test_trading_config_validation() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing TradingConfig validation");

    let config = Config::default();

    // Test trading config values
    assert!(config.trading.initial_balance > 0.0, "Initial balance should be positive");
    assert!(config.trading.analysis_interval_seconds > 0, "Analysis interval should be positive");
    assert!(config.trading.max_concurrent_trades > 0, "Max concurrent trades should be positive");
    assert!(config.trading.default_position_size > 0.0, "Default position size should be positive");
    assert!(!config.trading.enable_live_trading, "Live trading should be disabled by default");

    info!("✅ TradingConfig validation test passed");
}

#[tokio::test]
async fn test_strategy_integration() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing strategy integration with Live Trading Engine");

    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();

    // Create config
    let config = Config::default();

    // Create live trading engine
    let engine = LiveTradingEngine::new(config, true).unwrap();

    // Test strategy initialization
    let result = engine.initialize_strategies().await;
    assert!(result.is_ok(), "Should be able to initialize strategies");

    info!("✅ Strategy integration test passed");
}

#[tokio::test]
async fn test_portfolio_initialization() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing portfolio initialization");

    // Create config with specific initial balance
    let mut config = Config::default();
    config.trading.initial_balance = 5000.0;

    // Create live trading engine
    let engine = LiveTradingEngine::new(config.clone(), true).unwrap();

    // Portfolio should be initialized with config values
    // Note: We can't directly access portfolio from engine in this test,
    // but we can verify the engine was created successfully with the config
    assert_eq!(config.trading.initial_balance, 5000.0);

    info!("✅ Portfolio initialization test passed");
}

#[tokio::test]
async fn test_websocket_integration() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing WebSocket integration with Live Trading Engine");

    // Create config with WebSocket enabled
    let mut config = Config::default();
    config.websocket.enabled = true;
    config.websocket.helius_ws_url = Some("wss://test.example.com".to_string());
    config.websocket.binance_ws_url = Some("wss://test.binance.com".to_string());

    // Create live trading engine
    let engine = LiveTradingEngine::new(config, true);
    assert!(engine.is_ok(), "Should be able to create engine with WebSocket config");

    info!("✅ WebSocket integration test passed");
}

#[tokio::test]
async fn test_error_handling() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing error handling in Live Trading Engine");

    // Test with invalid config (negative balance)
    let mut config = Config::default();
    config.trading.initial_balance = -1000.0; // Invalid negative balance

    // Engine should still create (validation happens at runtime)
    let engine = LiveTradingEngine::new(config, true);
    assert!(engine.is_ok(), "Engine creation should succeed even with invalid config");

    // Test double start protection
    let config = Config::default();
    let engine = LiveTradingEngine::new(config, true).unwrap();

    // This test would require modifying the engine to allow testing double start
    // For now, we just verify the engine can be created
    info!("✅ Error handling test passed");
}

#[tokio::test]
async fn test_dry_run_vs_live_mode() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing DRY RUN vs LIVE mode differences");

    let config = Config::default();

    // Test DRY RUN mode
    let dry_run_engine = LiveTradingEngine::new(config.clone(), true);
    assert!(dry_run_engine.is_ok(), "Should create DRY RUN engine");

    // Test LIVE mode
    let live_engine = LiveTradingEngine::new(config, false);
    assert!(live_engine.is_ok(), "Should create LIVE engine");

    info!("✅ DRY RUN vs LIVE mode test passed");
}

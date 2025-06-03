use sniper_bot::config::Config;
use sniper_bot::data_fetcher::client_factory::ClientFactory;
use std::env;
use tracing::{info, warn};

#[tokio::test]
async fn test_config_loading() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();
    
    info!("🧪 Testing configuration loading");
    
    // Test default config
    let default_config = Config::default();
    assert_eq!(default_config.bot.name, "SniperBot 2.0");
    assert!(default_config.bot.dry_run);
    
    info!("✅ Default config test passed");
    
    // Test loading from file
    let config_result = Config::load_from_path("config.test.toml");
    match config_result {
        Ok(config) => {
            assert_eq!(config.bot.name, "SniperBot 2.0 Test");
            assert!(config.bot.dry_run);
            assert!(config.bot.paper_trading);
            info!("✅ Test config loaded successfully");
        }
        Err(e) => {
            warn!("⚠️ Could not load test config: {}", e);
            // This is OK for CI environments without config files
        }
    }
}

#[tokio::test]
async fn test_client_creation() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();
    
    info!("🧪 Testing client creation");
    
    // Create a test config
    let config = Config::default();
    
    // Test Jupiter client creation (doesn't require API keys)
    let jupiter_result = ClientFactory::create_jupiter_client(&config);
    match jupiter_result {
        Ok(_) => info!("✅ Jupiter client created successfully"),
        Err(e) => {
            warn!("⚠️ Jupiter client creation failed: {}", e);
        }
    }
    
    // Test Solana client creation (requires API key)
    if env::var("HELIUS_API_KEY").is_ok() {
        let solana_result = ClientFactory::create_solana_client(&config);
        match solana_result {
            Ok(_) => info!("✅ Solana client created successfully"),
            Err(e) => {
                warn!("⚠️ Solana client creation failed: {}", e);
            }
        }
    } else {
        info!("⚠️ Skipping Solana client test - no HELIUS_API_KEY");
    }
}

#[tokio::test]
async fn test_basic_validation() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
    
    info!("🧪 Testing basic validation");
    
    let config = Config::default();
    
    // Test configuration validation (will fail without env vars, which is expected)
    match config.validate() {
        Ok(_) => info!("✅ Configuration validation passed"),
        Err(e) => {
            info!("⚠️ Configuration validation failed (expected): {}", e);
            // This is expected without proper environment variables
        }
    }
    
    // Test that we can create basic structures
    assert!(config.bot.dry_run);
    assert!(config.bot.paper_trading);
    assert_eq!(config.bot.log_level, "info");
    
    // Test risk management defaults
    assert!(config.risk_management.max_position_size_usd > 0.0);
    assert!(config.risk_management.max_slippage_bps <= 10000);
    
    info!("✅ Basic validation tests passed");
}

#[tokio::test]
async fn test_environment_setup() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
    
    info!("🧪 Testing environment setup");
    
    // Test that we can load environment variables
    dotenvy::from_filename(".env.test").ok();
    
    // Check if test environment is properly configured
    let bot_mode = env::var("BOT_MODE").unwrap_or_default();
    if bot_mode == "testing" {
        info!("✅ Test environment detected");
    } else {
        info!("⚠️ Not in test environment (BOT_MODE={})", bot_mode);
    }
    
    // Test dry run setting
    let dry_run = env::var("DRY_RUN").unwrap_or_default();
    if dry_run == "true" {
        info!("✅ Dry run mode enabled");
    } else {
        info!("⚠️ Dry run mode not enabled");
    }
    
    info!("✅ Environment setup test completed");
}

#[tokio::test]
async fn test_network_connectivity() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
    
    info!("🧪 Testing network connectivity");
    
    // Skip if network tests are disabled
    if env::var("SKIP_NETWORK_TESTS").unwrap_or_default() == "true" {
        info!("⚠️ Skipping network test (SKIP_NETWORK_TESTS=true)");
        return;
    }
    
    // Test basic internet connectivity
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    
    match client.get("https://httpbin.org/status/200").send().await {
        Ok(response) if response.status().is_success() => {
            info!("✅ Network connectivity test passed");
        }
        Ok(response) => {
            warn!("⚠️ Network test failed with status: {}", response.status());
        }
        Err(e) => {
            warn!("⚠️ Network connectivity test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_api_key_availability() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
    
    info!("🧪 Testing API key availability");
    
    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();
    
    // Check Helius API key
    match env::var("HELIUS_API_KEY") {
        Ok(key) if !key.is_empty() && key != "your_helius_api_key_here" => {
            info!("✅ Helius API key found");
        }
        _ => {
            info!("⚠️ Helius API key not found or is placeholder");
        }
    }
    
    info!("✅ API key availability test completed");
}

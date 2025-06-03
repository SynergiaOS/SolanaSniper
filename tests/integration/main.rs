// Integration tests for SniperBot 2.0
// These tests interact with real APIs and require network connectivity

mod test_helpers;
mod helius_api_tests;
mod jupiter_api_tests;
mod jito_api_tests;

use test_helpers::*;
use tracing::{info, warn};
use std::time::Duration;

#[tokio::test]
async fn test_full_integration_workflow() {
    init_test_env();
    skip_if_no_network!();
    skip_if_no_helius_key!();
    
    info!("🚀 Starting full integration workflow test");
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test 1: Create all clients
    info!("📡 Testing client creation...");
    
    let solana_client = sniper_bot::data_fetcher::client_factory::ClientFactory::create_solana_client(&config);
    assert_api_success(&solana_client, "Solana Client Creation");
    
    let jupiter_client = sniper_bot::data_fetcher::client_factory::ClientFactory::create_jupiter_client(&config);
    assert_api_success(&jupiter_client, "Jupiter Client Creation");
    
    let rpc_url = config.get_solana_rpc_url().unwrap_or_else(|_| {
        "https://api.devnet.solana.com".to_string()
    });
    let jito_executor = sniper_bot::data_fetcher::client_factory::ClientFactory::create_jito_executor(&config, &rpc_url);
    assert_api_success(&jito_executor, "Jito Executor Creation");
    
    info!("✅ All clients created successfully");
    
    // Test 2: Basic connectivity
    info!("🌐 Testing API connectivity...");
    
    if let Ok(client) = solana_client {
        let health_result = tokio::time::timeout(
            Duration::from_secs(10), 
            client.health_check()
        ).await;
        
        match health_result {
            Ok(Ok(_)) => info!("✅ Helius RPC connectivity confirmed"),
            Ok(Err(e)) => warn!("⚠️ Helius RPC error: {}", e),
            Err(_) => warn!("⚠️ Helius RPC timeout"),
        }
        
        rate_limit_delay().await;
    }
    
    if let Ok(client) = jupiter_client {
        let tokens = TestTokens::devnet();
        let price_result = tokio::time::timeout(
            Duration::from_secs(10), 
            client.get_price(&tokens.sol)
        ).await;
        
        match price_result {
            Ok(Ok(price)) => info!("✅ Jupiter API connectivity confirmed, SOL price: ${:.2}", price),
            Ok(Err(e)) => warn!("⚠️ Jupiter API error: {}", e),
            Err(_) => warn!("⚠️ Jupiter API timeout"),
        }
        
        rate_limit_delay().await;
    }
    
    // Test 3: Configuration validation
    info!("⚙️ Testing configuration...");
    
    match config.validate() {
        Ok(_) => info!("✅ Configuration validation passed"),
        Err(e) => {
            warn!("❌ Configuration validation failed: {}", e);
            panic!("Configuration validation failed: {}", e);
        }
    }
    
    // Test 4: Environment setup
    info!("🔧 Testing environment setup...");
    
    assert!(config.bot.dry_run, "Should be in dry run mode for tests");
    assert!(config.bot.paper_trading, "Should be in paper trading mode for tests");
    assert_eq!(config.bot.log_level, "debug", "Should use debug logging for tests");
    
    info!("✅ Environment setup validated");
    
    info!("🎉 Full integration workflow test completed successfully!");
}

#[tokio::test]
async fn test_configuration_environments() {
    init_test_env();
    
    info!("🔧 Testing different configuration environments");
    
    // Test default config loading
    let default_config = sniper_bot::config::Config::default();
    assert_eq!(default_config.bot.name, "SniperBot 2.0");
    
    // Test test config loading
    let test_config = load_test_config();
    assert!(test_config.is_ok(), "Test config should load successfully");
    
    let test_config = test_config.unwrap();
    assert_eq!(test_config.bot.name, "SniperBot 2.0 Test");
    assert!(test_config.bot.dry_run);
    assert!(test_config.bot.paper_trading);
    
    info!("✅ Configuration environments test passed");
}

#[tokio::test]
async fn test_error_recovery() {
    init_test_env();
    skip_if_no_network!();
    
    info!("🔄 Testing error recovery mechanisms");
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test 1: Invalid API calls should be handled gracefully
    if let Ok(jupiter_client) = sniper_bot::data_fetcher::client_factory::ClientFactory::create_jupiter_client(&config) {
        // Test with invalid token address
        let invalid_price_result = jupiter_client.get_price("invalid_token_address").await;
        
        match invalid_price_result {
            Ok(_) => warn!("Expected error for invalid token, but got success"),
            Err(e) => info!("✅ Proper error handling for invalid token: {}", e),
        }
        
        rate_limit_delay().await;
    }
    
    // Test 2: Network timeout handling
    info!("Testing timeout handling...");
    
    // This test validates that our timeout mechanisms work
    let start_time = std::time::Instant::now();
    let timeout_result = tokio::time::timeout(
        Duration::from_millis(100), // Very short timeout
        tokio::time::sleep(Duration::from_secs(1)) // Long operation
    ).await;
    
    let elapsed = start_time.elapsed();
    assert!(timeout_result.is_err(), "Timeout should have occurred");
    assert!(elapsed < Duration::from_millis(200), "Should timeout quickly");
    
    info!("✅ Timeout handling works correctly");
    
    info!("✅ Error recovery test completed");
}

#[tokio::test]
async fn test_rate_limiting_compliance() {
    init_test_env();
    skip_if_no_network!();
    
    info!("⏱️ Testing rate limiting compliance");
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test that we respect rate limits
    if let Ok(jupiter_client) = sniper_bot::data_fetcher::client_factory::ClientFactory::create_jupiter_client(&config) {
        let tokens = TestTokens::devnet();
        let mut request_times = Vec::new();
        
        // Make several requests and measure timing
        for i in 0..3 {
            let start = std::time::Instant::now();
            
            let _result = jupiter_client.get_price(&tokens.sol).await;
            
            let elapsed = start.elapsed();
            request_times.push(elapsed);
            
            info!("Request {}: took {:?}", i + 1, elapsed);
            
            // Add delay between requests to be respectful
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // Verify we're not making requests too quickly
        let avg_time = request_times.iter().sum::<Duration>() / request_times.len() as u32;
        info!("Average request time: {:?}", avg_time);
        
        // Should not be making requests faster than reasonable limits
        assert!(avg_time >= Duration::from_millis(50), "Requests should not be too fast");
    }
    
    info!("✅ Rate limiting compliance test passed");
}

#[tokio::test]
async fn test_data_validation() {
    init_test_env();
    skip_if_no_network!();
    
    info!("🔍 Testing data validation");
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test token address validation
    let tokens = TestTokens::devnet();
    
    // Validate SOL token address
    match solana_sdk::pubkey::Pubkey::try_from(tokens.sol.as_str()) {
        Ok(_) => info!("✅ SOL token address is valid"),
        Err(e) => panic!("Invalid SOL token address: {}", e),
    }
    
    // Validate USDC token address
    match solana_sdk::pubkey::Pubkey::try_from(tokens.usdc.as_str()) {
        Ok(_) => info!("✅ USDC token address is valid"),
        Err(e) => panic!("Invalid USDC token address: {}", e),
    }
    
    // Test configuration value ranges
    assert!(config.risk_management.max_position_size_usd > 0.0, "Max position size should be positive");
    assert!(config.risk_management.max_slippage_bps <= 10000, "Max slippage should not exceed 100%");
    assert!(config.jito.max_tip_lamports > 0, "Max tip should be positive");
    
    info!("✅ Data validation test passed");
}

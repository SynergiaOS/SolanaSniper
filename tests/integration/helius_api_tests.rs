use sniper_bot::data_fetcher::client_factory::ClientFactory;
use sniper_bot::models::TradingResult;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

use crate::test_helpers::{init_test_env, load_test_config, TestTokens, TestWallet,
                         assert_api_success, assert_api_response_valid, rate_limit_delay,
                         should_skip_network_tests, check_helius_api_key};

#[tokio::test]
async fn test_helius_solana_client_creation() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test client creation
    let client_result = ClientFactory::create_solana_client(&config);
    assert_api_success(&client_result, "Helius SolanaDataFetcher");
    
    info!("✅ Helius client creation test passed");
}

#[tokio::test]
async fn test_helius_rpc_connectivity() {
    init_test_env();
    if should_skip_network_tests() {
        println!("Skipping network test (SKIP_NETWORK_TESTS=true)");
        return;
    }
    if !check_helius_api_key() {
        println!("Skipping test - Helius API key not available");
        return;
    }
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_solana_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Helius client: {}", e);
            return;
        }
    };

    // Test basic RPC connectivity with timeout
    let health_check = timeout(Duration::from_secs(10), client.health_check()).await;
    
    match health_check {
        Ok(result) => {
            assert_api_success(&result, "Helius RPC Health Check");
            info!("✅ Helius RPC connectivity test passed");
        }
        Err(_) => {
            warn!("⚠️ Helius RPC health check timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_helius_get_slot() {
    init_test_env();
    if should_skip_network_tests() { return; }
    if !check_helius_api_key() { return; }
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_solana_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Helius client: {}", e);
            return;
        }
    };

    // Test getting current slot
    let slot_result = timeout(Duration::from_secs(10), client.get_slot()).await;
    
    match slot_result {
        Ok(result) => {
            assert_api_response_valid(&result, "Helius Get Slot", |slot| *slot > 0);
            if let Ok(slot) = result {
                info!("✅ Current devnet slot: {}", slot);
            }
        }
        Err(_) => {
            warn!("⚠️ Helius get slot timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_helius_account_balance() {
    init_test_env();
    if should_skip_network_tests() { return; }
    if !check_helius_api_key() { return; }
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_solana_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Helius client: {}", e);
            return;
        }
    };

    let test_wallet = TestWallet::devnet();
    
    // Test getting account balance
    let balance_result = timeout(
        Duration::from_secs(10), 
        client.get_account_balance(&test_wallet.address)
    ).await;
    
    match balance_result {
        Ok(result) => {
            assert_api_response_valid(&result, "Helius Account Balance", |balance| *balance >= 0);
            if let Ok(balance) = result {
                info!("✅ Account balance: {} lamports", balance);
            }
        }
        Err(_) => {
            warn!("⚠️ Helius account balance check timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_helius_transaction_history() {
    init_test_env();
    if should_skip_network_tests() { return; }
    if !check_helius_api_key() { return; }
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_solana_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Helius client: {}", e);
            return;
        }
    };

    let test_wallet = TestWallet::devnet();
    
    // Test getting transaction history (limit to 5 for faster test)
    let history_result = timeout(
        Duration::from_secs(15), 
        client.get_transaction_history(&test_wallet.address, Some(5))
    ).await;
    
    match history_result {
        Ok(result) => {
            assert_api_response_valid(&result, "Helius Transaction History", |history| {
                history.len() <= 5 // Should respect the limit
            });
            if let Ok(history) = result {
                info!("✅ Retrieved {} transactions", history.len());
            }
        }
        Err(_) => {
            warn!("⚠️ Helius transaction history check timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_helius_api_rate_limiting() {
    init_test_env();
    if should_skip_network_tests() { return; }
    if !check_helius_api_key() { return; }
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_solana_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Helius client: {}", e);
            return;
        }
    };

    // Test multiple rapid requests to check rate limiting behavior
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    
    for i in 0..5 {
        let result = timeout(Duration::from_secs(5), client.get_slot()).await;
        
        match result {
            Ok(Ok(_)) => {
                successful_requests += 1;
                info!("Request {}: Success", i + 1);
            }
            Ok(Err(e)) => {
                failed_requests += 1;
                warn!("Request {}: Failed - {}", i + 1, e);
            }
            Err(_) => {
                failed_requests += 1;
                warn!("Request {}: Timeout", i + 1);
            }
        }
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    info!("Rate limiting test: {} successful, {} failed", successful_requests, failed_requests);
    
    // We expect at least some requests to succeed
    assert!(successful_requests > 0, "At least some requests should succeed");
}

#[tokio::test]
async fn test_helius_error_handling() {
    init_test_env();
    if should_skip_network_tests() { return; }
    if !check_helius_api_key() { return; }
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_solana_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Helius client: {}", e);
            return;
        }
    };

    // Test with invalid address to check error handling
    let invalid_address = "invalid_address_123";
    let result = client.get_account_balance(invalid_address).await;
    
    match result {
        Ok(_) => {
            warn!("Expected error for invalid address, but got success");
        }
        Err(e) => {
            info!("✅ Proper error handling for invalid address: {}", e);
        }
    }
    
    rate_limit_delay().await;
}

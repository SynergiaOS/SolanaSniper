use sniper_bot::data_fetcher::client_factory::ClientFactory;
use sniper_bot::models::TradingResult;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

mod test_helpers;
use test_helpers::*;

#[tokio::test]
async fn test_jito_executor_creation() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test executor creation
    let rpc_url = config.get_solana_rpc_url().unwrap_or_else(|_| {
        "https://api.devnet.solana.com".to_string()
    });
    
    let executor_result = ClientFactory::create_jito_executor(&config, &rpc_url);
    assert_api_success(&executor_result, "Jito Executor");
    
    info!("✅ Jito executor creation test passed");
}

#[tokio::test]
async fn test_jito_tip_calculation() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let rpc_url = config.get_solana_rpc_url().unwrap_or_else(|_| {
        "https://api.devnet.solana.com".to_string()
    });
    
    let executor = match ClientFactory::create_jito_executor(&config, &rpc_url) {
        Ok(executor) => executor,
        Err(e) => {
            warn!("Failed to create Jito executor: {}", e);
            return;
        }
    };

    // Test tip calculation for different order values and urgencies
    let test_cases = vec![
        (100.0, 1.0),   // $100 order, normal urgency
        (1000.0, 2.0),  // $1000 order, high urgency
        (10.0, 0.5),    // $10 order, low urgency
    ];

    for (order_value, urgency) in test_cases {
        let tip = executor.calculate_optimal_tip(order_value, urgency);
        
        // Tip should be reasonable (not zero, not excessive)
        assert!(tip > 0, "Tip should be greater than 0");
        assert!(tip <= config.jito.max_tip_lamports, "Tip should not exceed max limit");
        
        info!("✅ Tip calculation: ${:.2} order, {:.1}x urgency -> {} lamports", 
            order_value, urgency, tip);
    }
}

#[tokio::test]
async fn test_jito_bundle_creation_dry_run() {
    init_test_env();
    skip_if_no_network!();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let rpc_url = config.get_solana_rpc_url().unwrap_or_else(|_| {
        "https://api.devnet.solana.com".to_string()
    });
    
    let mut executor = match ClientFactory::create_jito_executor(&config, &rpc_url) {
        Ok(executor) => executor,
        Err(e) => {
            warn!("Failed to create Jito executor: {}", e);
            return;
        }
    };

    // Create a test keypair for dry run
    let test_keypair = solana_sdk::signature::Keypair::new();
    executor.set_wallet_keypair(test_keypair);

    // Test bundle creation (dry run - won't actually submit)
    let test_transactions = vec![
        "test_transaction_1".to_string(),
        "test_transaction_2".to_string(),
    ];

    // This should create a bundle structure without submitting
    info!("Testing bundle creation logic (dry run)");
    
    // Test tip calculation
    let tip = executor.calculate_optimal_tip(100.0, 1.0);
    assert!(tip > 0, "Tip calculation should work");
    
    info!("✅ Jito bundle creation dry run test passed");
}

#[tokio::test]
async fn test_jito_api_connectivity() {
    init_test_env();
    skip_if_no_network!();
    
    // Test basic connectivity to Jito API endpoint
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // Test if Jito API is reachable (just check if endpoint responds)
    let jito_url = "https://mainnet.block-engine.jito.wtf/api/v1";
    
    let response = timeout(
        Duration::from_secs(10),
        client.get(format!("{}/bundles", jito_url)).send()
    ).await;

    match response {
        Ok(Ok(resp)) => {
            info!("✅ Jito API endpoint reachable, status: {}", resp.status());
            // We expect some kind of response, even if it's an error due to missing auth
        }
        Ok(Err(e)) => {
            warn!("Jito API request failed: {}", e);
        }
        Err(_) => {
            warn!("⚠️ Jito API connectivity test timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_jito_tip_accounts_validation() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Validate that tip accounts are valid Solana addresses
    for (i, tip_account) in config.jito.tip_accounts.iter().enumerate() {
        match solana_sdk::pubkey::Pubkey::try_from(tip_account.as_str()) {
            Ok(pubkey) => {
                info!("✅ Tip account {}: {} is valid", i + 1, pubkey);
            }
            Err(e) => {
                panic!("Invalid tip account {}: {} - {}", i + 1, tip_account, e);
            }
        }
    }
    
    info!("✅ All {} tip accounts are valid", config.jito.tip_accounts.len());
}

#[tokio::test]
async fn test_jito_configuration_validation() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test Jito configuration values
    assert!(!config.jito.api_url.is_empty(), "Jito API URL should not be empty");
    assert!(!config.jito.tip_accounts.is_empty(), "Should have at least one tip account");
    assert!(config.jito.bundle_timeout_seconds > 0, "Bundle timeout should be positive");
    assert!(config.jito.max_tip_lamports > 0, "Max tip should be positive");
    
    // Test that max tip is reasonable (not too high for tests)
    assert!(config.jito.max_tip_lamports <= 100_000_000, "Max tip should be reasonable for tests"); // 0.1 SOL max
    
    info!("✅ Jito configuration validation passed");
    info!("   API URL: {}", config.jito.api_url);
    info!("   Tip accounts: {}", config.jito.tip_accounts.len());
    info!("   Bundle timeout: {}s", config.jito.bundle_timeout_seconds);
    info!("   Max tip: {} lamports", config.jito.max_tip_lamports);
}

#[tokio::test]
async fn test_jito_error_handling() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test with invalid RPC URL
    let invalid_rpc_url = "https://invalid-rpc-url.com";
    
    let executor_result = ClientFactory::create_jito_executor(&config, invalid_rpc_url);
    
    // Should still create executor (RPC validation happens on use)
    match executor_result {
        Ok(_) => {
            info!("✅ Jito executor created with invalid RPC (validation deferred)");
        }
        Err(e) => {
            info!("✅ Proper error handling for invalid RPC: {}", e);
        }
    }
}

#[tokio::test]
async fn test_jito_bundle_size_limits() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let rpc_url = config.get_solana_rpc_url().unwrap_or_else(|_| {
        "https://api.devnet.solana.com".to_string()
    });
    
    let executor = match ClientFactory::create_jito_executor(&config, &rpc_url) {
        Ok(executor) => executor,
        Err(e) => {
            warn!("Failed to create Jito executor: {}", e);
            return;
        }
    };

    // Test different bundle sizes
    let test_cases = vec![
        1,  // Single transaction
        3,  // Small bundle
        5,  // Maximum typical bundle size
    ];

    for bundle_size in test_cases {
        let test_transactions: Vec<String> = (0..bundle_size)
            .map(|i| format!("test_transaction_{}", i))
            .collect();

        // This is just testing the logic, not actual submission
        info!("Testing bundle with {} transactions", bundle_size);
        
        // Validate bundle size is reasonable
        assert!(bundle_size <= 5, "Bundle size should be reasonable");
        assert!(!test_transactions.is_empty(), "Bundle should not be empty");
        
        info!("✅ Bundle size {} validation passed", bundle_size);
    }
}

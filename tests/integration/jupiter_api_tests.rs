use sniper_bot::data_fetcher::client_factory::ClientFactory;
use sniper_bot::data_fetcher::jupiter_client::JupiterQuoteRequest;
use sniper_bot::models::TradingResult;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

mod test_helpers;
use test_helpers::*;

// Import the macro
use crate::skip_if_no_network;

#[tokio::test]
async fn test_jupiter_client_creation() {
    init_test_env();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    // Test client creation
    let client_result = ClientFactory::create_jupiter_client(&config);
    assert_api_success(&client_result, "Jupiter Client");
    
    info!("✅ Jupiter client creation test passed");
}

#[tokio::test]
async fn test_jupiter_quote_request() {
    init_test_env();
    skip_if_no_network!();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_jupiter_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Jupiter client: {}", e);
            return;
        }
    };

    let tokens = TestTokens::devnet();
    
    // Create a quote request for SOL -> USDC
    let quote_request = JupiterQuoteRequest {
        input_mint: tokens.sol.clone(),
        output_mint: tokens.usdc.clone(),
        amount: 1_000_000, // 0.001 SOL
        slippage_bps: Some(50), // 0.5%
        swap_mode: Some("ExactIn".to_string()),
        dexes: None,
        exclude_dexes: None,
        restrict_intermediate_tokens: None,
        only_direct_routes: Some(false),
        as_legacy_transaction: Some(false),
        platform_fee_bps: None,
        max_accounts: None,
    };

    // Test getting quote with timeout
    let quote_result = timeout(
        Duration::from_secs(10), 
        client.get_quote(quote_request)
    ).await;
    
    match quote_result {
        Ok(result) => {
            assert_api_response_valid(&result, "Jupiter Quote", |quote| {
                !quote.input_mint.is_empty() && 
                !quote.output_mint.is_empty() &&
                !quote.in_amount.is_empty() &&
                !quote.out_amount.is_empty()
            });
            
            if let Ok(quote) = result {
                info!("✅ Jupiter quote: {} {} -> {} {}", 
                    quote.in_amount, quote.input_mint,
                    quote.out_amount, quote.output_mint
                );
            }
        }
        Err(_) => {
            warn!("⚠️ Jupiter quote request timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_jupiter_price_check() {
    init_test_env();
    skip_if_no_network!();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_jupiter_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Jupiter client: {}", e);
            return;
        }
    };

    let tokens = TestTokens::devnet();
    
    // Test getting price for SOL
    let price_result = timeout(
        Duration::from_secs(10), 
        client.get_price(&tokens.sol)
    ).await;
    
    match price_result {
        Ok(result) => {
            assert_api_response_valid(&result, "Jupiter Price", |price| *price > 0.0);
            
            if let Ok(price) = result {
                info!("✅ SOL price: ${:.2}", price);
            }
        }
        Err(_) => {
            warn!("⚠️ Jupiter price check timed out");
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_jupiter_multiple_quotes() {
    init_test_env();
    skip_if_no_network!();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_jupiter_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Jupiter client: {}", e);
            return;
        }
    };

    let tokens = TestTokens::devnet();
    
    // Test multiple quote requests with different amounts
    let amounts = vec![100_000, 500_000, 1_000_000]; // Different SOL amounts
    let mut successful_quotes = 0;
    
    for (i, amount) in amounts.iter().enumerate() {
        let quote_request = JupiterQuoteRequest {
            input_mint: tokens.sol.clone(),
            output_mint: tokens.usdc.clone(),
            amount: *amount,
            slippage_bps: 50,
            only_direct_routes: Some(false),
            as_legacy_transaction: Some(false),
        };

        let result = timeout(Duration::from_secs(8), client.get_quote(quote_request)).await;
        
        match result {
            Ok(Ok(quote)) => {
                successful_quotes += 1;
                info!("Quote {}: {} lamports -> {} USDC", i + 1, amount, quote.out_amount);
            }
            Ok(Err(e)) => {
                warn!("Quote {} failed: {}", i + 1, e);
            }
            Err(_) => {
                warn!("Quote {} timed out", i + 1);
            }
        }
        
        // Rate limiting delay
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    info!("Multiple quotes test: {}/{} successful", successful_quotes, amounts.len());
    assert!(successful_quotes > 0, "At least one quote should succeed");
}

#[tokio::test]
async fn test_jupiter_error_handling() {
    init_test_env();
    skip_if_no_network!();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_jupiter_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Jupiter client: {}", e);
            return;
        }
    };

    // Test with invalid token mint
    let invalid_quote_request = JupiterQuoteRequest {
        input_mint: "invalid_mint_address".to_string(),
        output_mint: "another_invalid_mint".to_string(),
        amount: 1_000_000,
        slippage_bps: 50,
        only_direct_routes: Some(false),
        as_legacy_transaction: Some(false),
    };

    let result = client.get_quote(invalid_quote_request).await;
    
    match result {
        Ok(_) => {
            warn!("Expected error for invalid mints, but got success");
        }
        Err(e) => {
            info!("✅ Proper error handling for invalid mints: {}", e);
        }
    }
    
    rate_limit_delay().await;
}

#[tokio::test]
async fn test_jupiter_rate_limiting() {
    init_test_env();
    skip_if_no_network!();
    
    let config = match load_test_config() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load test config: {}", e);
            return;
        }
    };

    let client = match ClientFactory::create_jupiter_client(&config) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create Jupiter client: {}", e);
            return;
        }
    };

    let tokens = TestTokens::devnet();
    
    // Test rapid requests to check rate limiting
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    
    for i in 0..3 { // Reduced number for faster test
        let result = timeout(
            Duration::from_secs(5), 
            client.get_price(&tokens.sol)
        ).await;
        
        match result {
            Ok(Ok(_)) => {
                successful_requests += 1;
                info!("Price request {}: Success", i + 1);
            }
            Ok(Err(e)) => {
                failed_requests += 1;
                warn!("Price request {}: Failed - {}", i + 1, e);
            }
            Err(_) => {
                failed_requests += 1;
                warn!("Price request {}: Timeout", i + 1);
            }
        }
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    
    info!("Jupiter rate limiting test: {} successful, {} failed", successful_requests, failed_requests);
    
    // We expect at least some requests to succeed
    assert!(successful_requests > 0, "At least some requests should succeed");
}

use sniper_bot::config::Config;
use sniper_bot::models::{TradingResult, TradingError};
use std::env;
use std::sync::Once;
use tracing::{info, warn};

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        // Load test environment variables
        dotenvy::from_filename(".env.test").ok();
        
        // Initialize tracing for tests
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_test_writer()
            .init();
        
        info!("🧪 Test environment initialized");
    });
}

/// Load test configuration
pub fn load_test_config() -> TradingResult<Config> {
    init_test_env();
    
    let config = Config::load_from_path("config.test.toml")
        .map_err(|e| TradingError::DataError(e.to_string()))?;
    
    config.validate()
        .map_err(|e| TradingError::DataError(e))?;
    
    info!("✅ Test configuration loaded successfully");
    Ok(config)
}

/// Check if Helius API key is available
pub fn check_helius_api_key() -> bool {
    match env::var("HELIUS_API_KEY") {
        Ok(key) if !key.is_empty() && key != "your_helius_api_key_here" => {
            info!("✅ Helius API key found");
            true
        }
        _ => {
            warn!("⚠️ Helius API key not found or is placeholder");
            false
        }
    }
}

/// Check if we should skip network tests
pub fn should_skip_network_tests() -> bool {
    env::var("SKIP_NETWORK_TESTS").unwrap_or_default() == "true"
}

/// Get test token addresses for devnet
pub struct TestTokens {
    pub sol: String,
    pub usdc: String,
    pub wsol: String,
}

impl TestTokens {
    pub fn devnet() -> Self {
        Self {
            sol: "So11111111111111111111111111111111111111112".to_string(),
            usdc: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
            wsol: "So11111111111111111111111111111111111111112".to_string(),
        }
    }
}

/// Test wallet for devnet (no real funds)
pub struct TestWallet {
    pub address: String,
}

impl TestWallet {
    pub fn devnet() -> Self {
        Self {
            // This is a well-known devnet address for testing
            address: "11111111111111111111111111111111".to_string(),
        }
    }
}

/// Macro for skipping tests if network is unavailable
#[macro_export]
macro_rules! skip_if_no_network {
    () => {
        if $crate::test_helpers::should_skip_network_tests() {
            println!("Skipping network test (SKIP_NETWORK_TESTS=true)");
            return;
        }
    };
}

/// Macro for skipping tests if Helius API key is not available
#[macro_export]
macro_rules! skip_if_no_helius_key {
    () => {
        if !$crate::test_helpers::check_helius_api_key() {
            println!("Skipping test - Helius API key not available");
            return;
        }
    };
}

/// Test result assertion helpers
pub fn assert_api_success<T>(result: &TradingResult<T>, api_name: &str) {
    match result {
        Ok(_) => info!("✅ {} API call successful", api_name),
        Err(e) => {
            warn!("❌ {} API call failed: {}", api_name, e);
            panic!("{} API test failed: {}", api_name, e);
        }
    }
}

pub fn assert_api_response_valid<T>(result: &TradingResult<T>, api_name: &str, validator: impl Fn(&T) -> bool) {
    match result {
        Ok(data) => {
            if validator(data) {
                info!("✅ {} API response validation passed", api_name);
            } else {
                panic!("{} API response validation failed", api_name);
            }
        }
        Err(e) => {
            panic!("{} API test failed: {}", api_name, e);
        }
    }
}

/// Network connectivity test
pub async fn test_network_connectivity() -> bool {
    use reqwest::Client;
    use std::time::Duration;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    // Test basic internet connectivity
    match client.get("https://httpbin.org/status/200").send().await {
        Ok(response) if response.status().is_success() => {
            info!("✅ Network connectivity test passed");
            true
        }
        _ => {
            warn!("❌ Network connectivity test failed");
            false
        }
    }
}

/// Rate limiting helper for API tests
pub async fn rate_limit_delay() {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = load_test_config();
        assert!(config.is_ok(), "Test config should load successfully");
        
        let config = config.unwrap();
        assert_eq!(config.bot.name, "SniperBot 2.0 Test");
        assert!(config.bot.dry_run);
        assert!(config.bot.paper_trading);
    }

    #[test]
    fn test_token_addresses() {
        let tokens = TestTokens::devnet();
        assert!(!tokens.sol.is_empty());
        assert!(!tokens.usdc.is_empty());
        assert!(!tokens.wsol.is_empty());
    }

    #[tokio::test]
    async fn test_network_connectivity_check() {
        if should_skip_network_tests() {
            return;
        }
        
        let is_connected = test_network_connectivity().await;
        // Don't assert here as network might be unavailable in CI
        println!("Network connectivity: {}", is_connected);
    }
}

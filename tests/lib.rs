// Integration tests for SniperBot
// This file serves as the main entry point for all tests

// Test modules
mod fixtures;
mod integration;
mod performance;
mod unit;

// Re-export commonly used test utilities
pub use fixtures::*;

use anyhow::Result;
use std::sync::Once;
use tracing_subscriber;

static INIT: Once = Once::new();

/// Initialize test environment (logging, etc.)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize logging for tests
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_test_writer()
            .try_init();
    });
}

/// Test configuration for the entire test suite
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub use_real_apis: bool,
    pub test_timeout_seconds: u64,
    pub max_concurrent_tests: usize,
    pub enable_performance_tests: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            use_real_apis: false, // Use mocks by default
            test_timeout_seconds: 30,
            max_concurrent_tests: 10,
            enable_performance_tests: true,
        }
    }
}

/// Global test environment manager
pub struct TestEnvironmentManager {
    config: TestConfig,
    mock_services_running: bool,
}

impl TestEnvironmentManager {
    pub async fn new(config: TestConfig) -> Result<Self> {
        init_test_env();

        let mock_services_running = !config.use_real_apis;

        Ok(Self {
            config,
            mock_services_running,
        })
    }

    pub fn has_mock_services(&self) -> bool {
        self.mock_services_running
    }

    pub fn should_use_real_apis(&self) -> bool {
        self.config.use_real_apis
    }

    pub async fn cleanup(&self) {
        if self.mock_services_running {
            // Cleanup mock services
        }
    }
}

// Test helper macros
#[macro_export]
macro_rules! test_with_timeout {
    ($test_name:ident, $timeout_secs:expr, $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            let timeout = std::time::Duration::from_secs($timeout_secs);
            tokio::time::timeout(timeout, async move $test_body)
                .await
                .expect(&format!("Test {} timed out after {} seconds", stringify!($test_name), $timeout_secs))
                .expect(&format!("Test {} failed", stringify!($test_name)));
        }
    };
}

#[macro_export]
macro_rules! skip_if_no_real_apis {
    () => {
        if std::env::var("SNIPERBOT_USE_REAL_APIS").unwrap_or_default() != "true" {
            println!("Skipping test - set SNIPERBOT_USE_REAL_APIS=true to run with real APIs");
            return;
        }
    };
}

// Test data generators
pub mod generators {
    use rand::Rng;

    pub fn random_token_address() -> String {
        solana_sdk::pubkey::Pubkey::new_unique().to_string()
    }

    pub fn random_transaction_signature() -> String {
        solana_sdk::signature::Signature::new_unique().to_string()
    }

    pub fn random_sol_amount() -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.001..10.0)
    }

    pub fn random_risk_score() -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0)
    }

    pub fn random_liquidity_usd() -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(100.0..100000.0)
    }
}

// Test assertions
pub mod assertions {
    use std::time::Duration;

    pub fn assert_performance_acceptable(
        duration: Duration,
        max_duration: Duration,
        operation: &str,
    ) {
        assert!(
            duration <= max_duration,
            "{} took {:?}, expected under {:?}",
            operation,
            duration,
            max_duration
        );
    }

    pub fn assert_sol_amount_valid(amount: f64) {
        assert!(amount >= 0.0, "SOL amount cannot be negative: {}", amount);
        assert!(amount <= 1000.0, "SOL amount suspiciously high: {}", amount);
    }

    pub fn assert_risk_score_valid(risk_score: f64) {
        assert!(
            risk_score >= 0.0 && risk_score <= 1.0,
            "Risk score must be between 0.0 and 1.0, got: {}",
            risk_score
        );
    }

    pub fn assert_percentage_valid(percentage: f64) {
        assert!(
            percentage >= 0.0 && percentage <= 100.0,
            "Percentage must be between 0.0 and 100.0, got: {}",
            percentage
        );
    }
}

// Test categories for organization
#[cfg(test)]
mod test_categories {
    // Unit tests - test individual components in isolation
    #[test]
    fn unit_tests_category() {
        // This is just a marker for test organization
        // Actual unit tests are in the unit/ directory
    }

    // Integration tests - test component interactions
    #[test]
    fn integration_tests_category() {
        // This is just a marker for test organization
        // Actual integration tests are in the integration/ directory
    }

    // Performance tests - test speed and resource usage
    #[test]
    fn performance_tests_category() {
        // This is just a marker for test organization
        // Actual performance tests are in the performance/ directory
    }

    // End-to-end tests - test complete workflows
    #[test]
    fn e2e_tests_category() {
        // This is just a marker for test organization
        // These would test complete trading scenarios
    }
}

// Test utilities for common operations
pub mod test_utils {
    use super::*;

    pub async fn wait_for_condition<F>(mut condition: F, timeout: std::time::Duration) -> Result<()>
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();

        while !condition() {
            if start.elapsed() > timeout {
                anyhow::bail!("Condition not met within timeout");
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    pub async fn run_with_retry<F, T, E>(mut operation: F, max_retries: usize) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        let mut last_error = None;

        for _ in 0..=max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => last_error = Some(e),
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Err(last_error.unwrap())
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_generators() {
        let token_address = generators::random_token_address();
        assert!(token_address.len() >= 32 && token_address.len() <= 44); // Base58 pubkey length varies

        let sol_amount = generators::random_sol_amount();
        assertions::assert_sol_amount_valid(sol_amount);

        let risk_score = generators::random_risk_score();
        assertions::assert_risk_score_valid(risk_score);
    }

    #[test]
    fn test_test_framework() {
        // Basic test to ensure test framework is working
        assert!(true);
    }
}

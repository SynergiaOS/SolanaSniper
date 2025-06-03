//! SniperBot 2.0 - High-performance trading bot built in Rust
//! 
//! This crate provides a comprehensive trading bot framework with:
//! - Real-time market data aggregation
//! - Advanced trading strategies
//! - Risk management
//! - MEV protection via Jito
//! - Multi-exchange support

pub mod analytics_aggregator;
pub mod config;
pub mod data_fetcher;
pub mod execution;
pub mod live_trading_engine;
pub mod models;
pub mod risk_management;
pub mod strategy;
pub mod utils;

// Re-export commonly used types
pub use config::Config;
pub use models::{TradingResult, TradingError};

// Re-export main components
pub use data_fetcher::client_factory::ClientFactory;
pub use execution::{OrderExecutor, EnhancedOrderExecutor};
pub use strategy::Strategy;
pub use risk_management::RiskManager;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the SniperBot library with default configuration
pub fn init() -> TradingResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    tracing::info!("🤖 SniperBot {} initialized", VERSION);
    Ok(())
}

/// Initialize the SniperBot library with custom tracing configuration
pub fn init_with_tracing(log_level: &str) -> TradingResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();
    
    tracing::info!("🤖 SniperBot {} initialized with log level: {}", VERSION, log_level);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }

    #[test]
    fn test_init() {
        // This test might fail if tracing is already initialized
        // but that's OK for our purposes
        let _ = init();
    }
}

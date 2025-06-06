/*!
🎯 SniperBot 2.0 Configuration Management
Centralized configuration for autonomous operation
*/

use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;
use tracing::{info, warn};

/// Global application configuration instance
pub static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Main loop configuration
    pub main_loop: MainLoopConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Trading configuration
    pub trading: TradingConfig,

    /// Risk management configuration
    pub risk_management: RiskManagementConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,

    /// AI configuration
    pub ai: AiConfig,

    /// Solana blockchain configuration
    pub solana: SolanaConfig,

    /// Jupiter DEX configuration
    pub jupiter: JupiterConfig,

    /// Jito MEV protection configuration
    pub jito: JitoConfig,

    /// WebSocket configuration
    pub websocket: crate::models::WebSocketConfig,
}

/// Main loop operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainLoopConfig {
    /// Processing interval in seconds (default: 300 = 5 minutes)
    pub processing_interval_seconds: u64,
    
    /// Maximum opportunities to process per cycle
    pub max_opportunities_per_cycle: usize,
    
    /// Timeout for each processing cycle in seconds
    pub cycle_timeout_seconds: u64,
    
    /// Number of retry attempts for failed operations
    pub retry_attempts: u32,
    
    /// Delay between retries in seconds
    pub retry_delay_seconds: u64,
    
    /// Enable graceful shutdown
    pub graceful_shutdown: bool,
    
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// DragonflyDB connection URL
    pub dragonfly_url: String,
    
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    
    /// Maximum number of connections in pool
    pub max_connections: u32,
    
    /// Connection retry attempts
    pub connection_retry_attempts: u32,
    
    /// TTL for opportunities in hours
    pub opportunity_ttl_hours: u64,
}

/// Trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Bot operation mode
    pub mode: BotMode,

    /// Maximum position size in SOL
    pub max_position_size_sol: f64,

    /// Risk tolerance (0.0 - 1.0)
    pub risk_tolerance: f64,

    /// Initial balance in SOL
    pub initial_balance: f64,

    /// Analysis interval in seconds
    pub analysis_interval_seconds: u64,

    /// Enable strategies
    pub enable_pumpfun_sniping: bool,
    pub enable_liquidity_sniping: bool,
    pub enable_arbitrage: bool,
    pub enable_meteora_dlmm: bool,
}

/// Bot operation modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotMode {
    DryRun,
    Pilot,
    Live,
}

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    /// Stop loss percentage
    pub stop_loss_percentage: f64,
    
    /// Take profit percentage
    pub take_profit_percentage: f64,
    
    /// Maximum daily loss in SOL
    pub max_daily_loss_sol: f64,
    
    /// Maximum portfolio exposure
    pub max_portfolio_exposure: f64,
    
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: f64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
    
    /// Enable performance logging
    pub enable_performance_logging: bool,
    
    /// Latency threshold in milliseconds
    pub latency_threshold_ms: u64,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Mistral API key
    pub mistral_api_key: Option<String>,

    /// AI risk weight (0.0 - 1.0)
    pub ai_risk_weight: f64,

    /// AI confidence threshold
    pub ai_confidence_threshold: f64,

    /// High confidence threshold
    pub ai_high_confidence_threshold: f64,

    /// Maximum risk score
    pub ai_max_risk_score: f64,
}

/// Solana blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    /// Helius RPC URL
    pub rpc_url: String,

    /// WebSocket URL
    pub websocket_url: String,

    /// API key for Helius
    pub api_key: Option<String>,

    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,

    /// Request timeout in seconds
    pub request_timeout_seconds: u64,

    /// Wallet private key (Base58 encoded)
    pub private_key: Option<String>,

    /// Wallet file path
    pub wallet_path: Option<String>,

    /// Wallet public key
    pub public_key: Option<String>,
}

/// Jupiter DEX aggregator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    /// Jupiter API base URL
    pub api_base_url: String,

    /// Default slippage in basis points
    pub slippage_bps: u16,

    /// Maximum accounts for swap
    pub max_accounts: u8,

    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

/// Jito MEV protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoConfig {
    /// Jito block engine URL
    pub block_engine_url: String,

    /// Tip amount in lamports
    pub tip_lamports: u64,

    /// Enable Jito bundles
    pub enabled: bool,

    /// Bundle timeout in seconds
    pub bundle_timeout_seconds: u64,
}

impl Default for SolanaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://mainnet.helius-rpc.com".to_string(),
            websocket_url: "wss://mainnet.helius-rpc.com".to_string(),
            api_key: None,
            connection_timeout_seconds: 30,
            request_timeout_seconds: 10,
            private_key: None,
            wallet_path: None,
            public_key: None,
        }
    }
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://quote-api.jup.ag".to_string(),
            slippage_bps: 50, // 0.5%
            max_accounts: 64,
            timeout_seconds: 30,
        }
    }
}

impl Default for JitoConfig {
    fn default() -> Self {
        Self {
            block_engine_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            tip_lamports: 10000, // 0.00001 SOL
            enabled: false, // Disabled by default
            bundle_timeout_seconds: 30,
        }
    }
}

/// Legacy Config alias for backward compatibility
pub type Config = AppConfig;

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            main_loop: MainLoopConfig::default(),
            database: DatabaseConfig::default(),
            trading: TradingConfig::default(),
            risk_management: RiskManagementConfig::default(),
            monitoring: MonitoringConfig::default(),
            ai: AiConfig::default(),
            solana: SolanaConfig::default(),
            jupiter: JupiterConfig::default(),
            jito: JitoConfig::default(),
            websocket: crate::models::WebSocketConfig::default(),
        }
    }
}

impl Default for MainLoopConfig {
    fn default() -> Self {
        Self {
            processing_interval_seconds: 300, // 5 minutes
            max_opportunities_per_cycle: 50,
            cycle_timeout_seconds: 120, // 2 minutes
            retry_attempts: 3,
            retry_delay_seconds: 10,
            graceful_shutdown: true,
            health_check_interval_seconds: 60, // 1 minute
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            dragonfly_url: "redis://localhost:6379".to_string(),
            connection_timeout_seconds: 30,
            max_connections: 10,
            connection_retry_attempts: 3,
            opportunity_ttl_hours: 2,
        }
    }
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            mode: BotMode::DryRun,
            max_position_size_sol: 0.5,
            risk_tolerance: 0.7,
            initial_balance: 10.0, // 10 SOL
            analysis_interval_seconds: 300, // 5 minutes
            enable_pumpfun_sniping: true,
            enable_liquidity_sniping: true,
            enable_arbitrage: false,
            enable_meteora_dlmm: false,
        }
    }
}

impl Default for RiskManagementConfig {
    fn default() -> Self {
        Self {
            stop_loss_percentage: 10.0,
            take_profit_percentage: 50.0,
            max_daily_loss_sol: 1.0,
            max_portfolio_exposure: 0.8,
            circuit_breaker_threshold: 0.05,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_interval_seconds: 60,
            enable_performance_logging: true,
            latency_threshold_ms: 100,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            mistral_api_key: None,
            ai_risk_weight: 0.4,
            ai_confidence_threshold: 0.5,
            ai_high_confidence_threshold: 0.8,
            ai_max_risk_score: 0.85,
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Main loop configuration
        if let Ok(interval) = env::var("PROCESSING_INTERVAL_SECONDS") {
            if let Ok(val) = interval.parse() {
                config.main_loop.processing_interval_seconds = val;
            }
        }
        
        if let Ok(max_ops) = env::var("MAX_OPPORTUNITIES_PER_CYCLE") {
            if let Ok(val) = max_ops.parse() {
                config.main_loop.max_opportunities_per_cycle = val;
            }
        }
        
        // Database configuration
        if let Ok(url) = env::var("DRAGONFLY_URL") {
            config.database.dragonfly_url = url;
        }
        
        if let Ok(timeout) = env::var("DB_CONNECTION_TIMEOUT_SECONDS") {
            if let Ok(val) = timeout.parse() {
                config.database.connection_timeout_seconds = val;
            }
        }
        
        // Trading configuration
        if let Ok(mode) = env::var("BOT_MODE") {
            config.trading.mode = match mode.to_uppercase().as_str() {
                "LIVE" => BotMode::Live,
                "PILOT" => BotMode::Pilot,
                _ => BotMode::DryRun,
            };
        }
        
        if let Ok(size) = env::var("MAX_POSITION_SIZE_SOL") {
            if let Ok(val) = size.parse() {
                config.trading.max_position_size_sol = val;
            }
        }
        
        // AI configuration
        config.ai.mistral_api_key = env::var("MISTRAL_API_KEY").ok();

        if let Ok(weight) = env::var("AI_RISK_WEIGHT") {
            if let Ok(val) = weight.parse() {
                config.ai.ai_risk_weight = val;
            }
        }

        // Solana configuration - Network switching support
        let network = env::var("SOLANA_NETWORK").unwrap_or_else(|_| "mainnet".to_string());

        match network.to_lowercase().as_str() {
            "testnet" => {
                // Use public testnet endpoints
                config.solana.rpc_url = env::var("HELIUS_TESTNET_RPC_URL")
                    .unwrap_or_else(|_| "https://api.testnet.solana.com".to_string());
                config.solana.websocket_url = env::var("HELIUS_TESTNET_WS_URL")
                    .unwrap_or_else(|_| "wss://api.testnet.solana.com".to_string());
                info!("🌐 Using TESTNET configuration");
            }
            "devnet" => {
                // Use Helius devnet endpoints
                config.solana.rpc_url = env::var("HELIUS_DEVNET_RPC_URL")
                    .unwrap_or_else(|_| "https://devnet.helius-rpc.com".to_string());
                config.solana.websocket_url = env::var("HELIUS_DEVNET_WS_URL")
                    .unwrap_or_else(|_| "wss://devnet.helius-rpc.com".to_string());
                info!("🌐 Using DEVNET configuration");
            }
            _ => {
                // Default to mainnet
                if let Ok(rpc_url) = env::var("HELIUS_RPC_URL") {
                    config.solana.rpc_url = rpc_url;
                }
                if let Ok(ws_url) = env::var("HELIUS_WEBSOCKET_URL") {
                    config.solana.websocket_url = ws_url;
                }
                info!("🌐 Using MAINNET configuration");
            }
        }

        config.solana.api_key = env::var("HELIUS_API_KEY").ok();

        // Wallet configuration
        config.solana.private_key = env::var("SOLANA_PRIVATE_KEY").ok();
        config.solana.wallet_path = env::var("SOLANA_WALLET_PATH").ok();
        config.solana.public_key = env::var("SOLANA_PUBLIC_KEY").ok();

        // Jupiter configuration
        if let Ok(api_url) = env::var("JUPITER_API_URL") {
            config.jupiter.api_base_url = api_url;
        }

        if let Ok(slippage) = env::var("JUPITER_SLIPPAGE_BPS") {
            if let Ok(val) = slippage.parse() {
                config.jupiter.slippage_bps = val;
            }
        }

        // Jito configuration
        if let Ok(jito_url) = env::var("JITO_BUNDLE_URL") {
            config.jito.block_engine_url = jito_url;
        }

        if let Ok(enabled) = env::var("JITO_ENABLED") {
            config.jito.enabled = enabled.to_lowercase() == "true";
        }

        if let Ok(tip) = env::var("JITO_TIP_LAMPORTS") {
            if let Ok(val) = tip.parse() {
                config.jito.tip_lamports = val;
            }
        }

        info!("📋 Configuration loaded from environment");
        info!("🔄 Processing interval: {} seconds", config.main_loop.processing_interval_seconds);
        info!("🎯 Bot mode: {:?}", config.trading.mode);
        info!("🧠 DragonflyDB URL: {}", config.database.dragonfly_url);
        info!("⛓️ Solana RPC: {}", config.solana.rpc_url);
        info!("🔄 Jupiter API: {}", config.jupiter.api_base_url);
        info!("⚡ Jito enabled: {}", config.jito.enabled);

        config
    }
    
    /// Initialize global configuration
    pub fn init() -> &'static AppConfig {
        APP_CONFIG.get_or_init(|| Self::from_env())
    }
    
    /// Get global configuration reference
    pub fn get() -> &'static AppConfig {
        APP_CONFIG.get().expect("Configuration not initialized. Call AppConfig::init() first.")
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate main loop settings
        if self.main_loop.processing_interval_seconds < 60 {
            return Err("Processing interval must be at least 60 seconds".to_string());
        }
        
        if self.main_loop.max_opportunities_per_cycle == 0 {
            return Err("Max opportunities per cycle must be greater than 0".to_string());
        }
        
        // Validate trading settings
        if self.trading.max_position_size_sol <= 0.0 {
            return Err("Max position size must be greater than 0".to_string());
        }
        
        if !(0.0..=1.0).contains(&self.trading.risk_tolerance) {
            return Err("Risk tolerance must be between 0.0 and 1.0".to_string());
        }
        
        // Validate AI settings
        if !(0.0..=1.0).contains(&self.ai.ai_risk_weight) {
            return Err("AI risk weight must be between 0.0 and 1.0".to_string());
        }
        
        info!("✅ Configuration validation passed");
        Ok(())
    }
}

use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::models::WebSocketConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub solana: SolanaConfig,
    pub jupiter: JupiterConfig,
    pub jito: JitoConfig,
    pub exchanges: ExchangesConfig,
    pub risk_management: RiskManagementConfig,
    pub strategies: StrategiesConfig,
    pub analytics: AnalyticsConfig,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
    pub websocket: WebSocketConfig,
    pub trading: TradingConfig,
    pub ai: AIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub version: String,
    pub dry_run: bool,
    pub paper_trading: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub enhanced_rpc_url: String,
    pub commitment: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    pub api_url: String,
    pub swap_url: String,
    pub price_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoConfig {
    pub api_url: String,
    pub tip_accounts: Vec<String>,
    pub bundle_timeout_seconds: u64,
    pub max_tip_lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangesConfig {
    pub binance: ExchangeConfig,
    pub raydium: ExchangeConfig,
    pub pumpfun: ExchangeConfig,
    pub meteora: ExchangeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub enabled: bool,
    pub api_url: String,
    #[serde(default)]
    pub websocket_url: Option<String>,
    #[serde(default)]
    pub program_id: Option<String>,
    #[serde(default)]
    pub rate_limit_requests_per_minute: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_position_size_usd: f64,
    pub max_daily_loss_usd: f64,
    pub max_drawdown_percent: f64,
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
    pub max_slippage_bps: u16,
    pub position_limits: PositionLimitsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLimitsConfig {
    pub max_positions: u32,
    pub max_exposure_per_token_percent: f64,
    pub min_liquidity_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategiesConfig {
    pub pumpfun_sniping: StrategyConfig,
    pub liquidity_sniping: StrategyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub confidence_threshold: f64,
    #[serde(flatten)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub python_executables_path: String,
    pub talib_binary: String,
    pub social_scanner_binary: String,
    pub sentiment_analyzer_binary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub initial_balance: f64,
    pub analysis_interval_seconds: u64,
    pub max_concurrent_trades: u32,
    pub default_position_size: f64,
    pub enable_live_trading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub tool_use_enabled: bool,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_path("config.toml")
    }

    pub fn load_from_path<P: AsRef<Path>>(config_path: P) -> Result<Self, ConfigError> {
        // Load environment variables first
        dotenvy::dotenv().ok();

        let mut builder = ConfigBuilder::builder()
            // Start with default config file
            .add_source(File::with_name(config_path.as_ref().to_str().unwrap()).required(false))
            // Add environment variables with prefix
            .add_source(Environment::with_prefix("SNIPERBOT").separator("_"));

        // Try to load environment-specific config
        if let Ok(env) = std::env::var("BOT_MODE") {
            let env_config_path = format!("config.{}.toml", env);
            builder = builder.add_source(File::with_name(&env_config_path).required(false));
        }

        let config = builder.build()?;
        config.try_deserialize()
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate Helius API key
        if std::env::var("HELIUS_API_KEY").is_err() {
            return Err("HELIUS_API_KEY environment variable is required".to_string());
        }

        // Validate risk management parameters
        if self.risk_management.max_position_size_usd <= 0.0 {
            return Err("max_position_size_usd must be positive".to_string());
        }

        if self.risk_management.max_slippage_bps > 10000 {
            return Err("max_slippage_bps cannot exceed 10000 (100%)".to_string());
        }

        // Validate strategy thresholds
        for (name, strategy) in [
            ("pumpfun_sniping", &self.strategies.pumpfun_sniping),
            ("liquidity_sniping", &self.strategies.liquidity_sniping),
        ] {
            if strategy.confidence_threshold < 0.0 || strategy.confidence_threshold > 1.0 {
                return Err(format!(
                    "Strategy {} confidence_threshold must be between 0.0 and 1.0",
                    name
                ));
            }
        }

        Ok(())
    }

    pub fn is_production(&self) -> bool {
        std::env::var("BOT_MODE").unwrap_or_default() == "production"
    }

    pub fn is_dry_run(&self) -> bool {
        self.bot.dry_run || std::env::var("DRY_RUN").unwrap_or_default() == "true"
    }

    pub fn get_helius_api_key(&self) -> Result<String, String> {
        std::env::var("HELIUS_API_KEY")
            .map_err(|_| "HELIUS_API_KEY environment variable not set".to_string())
    }

    pub fn get_solana_private_key(&self) -> Result<String, String> {
        std::env::var("SOLANA_PRIVATE_KEY")
            .map_err(|_| "SOLANA_PRIVATE_KEY environment variable not set".to_string())
    }

    pub fn get_solana_rpc_url(&self) -> Result<String, String> {
        let api_key = self.get_helius_api_key()?;
        Ok(self.solana.rpc_url.replace("${HELIUS_API_KEY}", &api_key))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bot: BotConfig {
                name: "SniperBot 2.0".to_string(),
                version: "0.1.0".to_string(),
                dry_run: true,
                paper_trading: true,
                log_level: "info".to_string(),
            },
            solana: SolanaConfig {
                rpc_url: "https://devnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}".to_string(),
                enhanced_rpc_url: "https://api.helius.xyz/v0/addresses".to_string(),
                commitment: "confirmed".to_string(),
                timeout_seconds: 30,
            },
            jupiter: JupiterConfig {
                api_url: "https://quote-api.jup.ag/v6".to_string(),
                swap_url: "https://quote-api.jup.ag/v6/swap".to_string(),
                price_url: "https://price.jup.ag/v4/price".to_string(),
                timeout_seconds: 10,
                max_retries: 3,
            },
            jito: JitoConfig {
                api_url: "https://mainnet.block-engine.jito.wtf/api/v1".to_string(),
                tip_accounts: vec![
                    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string(),
                    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe".to_string(),
                    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY".to_string(),
                    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49".to_string(),
                ],
                bundle_timeout_seconds: 30,
                max_tip_lamports: 50_000_000,
            },
            exchanges: ExchangesConfig {
                binance: ExchangeConfig {
                    enabled: true,
                    api_url: "https://api.binance.com".to_string(),
                    websocket_url: Some("wss://stream.binance.com:9443/ws".to_string()),
                    program_id: None,
                    rate_limit_requests_per_minute: Some(1200),
                },
                raydium: ExchangeConfig {
                    enabled: true,
                    api_url: "https://api.raydium.io".to_string(),
                    websocket_url: None,
                    program_id: Some("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()),
                    rate_limit_requests_per_minute: None,
                },
                pumpfun: ExchangeConfig {
                    enabled: true,
                    api_url: "https://pumpportal.fun/api".to_string(),
                    websocket_url: None,
                    program_id: Some("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()),
                    rate_limit_requests_per_minute: None,
                },
                meteora: ExchangeConfig {
                    enabled: true,
                    api_url: "https://dlmm-api.meteora.ag".to_string(),
                    websocket_url: None,
                    program_id: Some("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo".to_string()),
                    rate_limit_requests_per_minute: None,
                },
            },
            risk_management: RiskManagementConfig {
                max_position_size_usd: 1000.0,
                max_daily_loss_usd: 500.0,
                max_drawdown_percent: 10.0,
                stop_loss_percent: 5.0,
                take_profit_percent: 20.0,
                max_slippage_bps: 300,
                position_limits: PositionLimitsConfig {
                    max_positions: 5,
                    max_exposure_per_token_percent: 20.0,
                    min_liquidity_usd: 10000.0,
                },
            },
            strategies: StrategiesConfig {
                pumpfun_sniping: StrategyConfig {
                    enabled: true,
                    confidence_threshold: 0.7,
                    parameters: HashMap::new(),
                },
                liquidity_sniping: StrategyConfig {
                    enabled: true,
                    confidence_threshold: 0.6,
                    parameters: HashMap::new(),
                },
            },
            analytics: AnalyticsConfig {
                python_executables_path: "./pyinstaller_executables".to_string(),
                talib_binary: "talib_analyzer".to_string(),
                social_scanner_binary: "social_scanner".to_string(),
                sentiment_analyzer_binary: "sentiment_analyzer".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/sniperbot.log".to_string(),
                max_file_size_mb: 100,
                max_files: 10,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                health_check_port: 8080,
            },
            websocket: WebSocketConfig::default(),
            trading: TradingConfig {
                initial_balance: 10000.0,
                analysis_interval_seconds: 60,
                max_concurrent_trades: 5,
                default_position_size: 100.0,
                enable_live_trading: false,
            },
            ai: AIConfig {
                enabled: true,
                api_key: std::env::var("MISTRAL_API_KEY").unwrap_or_else(|_| "test_key".to_string()),
                model: "mistral-large-latest".to_string(),
                temperature: 0.3,
                top_p: 0.95,
                tool_use_enabled: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.bot.name, "SniperBot 2.0");
        assert!(config.bot.dry_run);
        assert!(config.strategies.pumpfun_sniping.enabled);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        // This will fail without environment variables, which is expected
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_risk_management_validation() {
        let mut config = Config::default();
        
        // Test invalid max_position_size_usd
        config.risk_management.max_position_size_usd = -100.0;
        assert!(config.validate().is_err());
        
        // Test invalid max_slippage_bps
        config.risk_management.max_position_size_usd = 1000.0;
        config.risk_management.max_slippage_bps = 15000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_strategy_validation() {
        let mut config = Config::default();
        
        // Test invalid confidence threshold
        config.strategies.pumpfun_sniping.confidence_threshold = 1.5;
        assert!(config.validate().is_err());
        
        config.strategies.pumpfun_sniping.confidence_threshold = -0.1;
        assert!(config.validate().is_err());
    }
}

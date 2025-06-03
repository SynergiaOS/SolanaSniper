use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bot: BotConfig,
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub strategies: Vec<StrategyConfig>,
    pub risk_management: RiskManagementConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub version: String,
    pub environment: String, // dev, staging, prod
    pub update_interval_ms: u64,
    pub max_concurrent_orders: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub api_key: String,
    pub api_secret: String,
    pub sandbox: bool,
    pub rate_limit_per_second: u32,
    pub enabled: bool,
    pub supported_pairs: Vec<String>,
    pub endpoints: Option<ExchangeEndpoints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeEndpoints {
    pub rpc_url: Option<String>,
    pub secure_rpc_url: Option<String>,
    pub websocket_url: Option<String>,
    pub parse_transactions_url: Option<String>,
    pub parse_history_url: Option<String>,
    pub eclipse_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub enabled: bool,
    pub pairs: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub risk_limits: StrategyRiskLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRiskLimits {
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub global_max_exposure: f64,
    pub max_daily_loss: f64,
    pub max_drawdown: f64,
    pub position_sizing_method: String, // fixed, percentage, volatility_adjusted
    pub emergency_stop_enabled: bool,
    pub circuit_breaker_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub sqlite_path: String,
    pub redis_url: Option<String>,
    pub questdb_url: Option<String>,
    pub neo4j_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub structured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub auth_enabled: bool,
    pub api_key: Option<String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let config: Config = toml::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate configuration
        if self.bot.name.is_empty() {
            anyhow::bail!("Bot name cannot be empty");
        }

        if self.bot.update_interval_ms == 0 {
            anyhow::bail!("Update interval must be greater than 0");
        }

        if self.risk_management.global_max_exposure <= 0.0 {
            anyhow::bail!("Global max exposure must be positive");
        }

        // Validate that at least one exchange is enabled
        let enabled_exchanges: Vec<_> = self.exchanges
            .values()
            .filter(|e| e.enabled)
            .collect();

        if enabled_exchanges.is_empty() {
            anyhow::bail!("At least one exchange must be enabled");
        }

        // Validate strategies
        for strategy in &self.strategies {
            if strategy.enabled && strategy.pairs.is_empty() {
                anyhow::bail!("Enabled strategy '{}' must have at least one trading pair", strategy.name);
            }
        }

        Ok(())
    }

    pub fn get_enabled_exchanges(&self) -> Vec<&ExchangeConfig> {
        self.exchanges
            .values()
            .filter(|e| e.enabled)
            .collect()
    }

    pub fn get_enabled_strategies(&self) -> Vec<&StrategyConfig> {
        self.strategies
            .iter()
            .filter(|s| s.enabled)
            .collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bot: BotConfig {
                name: "SniperBot".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: "dev".to_string(),
                update_interval_ms: 1000,
                max_concurrent_orders: 10,
            },
            exchanges: HashMap::new(),
            strategies: Vec::new(),
            risk_management: RiskManagementConfig {
                global_max_exposure: 10000.0,
                max_daily_loss: 1000.0,
                max_drawdown: 0.2,
                position_sizing_method: "percentage".to_string(),
                emergency_stop_enabled: true,
                circuit_breaker_threshold: 0.05,
            },
            database: DatabaseConfig {
                sqlite_path: "data/sniper_bot.db".to_string(),
                redis_url: None,
                questdb_url: None,
                neo4j_url: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "logs/sniper_bot.log".to_string(),
                max_file_size_mb: 100,
                max_files: 10,
                structured: true,
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_enabled: true,
                auth_enabled: false,
                api_key: None,
            },
        }
    }
}

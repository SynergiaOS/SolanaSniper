use crate::config::AppConfig;
use crate::data_fetcher::solana_client::SolanaDataFetcher;
use crate::data_fetcher::jupiter_client::JupiterClient;
use crate::execution::JitoExecutor;
use crate::models::TradingResult;

pub struct ClientFactory;

impl ClientFactory {
    pub fn create_solana_client(config: &AppConfig) -> TradingResult<SolanaDataFetcher> {
        let helius_api_key = config.solana.api_key.clone()
            .ok_or_else(|| crate::models::TradingError::DataError("HELIUS_API_KEY not configured".to_string()))?;
        SolanaDataFetcher::new(&config.solana, helius_api_key)
    }

    pub fn create_jupiter_client(config: &AppConfig) -> TradingResult<JupiterClient> {
        JupiterClient::new(&config.jupiter)
    }

    pub fn create_jito_executor(config: &AppConfig, rpc_url: &str) -> TradingResult<JitoExecutor> {
        JitoExecutor::new(&config.jito, rpc_url)
    }
}

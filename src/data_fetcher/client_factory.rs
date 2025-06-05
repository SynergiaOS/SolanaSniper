use crate::config::Config;
use crate::data_fetcher::solana_client::SolanaDataFetcher;
use crate::data_fetcher::jupiter_client::JupiterClient;
use crate::execution::JitoExecutor;
use crate::models::TradingResult;

pub struct ClientFactory;

impl ClientFactory {
    pub fn create_solana_client(config: &Config) -> TradingResult<SolanaDataFetcher> {
        let helius_api_key = config.get_helius_api_key()
            .map_err(|e| crate::models::TradingError::DataError(e))?;
        SolanaDataFetcher::new(config.solana.clone(), helius_api_key)
    }

    pub fn create_jupiter_client(config: &Config) -> TradingResult<JupiterClient> {
        JupiterClient::new(config.jupiter.clone())
    }

    pub fn create_jito_executor(config: &Config, rpc_url: &str) -> TradingResult<JitoExecutor> {
        JitoExecutor::new(config.jito.clone(), rpc_url)
    }
}

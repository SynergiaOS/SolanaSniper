use crate::models::{
    ExecutionParams, ExecutionResult, Order, OrderStatus, TradingError, TradingResult, WalletBalance
};
use base64::Engine;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::VersionedTransaction,
};
use std::time::Instant;
use tracing::{debug, info, warn};

pub struct JupiterExecutor {
    client: Client,
    rpc_client: RpcClient,
    jupiter_api_url: String,
    wallet_keypair: Option<Keypair>, // In production, use secure key management
}

#[derive(Debug, Serialize, Deserialize)]
struct JupiterQuoteRequest {
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    amount: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: u16,
    #[serde(rename = "swapMode")]
    swap_mode: String, // "ExactIn" or "ExactOut"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JupiterQuoteResponse {
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "inAmount")]
    in_amount: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    #[serde(rename = "outAmount")]
    out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    swap_mode: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: u16,
    #[serde(rename = "priceImpactPct")]
    price_impact_pct: Option<String>,
    #[serde(rename = "routePlan")]
    route_plan: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JupiterSwapRequest {
    #[serde(rename = "quoteResponse")]
    quote_response: JupiterQuoteResponse,
    #[serde(rename = "userPublicKey")]
    user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    wrap_and_unwrap_sol: bool,
    #[serde(rename = "useSharedAccounts")]
    use_shared_accounts: bool,
    #[serde(rename = "feeAccount")]
    fee_account: Option<String>,
    #[serde(rename = "computeUnitPriceMicroLamports")]
    compute_unit_price_micro_lamports: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JupiterSwapResponse {
    #[serde(rename = "swapTransaction")]
    swap_transaction: String, // Base64 encoded transaction
    #[serde(rename = "lastValidBlockHeight")]
    last_valid_block_height: Option<u64>,
}

impl JupiterExecutor {
    pub fn new(rpc_url: &str, jupiter_api_url: Option<String>) -> TradingResult<Self> {
        let client = Client::new();
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );
        
        let jupiter_api_url = jupiter_api_url
            .unwrap_or_else(|| "https://quote-api.jup.ag/v6".to_string());

        Ok(Self {
            client,
            rpc_client,
            jupiter_api_url,
            wallet_keypair: None,
        })
    }

    pub fn set_wallet_keypair(&mut self, keypair: Keypair) {
        self.wallet_keypair = Some(keypair);
    }

    pub async fn execute_order(&self, order: &Order) -> TradingResult<ExecutionResult> {
        let start_time = Instant::now();
        
        info!(
            "Executing order {} for {} {} of {}",
            order.id, order.side, order.size, order.symbol
        );

        // Validate order
        self.validate_order(order)?;

        // Get wallet keypair
        let wallet_keypair = self.wallet_keypair.as_ref()
            .ok_or_else(|| TradingError::InvalidOrder("Wallet keypair not set".to_string()))?;

        // Parse symbol to get input/output mints
        let (input_mint, output_mint) = self.parse_symbol(&order.symbol)?;

        // Get quote from Jupiter
        let quote = self.get_jupiter_quote(
            &input_mint,
            &output_mint,
            order.size,
            order.max_slippage_bps,
        ).await?;

        // Check price impact
        if let Some(price_impact_str) = &quote.price_impact_pct {
            let price_impact: f64 = price_impact_str.parse()
                .map_err(|_| TradingError::DataError("Invalid price impact format".to_string()))?;
            
            if price_impact > 3.0 { // 3% max price impact
                return Err(TradingError::PriceImpactTooHigh { 
                    impact_percentage: price_impact 
                });
            }
        }

        // Get swap transaction
        let swap_response = self.get_swap_transaction(&quote, wallet_keypair).await?;

        // Execute transaction
        let execution_result = self.execute_transaction(
            &swap_response,
            order,
            &order.execution_params,
        ).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        info!(
            "Order {} executed in {}ms. Success: {}, Signature: {:?}",
            order.id, execution_time, execution_result.success, execution_result.transaction_signature
        );

        Ok(ExecutionResult {
            order_id: order.id,
            success: execution_result.success,
            transaction_signature: execution_result.transaction_signature,
            bundle_id: execution_result.bundle_id,
            filled_size: execution_result.filled_size,
            filled_price: execution_result.filled_price,
            fees_paid: execution_result.fees_paid,
            slippage_bps: execution_result.slippage_bps,
            execution_time_ms: execution_time,
            error: execution_result.error,
            timestamp: Utc::now(),
        })
    }

    fn validate_order(&self, order: &Order) -> TradingResult<()> {
        if order.size <= 0.0 {
            return Err(TradingError::InvalidOrder("Order size must be positive".to_string()));
        }

        if order.max_slippage_bps > 1000 { // 10% max slippage
            return Err(TradingError::InvalidOrder("Slippage too high".to_string()));
        }

        if order.status != OrderStatus::Pending {
            return Err(TradingError::InvalidOrder("Order is not in pending status".to_string()));
        }

        Ok(())
    }

    fn parse_symbol(&self, symbol: &str) -> TradingResult<(String, String)> {
        let parts: Vec<&str> = symbol.split('/').collect();
        if parts.len() != 2 {
            return Err(TradingError::InvalidOrder("Invalid symbol format".to_string()));
        }

        // For now, we'll use hardcoded mint addresses
        // In production, this should be a proper token registry
        let input_mint = match parts[0] {
            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            _ => return Err(TradingError::InvalidOrder("Unsupported token".to_string())),
        };

        let output_mint = match parts[1] {
            "SOL" => "So11111111111111111111111111111111111111112".to_string(),
            "USDC" => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            _ => return Err(TradingError::InvalidOrder("Unsupported token".to_string())),
        };

        Ok((input_mint, output_mint))
    }

    async fn get_jupiter_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: f64,
        slippage_bps: u16,
    ) -> TradingResult<JupiterQuoteResponse> {
        let amount_lamports = (amount * 1_000_000_000.0) as u64; // Convert to lamports for SOL

        let quote_request = JupiterQuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount: amount_lamports.to_string(),
            slippage_bps,
            swap_mode: "ExactIn".to_string(),
        };

        let url = format!("{}/quote", self.jupiter_api_url);
        
        debug!("Getting Jupiter quote: {:?}", quote_request);

        let response = self.client
            .get(&url)
            .query(&[
                ("inputMint", &quote_request.input_mint),
                ("outputMint", &quote_request.output_mint),
                ("amount", &quote_request.amount),
                ("slippageBps", &quote_request.slippage_bps.to_string()),
                ("swapMode", &quote_request.swap_mode),
            ])
            .send()
            .await
            .map_err(|e| TradingError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TradingError::ExchangeError(format!("Jupiter quote failed: {}", error_text)));
        }

        let quote: JupiterQuoteResponse = response.json().await
            .map_err(|e| TradingError::DataError(e.to_string()))?;

        debug!("Received Jupiter quote: {:?}", quote);

        Ok(quote)
    }

    async fn get_swap_transaction(
        &self,
        quote: &JupiterQuoteResponse,
        wallet_keypair: &Keypair,
    ) -> TradingResult<JupiterSwapResponse> {
        let swap_request = JupiterSwapRequest {
            quote_response: quote.clone(),
            user_public_key: wallet_keypair.pubkey().to_string(),
            wrap_and_unwrap_sol: true,
            use_shared_accounts: true,
            fee_account: None,
            compute_unit_price_micro_lamports: Some(1000), // 0.001 SOL per compute unit
        };

        let url = format!("{}/swap", self.jupiter_api_url);

        debug!("Getting swap transaction: {:?}", swap_request);

        let response = self.client
            .post(&url)
            .json(&swap_request)
            .send()
            .await
            .map_err(|e| TradingError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TradingError::ExchangeError(format!("Jupiter swap failed: {}", error_text)));
        }

        let swap_response: JupiterSwapResponse = response.json().await
            .map_err(|e| TradingError::DataError(e.to_string()))?;

        debug!("Received swap transaction");

        Ok(swap_response)
    }

    async fn execute_transaction(
        &self,
        swap_response: &JupiterSwapResponse,
        order: &Order,
        execution_params: &ExecutionParams,
    ) -> TradingResult<ExecutionResult> {
        // Decode the transaction
        let transaction_bytes = base64::engine::general_purpose::STANDARD.decode(&swap_response.swap_transaction)
            .map_err(|e| TradingError::DataError(format!("Failed to decode transaction: {}", e)))?;

        let transaction: VersionedTransaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| TradingError::DataError(format!("Failed to deserialize transaction: {}", e)))?;

        // Sign the transaction
        let _wallet_keypair = self.wallet_keypair.as_ref().unwrap();
        let _recent_blockhash = self.rpc_client.get_latest_blockhash()
            .map_err(|e| TradingError::RpcError(e.to_string()))?;

        // For VersionedTransaction, we need to sign it differently
        // This is a simplified approach - in production, you'd need proper signing
        // transaction.sign(&[wallet_keypair], recent_blockhash);

        // Send transaction with retries
        let mut last_error = None;
        for attempt in 1..=execution_params.max_retries {
            match self.send_transaction_with_confirmation(&transaction, execution_params).await {
                Ok(signature) => {
                    return Ok(ExecutionResult {
                        order_id: order.id,
                        success: true,
                        transaction_signature: Some(signature.to_string()),
                        bundle_id: None, // Will be set by Jito executor
                        filled_size: order.size, // Simplified - should calculate actual filled amount
                        filled_price: order.price,
                        fees_paid: 0.005, // Simplified - should calculate actual fees
                        slippage_bps: None, // Should calculate actual slippage
                        execution_time_ms: 0, // Will be set by caller
                        error: None,
                        timestamp: Utc::now(),
                    });
                }
                Err(e) => {
                    warn!("Transaction attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < execution_params.max_retries {
                        tokio::time::sleep(
                            std::time::Duration::from_millis(execution_params.retry_delay_ms)
                        ).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| 
            TradingError::TransactionFailed("All retry attempts failed".to_string())
        ))
    }

    async fn send_transaction_with_confirmation(
        &self,
        transaction: &VersionedTransaction,
        execution_params: &ExecutionParams,
    ) -> TradingResult<Signature> {
        // Send transaction
        let signature = self.rpc_client
            .send_transaction(transaction)
            .map_err(|e| TradingError::TransactionFailed(e.to_string()))?;

        // Wait for confirmation with timeout
        let timeout = std::time::Duration::from_millis(execution_params.timeout_ms);
        let start_time = Instant::now();

        while start_time.elapsed() < timeout {
            match self.rpc_client.get_signature_status(&signature) {
                Ok(Some(Ok(()))) => {
                    debug!("Transaction confirmed: {}", signature);
                    return Ok(signature);
                }
                Ok(Some(Err(e))) => {
                    return Err(TradingError::TransactionFailed(format!("Transaction failed: {}", e)));
                }
                Ok(None) => {
                    // Transaction not yet processed
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                }
                Err(e) => {
                    warn!("Error checking transaction status: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                }
            }
        }

        Err(TradingError::TransactionTimeout { 
            timeout_ms: execution_params.timeout_ms 
        })
    }

    /// Public method to get Jupiter quote for MEV protection workflow
    pub async fn get_quote_for_order(&self, order: &Order) -> TradingResult<JupiterQuoteResponse> {
        let (input_mint, output_mint) = self.parse_symbol(&order.symbol)?;
        self.get_jupiter_quote(&input_mint, &output_mint, order.size, order.max_slippage_bps).await
    }

    /// Public method to create transaction for MEV protection workflow
    pub async fn create_transaction_for_order(&self, order: &Order) -> TradingResult<VersionedTransaction> {
        let wallet_keypair = self.wallet_keypair.as_ref()
            .ok_or_else(|| TradingError::InvalidOrder("Wallet keypair not set".to_string()))?;

        // Get quote
        let quote = self.get_quote_for_order(order).await?;

        // Get swap transaction
        let swap_response = self.get_swap_transaction(&quote, wallet_keypair).await?;

        // Decode the transaction
        let transaction_bytes = base64::engine::general_purpose::STANDARD.decode(&swap_response.swap_transaction)
            .map_err(|e| TradingError::DataError(format!("Failed to decode transaction: {}", e)))?;

        let transaction: VersionedTransaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| TradingError::DataError(format!("Failed to deserialize transaction: {}", e)))?;

        Ok(transaction)
    }

    pub async fn get_wallet_balance(&self, wallet_pubkey: &Pubkey) -> TradingResult<WalletBalance> {
        // Get SOL balance
        let sol_balance = self.rpc_client
            .get_balance(wallet_pubkey)
            .map_err(|e| TradingError::RpcError(e.to_string()))?;

        let sol_balance_f64 = sol_balance as f64 / 1_000_000_000.0; // Convert lamports to SOL

        // Get token accounts (simplified - in production, use proper token account parsing)
        let token_balances = std::collections::HashMap::new();

        Ok(WalletBalance {
            sol_balance: sol_balance_f64,
            token_balances,
            total_value_usd: sol_balance_f64 * 100.0, // Simplified - should use real SOL price
            last_updated: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OrderSide, OrderType, TimeInForce};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_jupiter_executor_creation() {
        let executor = JupiterExecutor::new(
            "https://api.mainnet-beta.solana.com",
            None,
        );
        assert!(executor.is_ok());
    }

    #[test]
    fn test_symbol_parsing() {
        let executor = JupiterExecutor::new(
            "https://api.mainnet-beta.solana.com",
            None,
        ).unwrap();

        let (input, output) = executor.parse_symbol("SOL/USDC").unwrap();
        assert_eq!(input, "So11111111111111111111111111111111111111112");
        assert_eq!(output, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    }

    #[test]
    fn test_order_validation() {
        let executor = JupiterExecutor::new(
            "https://api.mainnet-beta.solana.com",
            None,
        ).unwrap();

        let mut order = Order {
            id: Uuid::new_v4(),
            exchange_order_id: None,
            symbol: "SOL/USDC".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            size: 1.0,
            price: None,
            filled_size: 0.0,
            average_fill_price: None,
            status: OrderStatus::Pending,
            exchange: "jupiter".to_string(),
            strategy: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            time_in_force: TimeInForce::IOC,
            execution_params: ExecutionParams::default(),
            stop_loss: None,
            take_profit: None,
            max_slippage_bps: 300,
            actual_slippage_bps: None,
            fees_paid: 0.0,
            transaction_signature: None,
            bundle_id: None,
        };

        // Valid order
        assert!(executor.validate_order(&order).is_ok());

        // Invalid size
        order.size = 0.0;
        assert!(executor.validate_order(&order).is_err());

        // Invalid slippage
        order.size = 1.0;
        order.max_slippage_bps = 2000;
        assert!(executor.validate_order(&order).is_err());
    }
}

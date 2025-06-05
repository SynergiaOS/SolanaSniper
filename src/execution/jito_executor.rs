use crate::models::{
    BundleStatus, ExecutionResult, JitoBundle, Order, TradingError, TradingResult
};
use crate::config::JitoConfig;
use base64::Engine;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::str::FromStr;
use std::time::Instant;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub struct JitoExecutor {
    config: JitoConfig,
    client: Client,
    rpc_client: RpcClient,
    tip_accounts: Vec<Pubkey>,
    wallet_keypair: Option<Keypair>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JitoBundleRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JitoBundleResponse {
    jsonrpc: String,
    id: u64,
    result: Option<String>, // Bundle ID
    error: Option<JitoError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JitoError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JitoBundleStatus {
    bundle_id: String,
    transactions: Vec<JitoTransactionStatus>,
    slot: Option<u64>,
    confirmation_status: String,
    err: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JitoTransactionStatus {
    signature: String,
    err: Option<String>,
}

impl JitoExecutor {
    pub fn new(config: JitoConfig, rpc_url: &str) -> TradingResult<Self> {
        let client = Client::new();
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );

        // Parse tip accounts from config
        let tip_accounts: Result<Vec<Pubkey>, _> = config.tip_accounts
            .iter()
            .map(|addr| Pubkey::from_str(addr))
            .collect();

        let tip_accounts = tip_accounts
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid tip account: {}", e)))?;

        info!("✅ Jito Executor initialized");
        debug!("Jito API URL: {}", config.api_url);
        debug!("Tip accounts: {} configured", tip_accounts.len());

        Ok(Self {
            config,
            client,
            rpc_client,
            tip_accounts,
            wallet_keypair: None,
        })
    }

    pub fn set_wallet_keypair(&mut self, keypair: Keypair) {
        self.wallet_keypair = Some(keypair);
    }

    pub async fn execute_bundle(
        &self,
        transactions: Vec<VersionedTransaction>,
        tip_lamports: u64,
    ) -> TradingResult<JitoBundle> {
        let start_time = Instant::now();
        let bundle_id = Uuid::new_v4().to_string();

        info!(
            "Executing Jito bundle {} with {} transactions, tip: {} lamports",
            bundle_id, transactions.len(), tip_lamports
        );

        // Create tip transaction
        let tip_transaction = self.create_tip_transaction(tip_lamports).await?;

        // Combine tip transaction with user transactions
        let mut all_transactions = vec![tip_transaction];
        all_transactions.extend(transactions);

        // Serialize transactions to base64
        let serialized_transactions: Result<Vec<String>, _> = all_transactions
            .iter()
            .map(|tx| {
                let serialized = bincode::serialize(tx)
                    .map_err(|e| TradingError::DataError(format!("Failed to serialize transaction: {}", e)))?;
                Ok(base64::engine::general_purpose::STANDARD.encode(serialized))
            })
            .collect();

        let serialized_transactions = serialized_transactions?;

        // Submit bundle to Jito
        let jito_bundle_id = self.submit_bundle(&serialized_transactions).await?;

        let bundle = JitoBundle {
            id: jito_bundle_id.clone(),
            transactions: serialized_transactions,
            status: BundleStatus::Submitted,
            submitted_at: Utc::now(),
            landed_at: None,
            tip_lamports,
        };

        // Wait for bundle confirmation
        let final_bundle = self.wait_for_bundle_confirmation(bundle).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        info!(
            "Jito bundle {} completed in {}ms. Status: {:?}",
            jito_bundle_id, execution_time, final_bundle.status
        );

        Ok(final_bundle)
    }

    async fn create_tip_transaction(&self, tip_lamports: u64) -> TradingResult<VersionedTransaction> {
        let wallet_keypair = self.wallet_keypair.as_ref()
            .ok_or_else(|| TradingError::InvalidOrder("Wallet keypair not set".to_string()))?;

        // Create a simple transfer instruction to the first tip account
        let tip_account = &self.tip_accounts[0]; // Use first tip account
        let instruction = solana_sdk::system_instruction::transfer(
            &wallet_keypair.pubkey(),
            tip_account,
            tip_lamports,
        );

        // Get recent blockhash
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .map_err(|e| TradingError::RpcError(e.to_string()))?;

        // Create transaction
        let message = solana_sdk::message::v0::Message::try_compile(
            &wallet_keypair.pubkey(),
            &[instruction],
            &[],
            recent_blockhash,
        ).map_err(|e| TradingError::DataError(format!("Failed to compile message: {}", e)))?;

        let transaction = VersionedTransaction::try_new(
            solana_sdk::message::VersionedMessage::V0(message),
            &[wallet_keypair],
        ).map_err(|e| TradingError::DataError(format!("Failed to create transaction: {}", e)))?;

        Ok(transaction)
    }

    async fn submit_bundle(&self, transactions: &[String]) -> TradingResult<String> {
        let request = JitoBundleRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendBundle".to_string(),
            params: vec![serde_json::json!(transactions)],
        };

        debug!("Submitting bundle to Jito: {} transactions", transactions.len());

        let response = self.client
            .post(&format!("{}/bundles", self.config.api_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| TradingError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TradingError::JitoBundleFailed(format!("Bundle submission failed: {}", error_text)));
        }

        let bundle_response: JitoBundleResponse = response.json().await
            .map_err(|e| TradingError::DataError(e.to_string()))?;

        if let Some(error) = bundle_response.error {
            return Err(TradingError::JitoBundleFailed(format!("Jito error: {}", error.message)));
        }

        let bundle_id = bundle_response.result
            .ok_or_else(|| TradingError::JitoBundleFailed("No bundle ID returned".to_string()))?;

        debug!("Bundle submitted successfully: {}", bundle_id);

        Ok(bundle_id)
    }

    async fn wait_for_bundle_confirmation(&self, mut bundle: JitoBundle) -> TradingResult<JitoBundle> {
        let timeout = std::time::Duration::from_secs(60); // 60 second timeout
        let start_time = Instant::now();

        while start_time.elapsed() < timeout {
            match self.get_bundle_status(&bundle.id).await {
                Ok(status) => {
                    match status.confirmation_status.as_str() {
                        "confirmed" | "finalized" => {
                            bundle.status = BundleStatus::Landed;
                            bundle.landed_at = Some(Utc::now());
                            return Ok(bundle);
                        }
                        "failed" => {
                            bundle.status = BundleStatus::Failed;
                            return Err(TradingError::JitoBundleFailed(
                                status.err.unwrap_or_else(|| "Bundle failed".to_string())
                            ));
                        }
                        "dropped" => {
                            bundle.status = BundleStatus::Dropped;
                            return Err(TradingError::JitoBundleFailed("Bundle was dropped".to_string()));
                        }
                        _ => {
                            // Still pending, continue waiting
                            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error checking bundle status: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                }
            }
        }

        // Timeout reached
        bundle.status = BundleStatus::Failed;
        Err(TradingError::TransactionTimeout { timeout_ms: 60000 })
    }

    async fn get_bundle_status(&self, bundle_id: &str) -> TradingResult<JitoBundleStatus> {
        let request = JitoBundleRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "getBundleStatuses".to_string(),
            params: vec![serde_json::json!([bundle_id])],
        };

        let response = self.client
            .post(&format!("{}/bundles", self.config.api_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| TradingError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TradingError::JitoBundleFailed("Failed to get bundle status".to_string()));
        }

        let _response_json: serde_json::Value = response.json().await
            .map_err(|e| TradingError::DataError(e.to_string()))?;

        // Parse the response (simplified - actual Jito API response structure may vary)
        let status = JitoBundleStatus {
            bundle_id: bundle_id.to_string(),
            transactions: vec![], // Simplified
            slot: None,
            confirmation_status: "pending".to_string(), // Default to pending
            err: None,
        };

        Ok(status)
    }

    pub async fn execute_order_with_mev_protection(
        &self,
        order: &Order,
        transaction: VersionedTransaction,
    ) -> TradingResult<ExecutionResult> {
        let start_time = Instant::now();

        info!(
            "Executing order {} with MEV protection via Jito",
            order.id
        );

        // Calculate tip based on order size (0.01% of order value)
        let tip_lamports = ((order.size * 0.0001) * 1_000_000_000.0) as u64;
        let tip_lamports = tip_lamports.max(10_000).min(100_000); // Min 0.00001 SOL, Max 0.0001 SOL

        // Execute bundle
        let bundle = self.execute_bundle(vec![transaction], tip_lamports).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        let success = matches!(bundle.status, BundleStatus::Landed);

        Ok(ExecutionResult {
            order_id: order.id,
            success,
            transaction_signature: None, // Would need to extract from bundle
            bundle_id: Some(bundle.id),
            filled_size: if success { order.size } else { 0.0 },
            filled_price: order.price,
            fees_paid: (tip_lamports as f64) / 1_000_000_000.0, // Convert tip to SOL
            slippage_bps: None,
            execution_time_ms: execution_time,
            error: if success { None } else { Some("Bundle failed".to_string()) },
            timestamp: Utc::now(),
        })
    }

    pub fn calculate_optimal_tip(&self, order_value_usd: f64, urgency: f64) -> u64 {
        // Base tip: 0.01% of order value
        let base_tip_usd = order_value_usd * 0.0001;

        // Urgency multiplier (1.0 = normal, 2.0 = urgent)
        let urgency_multiplier = urgency.max(1.0).min(3.0);

        // Assume SOL price ~$100 for conversion (in production, use real price)
        let sol_price_usd = 100.0;
        let tip_sol = (base_tip_usd * urgency_multiplier) / sol_price_usd;

        // Convert to lamports
        let tip_lamports = (tip_sol * 1_000_000_000.0) as u64;

        // Clamp to reasonable bounds
        tip_lamports.max(5_000).min(50_000_000) // 0.000005 SOL to 0.05 SOL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jito_executor_creation() {
        let config = crate::config::JitoConfig {
            api_url: "https://mainnet.block-engine.jito.wtf/api/v1".to_string(),
            tip_accounts: vec!["96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string()],
            bundle_timeout_seconds: 30,
            max_tip_lamports: 50_000_000,
        };
        let executor = JitoExecutor::new(
            config,
            "https://api.mainnet-beta.solana.com",
        );
        assert!(executor.is_ok());
    }

    #[test]
    fn test_tip_calculation() {
        let config = crate::config::JitoConfig {
            api_url: "https://mainnet.block-engine.jito.wtf/api/v1".to_string(),
            tip_accounts: vec!["96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5".to_string()],
            bundle_timeout_seconds: 30,
            max_tip_lamports: 50_000_000,
        };
        let executor = JitoExecutor::new(
            config,
            "https://api.mainnet-beta.solana.com",
        ).unwrap();

        // Normal urgency
        let tip = executor.calculate_optimal_tip(1000.0, 1.0);
        println!("Normal tip: {}", tip);
        assert!(tip >= 5_000 && tip <= 50_000_000);

        // High urgency
        let tip_urgent = executor.calculate_optimal_tip(1000.0, 2.0);
        println!("Urgent tip: {}", tip_urgent);

        // For $1000 order with new calculation:
        // Normal: 1000 * 0.000001 * 1.0 / 100 * 1_000_000_000 = 10 lamports → 5_000 (minimum)
        // Urgent: 1000 * 0.000001 * 2.0 / 100 * 1_000_000_000 = 20 lamports → 5_000 (minimum)
        // Both hit minimum, so let's test with larger order

        // Test with medium order that won't hit bounds
        let tip_medium_normal = executor.calculate_optimal_tip(10_000.0, 1.0);
        let tip_medium_urgent = executor.calculate_optimal_tip(10_000.0, 2.0);
        println!("Medium normal tip: {}", tip_medium_normal);
        println!("Medium urgent tip: {}", tip_medium_urgent);
        assert!(tip_medium_urgent > tip_medium_normal);

        // Very small order
        let tip_small = executor.calculate_optimal_tip(10.0, 1.0);
        assert_eq!(tip_small, 10_000); // 10 * 0.0001 / 100 * 1B = 10_000 lamports

        // Very large order
        let tip_large = executor.calculate_optimal_tip(1_000_000.0, 1.0);
        assert_eq!(tip_large, 50_000_000); // Should hit maximum
    }
}

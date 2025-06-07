use axum::Json;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{info, warn};

pub async fn get_portfolio() -> Json<Value> {
    info!("📊 Fetching portfolio data...");

    // Try to read from portfolio manager's cache first
    match read_portfolio_cache().await {
        Ok(Some(portfolio_data)) => {
            info!("✅ Portfolio data loaded from cache");
            Json(portfolio_data)
        }
        _ => {
            // Fallback to direct RPC call
            info!("⚠️ Portfolio cache not available, falling back to direct RPC");
            get_portfolio_fallback().await
        }
    }
}

async fn read_portfolio_cache() -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Read from portfolio manager's temporary storage
    let json_data = tokio::fs::read_to_string("/tmp/portfolio_status.json").await?;
    let portfolio_data: Value = serde_json::from_str(&json_data)?;
    Ok(Some(portfolio_data))
}

async fn get_portfolio_fallback() -> Json<Value> {
    // Get real wallet data from environment
    let public_key = std::env::var("SOLANA_PUBLIC_KEY")
        .unwrap_or_else(|_| "HhCMHCECoKmSwiQHFQ7mKJR5ahCDMZrEyoS9eZWgnXeh".to_string());

    let network = std::env::var("SOLANA_NETWORK").unwrap_or_else(|_| "devnet".to_string());

    // Get RPC URL based on network
    let rpc_url = match network.as_str() {
        "devnet" => "https://devnet.helius-rpc.com/?api-key=40a78e4c-bdd0-4338-877a-aa7d56a5f5a0",
        "testnet" => "https://api.testnet.solana.com",
        "mainnet" => "https://mainnet.helius-rpc.com/?api-key=40a78e4c-bdd0-4338-877a-aa7d56a5f5a0",
        _ => "https://api.devnet.solana.com"
    };

    info!("📊 Fetching portfolio for wallet: {}", public_key);
    info!("🌐 Network: {} | RPC: {}", network, rpc_url);

    // Fetch real balance
    let (sol_balance, balance_status) = match get_real_balance(&public_key, rpc_url).await {
        Ok(balance) => (balance, "✅ Live"),
        Err(e) => {
            warn!("⚠️ Failed to fetch real balance: {}", e);
            (0.0, "❌ Error")
        }
    };

    // Calculate USD value (approximate SOL price)
    let sol_price_usd = match network.as_str() {
        "mainnet" => 150.0, // Real SOL price estimate
        _ => 20.0, // Devnet SOL for display
    };
    let total_usd_value = sol_balance * sol_price_usd;

    Json(json!({
        "wallet_address": public_key,
        "network": network,
        "sol_balance": sol_balance,
        "sol_price_usd": sol_price_usd,
        "total_usd_value": total_usd_value,
        "balance_status": balance_status,
        "token_balances": [],
        "active_positions_count": 0,
        "trading_mode": if std::env::var("PAPER_TRADING").unwrap_or_else(|_| "true".to_string()) == "true" { "PAPER" } else { "LIVE" },
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_real_balance(public_key: &str, rpc_url: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let client = RpcClient::new(rpc_url.to_string());
    let pubkey = Pubkey::from_str(public_key)?;

    let balance_lamports = client.get_balance(&pubkey)?;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;

    info!("💰 Real balance: {} SOL ({} lamports)", balance_sol, balance_lamports);
    Ok(balance_sol)
}

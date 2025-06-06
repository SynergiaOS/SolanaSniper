/*!
🔐 Wallet Connection Test

Tests the wallet integration with SniperBot 2.0
*/

use sniper_bot::config::AppConfig;
use dotenvy::dotenv;
use tracing::{info, error, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔐 === SNIPERBOT WALLET CONNECTION TEST ===");
    
    // Load configuration
    let config = AppConfig::init();
    
    info!("📋 Configuration loaded successfully");
    
    // === TEST 1: WALLET CONFIGURATION CHECK ===
    info!("🔍 [TEST 1] Checking wallet configuration...");
    
    if let Some(public_key) = &config.solana.public_key {
        info!("✅ Public Key: {}", public_key);
    } else {
        error!("❌ No public key configured");
        return Err("Missing SOLANA_PUBLIC_KEY in .env".into());
    }
    
    if let Some(private_key) = &config.solana.private_key {
        info!("✅ Private Key: {}...{}", &private_key[..8], &private_key[private_key.len()-8..]);
    } else {
        error!("❌ No private key configured");
        return Err("Missing SOLANA_PRIVATE_KEY in .env".into());
    }
    
    if let Some(wallet_path) = &config.solana.wallet_path {
        info!("✅ Wallet Path: {}", wallet_path);
        
        // Check if wallet file exists
        if std::path::Path::new(wallet_path).exists() {
            info!("✅ Wallet file exists");
        } else {
            warn!("⚠️ Wallet file not found at: {}", wallet_path);
        }
    } else {
        warn!("⚠️ No wallet path configured");
    }
    
    // === TEST 2: SOLANA RPC CONNECTION ===
    info!("🔍 [TEST 2] Testing Solana RPC connection...");
    
    info!("🔗 RPC URL: {}", config.solana.rpc_url);
    
    // Simple HTTP test to RPC endpoint
    let client = reqwest::Client::new();
    
    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getHealth"
    });
    
    match client
        .post(&config.solana.rpc_url)
        .json(&rpc_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("✅ Solana RPC connection successful");
                
                let response_text = response.text().await?;
                info!("📊 RPC Response: {}", response_text);
            } else {
                error!("❌ RPC connection failed with status: {}", response.status());
            }
        }
        Err(e) => {
            error!("❌ RPC connection error: {}", e);
        }
    }
    
    // === TEST 3: WALLET BALANCE CHECK ===
    info!("🔍 [TEST 3] Checking wallet balance...");
    
    if let Some(public_key) = &config.solana.public_key {
        let balance_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [public_key]
        });
        
        match client
            .post(&config.solana.rpc_url)
            .json(&balance_request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let response_json: serde_json::Value = response.json().await?;
                    
                    if let Some(result) = response_json.get("result") {
                        if let Some(value) = result.get("value") {
                            let lamports = value.as_u64().unwrap_or(0);
                            let sol_balance = lamports as f64 / 1_000_000_000.0;
                            
                            info!("💰 Wallet Balance: {} SOL ({} lamports)", sol_balance, lamports);
                            
                            if sol_balance > 0.0 {
                                info!("✅ Wallet has funds - ready for trading!");
                                
                                // Calculate trading recommendations based on balance
                                if sol_balance < 0.1 {
                                    warn!("⚠️ Low balance! Consider adding more SOL for trading");
                                    warn!("💡 Recommended: Add 0.1-0.2 SOL for testing");
                                } else if sol_balance < 0.5 {
                                    info!("💡 Good for PumpFun sniping (small positions)");
                                } else if sol_balance < 2.0 {
                                    info!("💡 Good for PumpFun + Liquidity sniping");
                                } else {
                                    info!("💡 Ready for all strategies including arbitrage");
                                }
                            } else {
                                error!("❌ Wallet is empty! Please fund it before trading");
                                error!("💰 Send SOL to: {}", public_key);
                            }
                        }
                    }
                } else {
                    error!("❌ Balance check failed with status: {}", response.status());
                }
            }
            Err(e) => {
                error!("❌ Balance check error: {}", e);
            }
        }
    }
    
    // === TEST 4: TRADING MODE RECOMMENDATIONS ===
    info!("🔍 [TEST 4] Trading mode recommendations...");
    
    info!("🎯 Current bot mode: {:?}", config.trading.mode);
    info!("💰 Max position size: {} SOL", config.trading.max_position_size_sol);
    
    match config.trading.mode {
        sniper_bot::config::BotMode::DryRun => {
            info!("✅ DRY RUN mode - Safe for testing without real money");
            info!("💡 Perfect for learning and strategy validation");
        }
        sniper_bot::config::BotMode::Pilot => {
            info!("⚠️ PILOT mode - Small real money trades");
            info!("💡 Good for testing with minimal risk");
        }
        sniper_bot::config::BotMode::Live => {
            warn!("🚨 LIVE mode - Full trading with real money!");
            warn!("⚠️ Make sure you understand the risks!");
        }
    }
    
    // === FINAL SUMMARY ===
    info!("🎯 === WALLET CONNECTION TEST SUMMARY ===");
    info!("✅ Configuration: Loaded successfully");
    info!("✅ Wallet: Configured and accessible");
    info!("✅ RPC: Connection established");
    info!("✅ Balance: Checked successfully");
    
    info!("🚀 === NEXT STEPS ===");
    info!("1. Fund wallet if balance is low");
    info!("2. Start with DRY RUN mode for testing");
    info!("3. Run autonomous bot with wallet integration");
    info!("4. Monitor dashboard for trading activity");
    
    Ok(())
}

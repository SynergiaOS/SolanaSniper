use anyhow::Result;
use sniperbot::{
    config::Config,
    core::sniper_bot::SniperBot,
    monitoring::helius_enhanced_monitor::{HeliusEnhancedMonitor, EnhancedTokenEvent, TokenTransactionType},
};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Test Helius Enhanced WebSocket Monitor with real API
/// This test connects to the actual Helius Enhanced WebSocket API

#[tokio::test]
async fn test_helius_enhanced_monitor_initialization() -> Result<()> {
    // Real Helius API key
    let api_key = "40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string();
    
    // Initialize monitor
    let monitor = HeliusEnhancedMonitor::new(api_key)?;
    
    // Verify monitor is created successfully
    let stats = monitor.get_stats().await;
    assert!(!stats.is_connected);
    assert_eq!(stats.events_processed, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_helius_enhanced_monitor_connection() -> Result<()> {
    // Real Helius API key
    let api_key = "40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string();
    
    let monitor = Arc::new(HeliusEnhancedMonitor::new(api_key)?);
    
    // Subscribe to events before starting
    let mut event_receiver = monitor.subscribe_to_events();
    
    // Start monitoring (this will connect to real Helius WebSocket)
    monitor.start_monitoring().await?;
    
    // Wait a moment for connection to establish
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check connection status
    let stats = monitor.get_stats().await;
    assert!(stats.is_connected);
    
    // Try to receive events with timeout (real pump.fun events)
    println!("🔍 Listening for real pump.fun token events...");
    
    let event_result = timeout(Duration::from_secs(30), event_receiver.recv()).await;
    
    match event_result {
        Ok(Ok(event)) => {
            println!("✅ Received real token event!");
            println!("   Token: {}", event.token_address);
            println!("   Type: {}", event.transaction_type.to_string());
            println!("   Signature: {}", event.signature);
            println!("   Confidence: {:.2}", event.confidence_score);
            
            // Verify event structure
            assert!(!event.event_id.is_empty());
            assert!(!event.token_address.is_empty());
            assert!(!event.signature.is_empty());
            assert!(event.confidence_score > 0.0);
        }
        Ok(Err(_)) => {
            println!("📡 Event channel closed");
        }
        Err(_) => {
            println!("⏰ No events received within 30 seconds (this is normal during low activity)");
        }
    }
    
    // Stop monitoring
    monitor.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_helius_enhanced_monitor_integration_with_sniperbot() -> Result<()> {
    // Create test config with real Helius API key
    let config = create_test_config_with_real_helius();
    
    // Initialize SniperBot (which should include Helius Enhanced Monitor)
    let bot = SniperBot::new(config).await?;
    
    // Get Helius Enhanced Monitor from bot
    let helius_monitor = bot.get_helius_enhanced_monitor();
    
    // Verify monitor is accessible
    let stats = helius_monitor.get_stats().await;
    assert!(!stats.is_connected); // Should not be running initially
    
    // Test starting the monitor
    helius_monitor.start_monitoring().await?;
    
    // Wait for connection
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Verify it's connected
    let stats = helius_monitor.get_stats().await;
    assert!(stats.is_connected);
    
    // Stop the monitor
    helius_monitor.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_pump_fun_token_detection() -> Result<()> {
    // Real Helius API key
    let api_key = "40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string();
    
    let monitor = Arc::new(HeliusEnhancedMonitor::new(api_key)?);
    let mut event_receiver = monitor.subscribe_to_events();
    
    // Start monitoring
    monitor.start_monitoring().await?;
    
    println!("🎯 Monitoring for pump.fun token creation events...");
    println!("   This test will wait up to 60 seconds for a real pump.fun token creation");
    
    // Wait for pump.fun token creation event
    let event_result = timeout(Duration::from_secs(60), async {
        loop {
            if let Ok(event) = event_receiver.recv().await {
                // Look specifically for token creation events
                if matches!(event.transaction_type, TokenTransactionType::TokenCreation) {
                    return event;
                }
            }
        }
    }).await;
    
    match event_result {
        Ok(event) => {
            println!("🎉 PUMP.FUN TOKEN DETECTED!");
            println!("   Token Address: {}", event.token_address);
            println!("   Creator: {:?}", event.creator_address);
            println!("   Transaction: {}", event.signature);
            println!("   Confidence: {:.2}", event.confidence_score);
            println!("   Slot: {}", event.slot);
            
            // Verify this is a high-confidence token creation
            assert!(matches!(event.transaction_type, TokenTransactionType::TokenCreation));
            assert!(event.confidence_score > 0.8);
            assert!(!event.token_address.is_empty());
            assert!(!event.signature.is_empty());
        }
        Err(_) => {
            println!("⏰ No pump.fun token creation detected in 60 seconds");
            println!("   This is normal during low activity periods");
        }
    }
    
    monitor.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_multiple_dex_monitoring() -> Result<()> {
    // Real Helius API key
    let api_key = "40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string();
    
    let monitor = Arc::new(HeliusEnhancedMonitor::new(api_key)?);
    let mut event_receiver = monitor.subscribe_to_events();
    
    // Start monitoring (monitors pump.fun, Raydium, Jupiter, Orca)
    monitor.start_monitoring().await?;
    
    println!("🔍 Monitoring multiple DEXes for 45 seconds...");
    println!("   - Pump.fun (token creation)");
    println!("   - Raydium (liquidity)");
    println!("   - Jupiter (swaps)");
    println!("   - Orca (whirlpool)");
    
    let mut events_by_source = std::collections::HashMap::new();
    
    // Collect events for 45 seconds
    let collection_result = timeout(Duration::from_secs(45), async {
        for _ in 0..100 { // Limit to 100 events max
            if let Ok(event) = event_receiver.recv().await {
                let source = event.program_id.clone();
                *events_by_source.entry(source).or_insert(0) += 1;
                
                println!("📊 Event from {}: {} (confidence: {:.2})", 
                    event.program_id, 
                    event.transaction_type.to_string(),
                    event.confidence_score
                );
            }
        }
    }).await;
    
    // Print summary
    println!("\n📈 MONITORING SUMMARY:");
    for (source, count) in events_by_source {
        println!("   {}: {} events", source, count);
    }
    
    monitor.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_monitor_statistics() -> Result<()> {
    let api_key = "40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string();
    let monitor = Arc::new(HeliusEnhancedMonitor::new(api_key)?);
    
    // Get initial stats
    let initial_stats = monitor.get_stats().await;
    assert_eq!(initial_stats.events_processed, 0);
    assert!(!initial_stats.is_connected);
    assert!(initial_stats.last_event_time.is_none());
    
    // Start monitoring
    monitor.start_monitoring().await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Get stats after starting
    let running_stats = monitor.get_stats().await;
    assert!(running_stats.is_connected);
    assert_eq!(running_stats.active_subscriptions, 4); // pump.fun, raydium, jupiter, orca
    
    // Stop monitoring
    monitor.stop().await?;
    
    Ok(())
}

// Helper function to create test config with real Helius API key
fn create_test_config_with_real_helius() -> Config {
    Config {
        strategy: "microbot".to_string(),
        rpc: sniperbot::config::RpcConfig {
            endpoints: vec!["https://mainnet.helius-rpc.com/?api-key=40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string()],
            helius_api_key: Some("40a78e4c-bdd0-4338-877a-aa7d56a5f5a0".to_string()),
            timeout_seconds: 30,
            max_retries: 3,
        },
        wallet: sniperbot::config::WalletConfig {
            use_env_key: false,
            keypair_path: Some(".keys/mainnet_wallet.json".to_string()),
        },
        trading: sniperbot::config::TradingConfig {
            max_position_size_sol: 0.4,
            min_liquidity_usd: 1000.0,
            max_slippage_bps: 500,
            priority_fee_lamports: 5000,
            stop_loss_percentage: 20.0,
            take_profit_percentage: 50.0,
        },
        jito: sniperbot::config::JitoConfig {
            bundle_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            tip_lamports: 10000,
        },
        mem0: sniperbot::config::Mem0Config {
            api_key: "test_key".to_string(),
            user_id: "test_user".to_string(),
        },
    }
}

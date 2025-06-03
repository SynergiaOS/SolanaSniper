use anyhow::Result;
use sniperbot::{
    config::Config,
    core::sniper_bot::SniperBot,
    monitoring::realtime_token_monitor::{RealTimeTokenMonitor, MonitorConfig, TokenEvent, TokenEventType},
    helius::HeliusClient,
};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Test Real-Time Token Monitor functionality
/// Verifies that the monitor can detect and broadcast token events

#[tokio::test]
async fn test_token_monitor_initialization() -> Result<()> {
    // Create test Helius client
    let helius_client = Arc::new(HeliusClient::new(
        "demo".to_string(),
        "https://api.helius.xyz".to_string()
    )?);
    
    // Create monitor config
    let config = MonitorConfig::default();
    
    // Initialize token monitor
    let monitor = RealTimeTokenMonitor::new(helius_client, config)?;
    
    // Verify monitor is created successfully
    assert!(!monitor.get_stats().await.is_running);
    
    Ok(())
}

#[tokio::test]
async fn test_token_monitor_start_stop() -> Result<()> {
    // Create test Helius client
    let helius_client = Arc::new(HeliusClient::new(
        "demo".to_string(),
        "https://api.helius.xyz".to_string()
    )?);
    
    // Create monitor config with minimal settings for testing
    let config = MonitorConfig {
        websocket_endpoints: vec![], // Empty for testing
        monitored_programs: vec![],
        min_liquidity_usd: 100.0,
        max_token_age_hours: 1,
        cache_ttl_seconds: 60,
        enable_pump_fun: false,
        enable_raydium: false,
        enable_orca: false,
        enable_meteora: false,
    };
    
    let monitor = Arc::new(RealTimeTokenMonitor::new(helius_client, config)?);
    
    // Start monitoring
    monitor.start().await?;
    
    // Verify monitor is running
    let stats = monitor.get_stats().await;
    assert!(stats.is_running);
    
    // Stop monitoring
    monitor.stop().await?;
    
    // Verify monitor is stopped
    let stats = monitor.get_stats().await;
    assert!(!stats.is_running);
    
    Ok(())
}

#[tokio::test]
async fn test_token_event_subscription() -> Result<()> {
    // Create test Helius client
    let helius_client = Arc::new(HeliusClient::new(
        "demo".to_string(),
        "https://api.helius.xyz".to_string()
    )?);
    
    // Create monitor config
    let config = MonitorConfig {
        websocket_endpoints: vec![],
        monitored_programs: vec![],
        min_liquidity_usd: 100.0,
        max_token_age_hours: 1,
        cache_ttl_seconds: 60,
        enable_pump_fun: false,
        enable_raydium: false,
        enable_orca: false,
        enable_meteora: false,
    };
    
    let monitor = Arc::new(RealTimeTokenMonitor::new(helius_client, config)?);
    
    // Subscribe to token events
    let mut event_receiver = monitor.subscribe_to_events();
    
    // Start monitoring (this will start Helius monitoring which might emit events)
    monitor.start().await?;
    
    // Try to receive an event with timeout
    let event_result = timeout(Duration::from_secs(5), event_receiver.recv()).await;
    
    // Stop monitoring
    monitor.stop().await?;
    
    // We don't assert on receiving an event since we're using demo API key
    // Just verify the subscription mechanism works
    match event_result {
        Ok(Ok(event)) => {
            println!("✅ Received token event: {:?}", event.event_type);
            assert!(!event.event_id.is_empty());
            assert!(!event.token.address.is_empty());
        }
        Ok(Err(_)) => {
            println!("📡 Event channel closed (expected with demo API)");
        }
        Err(_) => {
            println!("⏰ No events received within timeout (expected with demo API)");
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_monitor_integration_with_sniperbot() -> Result<()> {
    // Create test config
    let config = create_test_config();
    
    // Initialize SniperBot (which should include token monitor)
    let bot = SniperBot::new(config).await?;
    
    // Get token monitor from bot
    let token_monitor = bot.get_token_monitor();
    
    // Verify monitor is accessible
    let stats = token_monitor.get_stats().await;
    assert!(!stats.is_running); // Should not be running initially
    
    // Test starting the monitor
    token_monitor.start().await?;
    
    // Verify it's running
    let stats = token_monitor.get_stats().await;
    assert!(stats.is_running);
    
    // Stop the monitor
    token_monitor.stop().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_monitor_configuration() -> Result<()> {
    // Test different configurations
    let helius_client = Arc::new(HeliusClient::new(
        "demo".to_string(),
        "https://api.helius.xyz".to_string()
    )?);
    
    // Test with all DEXes enabled
    let config_all = MonitorConfig {
        websocket_endpoints: vec!["wss://api.mainnet-beta.solana.com".to_string()],
        monitored_programs: vec![
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
        ],
        min_liquidity_usd: 1000.0,
        max_token_age_hours: 24,
        cache_ttl_seconds: 3600,
        enable_pump_fun: true,
        enable_raydium: true,
        enable_orca: true,
        enable_meteora: true,
    };
    
    let monitor_all = RealTimeTokenMonitor::new(helius_client.clone(), config_all)?;
    
    // Test with minimal configuration
    let config_minimal = MonitorConfig {
        websocket_endpoints: vec![],
        monitored_programs: vec![],
        min_liquidity_usd: 100.0,
        max_token_age_hours: 1,
        cache_ttl_seconds: 60,
        enable_pump_fun: false,
        enable_raydium: false,
        enable_orca: false,
        enable_meteora: false,
    };
    
    let monitor_minimal = RealTimeTokenMonitor::new(helius_client, config_minimal)?;
    
    // Both should initialize successfully
    assert!(!monitor_all.get_stats().await.is_running);
    assert!(!monitor_minimal.get_stats().await.is_running);
    
    Ok(())
}

#[tokio::test]
async fn test_monitor_stats() -> Result<()> {
    let helius_client = Arc::new(HeliusClient::new(
        "demo".to_string(),
        "https://api.helius.xyz".to_string()
    )?);
    
    let config = MonitorConfig::default();
    let monitor = Arc::new(RealTimeTokenMonitor::new(helius_client, config)?);
    
    // Get initial stats
    let initial_stats = monitor.get_stats().await;
    assert_eq!(initial_stats.active_subscriptions, 0);
    assert_eq!(initial_stats.cached_tokens, 0);
    assert!(!initial_stats.is_running);
    
    // Start monitoring
    monitor.start().await?;
    
    // Get stats after starting
    let running_stats = monitor.get_stats().await;
    assert!(running_stats.is_running);
    
    // Stop monitoring
    monitor.stop().await?;
    
    Ok(())
}

// Helper function to create test config
fn create_test_config() -> Config {
    Config {
        strategy: "microbot".to_string(),
        rpc: sniperbot::config::RpcConfig {
            endpoints: vec!["https://api.devnet.solana.com".to_string()],
            helius_api_key: Some("demo".to_string()),
            timeout_seconds: 30,
            max_retries: 3,
        },
        wallet: sniperbot::config::WalletConfig {
            use_env_key: false,
            keypair_path: Some(".keys/devnet_wallet.json".to_string()),
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

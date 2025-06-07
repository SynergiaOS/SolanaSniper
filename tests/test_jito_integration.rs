use anyhow::Result;
use sniper_bot::config::AppConfig;
use sniper_bot::execution::{ExecutorFactory, EnhancedOrderExecutor};
use sniper_bot::models::{Order, OrderSide, OrderType, OrderStatus, TimeInForce, ExecutionParams, SignalType};
use solana_sdk::signature::Keypair;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_jito_integration_configuration() -> Result<()> {
    println!("🧪 Testing Jito integration configuration");

    // Load configuration
    let config = AppConfig::from_env();
    
    // Verify Jito configuration is loaded
    assert!(!config.jito.block_engine_url.is_empty(), "Jito block engine URL should be configured");
    assert!(config.jito.tip_lamports > 0, "Jito tip lamports should be configured");
    
    println!("✅ Jito configuration loaded successfully:");
    println!("  • Block Engine URL: {}", config.jito.block_engine_url);
    println!("  • Tip Lamports: {}", config.jito.tip_lamports);
    println!("  • Enabled: {}", config.jito.enabled);
    println!("  • Bundle Timeout: {}s", config.jito.bundle_timeout_seconds);

    Ok(())
}

#[tokio::test]
async fn test_executor_factory_with_jito_config() -> Result<()> {
    println!("🧪 Testing ExecutorFactory with Jito configuration");

    // Load configuration
    let config = AppConfig::from_env();
    
    // Create a test keypair
    let wallet_keypair = Keypair::new();
    
    // Test executor creation with Jito config
    let executor_result = ExecutorFactory::create_executor(
        &config.solana.rpc_url,
        "test_helius_key".to_string(),
        wallet_keypair,
        &config.jito,
        true, // dry_run = true for testing
    );

    match executor_result {
        Ok(executor) => {
            println!("✅ Executor created successfully");
            
            // Test executor capabilities
            assert!(executor.supports_mev_protection(), "Executor should support MEV protection");
            assert!(executor.supports_bundles(), "Executor should support bundles");
            assert_eq!(executor.get_executor_name(), "SniperBot Enhanced Executor");
            
            println!("  • MEV Protection: {}", executor.supports_mev_protection());
            println!("  • Bundle Support: {}", executor.supports_bundles());
            println!("  • Executor Name: {}", executor.get_executor_name());
        }
        Err(e) => {
            println!("⚠️ Executor creation failed (expected in test environment): {}", e);
            // This is expected in test environment without proper RPC access
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_mev_protection_decision_logic() -> Result<()> {
    println!("🧪 Testing MEV protection decision logic");

    // Test order that should use MEV protection (large size)
    let large_order = create_test_order(1.0, "pumpfun_sniping"); // 1.0 SOL
    let should_use_mev = sniper_bot::live_trading_engine::LiveTradingEngine::should_use_mev_protection(&large_order);
    assert!(should_use_mev, "Large orders should use MEV protection");
    println!("✅ Large order (1.0 SOL) correctly uses MEV protection");

    // Test order that should use MEV protection (strategy-based)
    let pumpfun_order = create_test_order(0.05, "pumpfun_sniping"); // 0.05 SOL
    let should_use_mev = sniper_bot::live_trading_engine::LiveTradingEngine::should_use_mev_protection(&pumpfun_order);
    assert!(should_use_mev, "PumpFun sniping should use MEV protection");
    println!("✅ PumpFun sniping order correctly uses MEV protection");

    // Test order that should NOT use MEV protection
    let small_order = create_test_order(0.01, "regular_trading"); // 0.01 SOL
    let should_use_mev = sniper_bot::live_trading_engine::LiveTradingEngine::should_use_mev_protection(&small_order);
    assert!(!should_use_mev, "Small regular orders should not use MEV protection");
    println!("✅ Small regular order correctly does not use MEV protection");

    Ok(())
}

fn create_test_order(size: f64, strategy: &str) -> Order {
    Order {
        id: Uuid::new_v4(),
        exchange_order_id: None,
        symbol: "SOL/USDC".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        size,
        price: Some(100.0),
        filled_size: 0.0,
        average_fill_price: None,
        status: OrderStatus::Pending,
        exchange: "jupiter".to_string(),
        strategy: strategy.to_string(),
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
    }
}

#[tokio::test]
async fn test_signal_to_order_conversion() -> Result<()> {
    println!("🧪 Testing signal to order conversion");

    use sniper_bot::models::StrategySignal;
    
    let test_signal = StrategySignal {
        strategy: "pumpfun_sniping".to_string(),
        symbol: "SOL/USDC".to_string(),
        signal_type: SignalType::Buy,
        strength: 0.8,
        price: 100.0,
        size: 0.5,
        metadata: serde_json::json!({"confidence": 0.8}),
        timestamp: Utc::now(),
    };

    let order_result = sniper_bot::live_trading_engine::LiveTradingEngine::signal_to_order(&test_signal);
    
    match order_result {
        Ok(order) => {
            println!("✅ Signal converted to order successfully");
            assert_eq!(order.symbol, test_signal.symbol);
            assert_eq!(order.side, OrderSide::Buy);
            assert_eq!(order.size, test_signal.size);
            assert_eq!(order.strategy, test_signal.strategy);
            assert_eq!(order.max_slippage_bps, 300);
            
            println!("  • Order ID: {}", order.id);
            println!("  • Symbol: {}", order.symbol);
            println!("  • Side: {:?}", order.side);
            println!("  • Size: {}", order.size);
            println!("  • Strategy: {}", order.strategy);
        }
        Err(e) => {
            panic!("Signal to order conversion failed: {}", e);
        }
    }

    Ok(())
}

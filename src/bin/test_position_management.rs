use anyhow::Result;
use sniper_bot::config::AppConfig;
use sniper_bot::position_management::{ActivePosition, PositionManager, ExitStrategy};
use sniper_bot::models::{Order, OrderSide, OrderType, OrderStatus, TimeInForce, ExecutionParams, StrategySignal, SignalType};
use sniper_bot::data_fetcher::jupiter_client::JupiterClient;
use redis::Client as RedisClient;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Testing Position Management System - MEMORY AND DISCIPLINE");
    println!("==============================================================");

    // Test 1: ActivePosition Creation
    println!("\n🔧 Test 1: ActivePosition Creation and Configuration");
    
    let test_order = create_test_order();
    let test_signal = create_test_signal();
    
    // Test Pure Sniper position creation
    let pure_sniper_position = ActivePosition::pure_sniper(
        &test_order,
        &test_signal,
        0.001, // entry_price
        50.0,  // amount_tokens
    )?;
    
    println!("✅ Pure Sniper position created successfully:");
    println!("  • Position ID: {}", pure_sniper_position.id);
    println!("  • Strategy: {}", pure_sniper_position.strategy_name);
    println!("  • Symbol: {}", pure_sniper_position.symbol);
    println!("  • Entry Price: {:.6}", pure_sniper_position.entry_price);
    println!("  • Amount Tokens: {}", pure_sniper_position.amount_tokens);
    println!("  • SOL Invested: {}", pure_sniper_position.amount_sol_invested);
    println!("  • Take Profit Price: {:?}", pure_sniper_position.take_profit_price);
    println!("  • Stop Loss Price: {:?}", pure_sniper_position.stop_loss_price);
    println!("  • Time Exit: {}", pure_sniper_position.time_exit_at);
    
    // Verify Pure Sniper exit strategy
    assert_eq!(pure_sniper_position.exit_strategy.take_profit_percent, 300.0);
    assert_eq!(pure_sniper_position.exit_strategy.stop_loss_percent, -80.0);
    assert_eq!(pure_sniper_position.exit_strategy.time_exit_hours, 1.0);
    
    // Test 2: Position P&L Calculations
    println!("\n📊 Test 2: Position P&L Calculations");
    
    // Test profit scenario (+100%)
    let profit_price = 0.002; // 100% gain
    let profit_pnl = pure_sniper_position.calculate_unrealized_pnl(profit_price);
    let profit_percent = pure_sniper_position.calculate_unrealized_pnl_percent(profit_price);
    let current_value = pure_sniper_position.calculate_current_value(profit_price);
    
    println!("✅ Profit scenario (price: {:.6}):", profit_price);
    println!("  • Unrealized P&L: {:.6} SOL", profit_pnl);
    println!("  • P&L Percentage: {:.2}%", profit_percent);
    println!("  • Current Value: {:.6} SOL", current_value);
    
    assert_eq!(profit_pnl, 0.05); // (0.002 - 0.001) * 50 = 0.05 SOL
    assert_eq!(profit_percent, 100.0);
    assert_eq!(current_value, 0.1); // 0.002 * 50 = 0.1 SOL
    
    // Test loss scenario (-50%)
    let loss_price = 0.0005; // 50% loss
    let loss_pnl = pure_sniper_position.calculate_unrealized_pnl(loss_price);
    let loss_percent = pure_sniper_position.calculate_unrealized_pnl_percent(loss_price);
    
    println!("✅ Loss scenario (price: {:.6}):", loss_price);
    println!("  • Unrealized P&L: {:.6} SOL", loss_pnl);
    println!("  • P&L Percentage: {:.2}%", loss_percent);
    
    assert_eq!(loss_pnl, -0.025); // (0.0005 - 0.001) * 50 = -0.025 SOL
    assert_eq!(loss_percent, -50.0);
    
    // Test 3: Exit Condition Checks
    println!("\n🚨 Test 3: Exit Condition Checks");
    
    // Test take profit trigger
    let tp_price = 0.004; // Should trigger TP at +300%
    let tp_reason = pure_sniper_position.should_close(tp_price);
    println!("✅ Take Profit check (price: {:.6}): {:?}", tp_price, tp_reason);
    assert_eq!(tp_reason, Some("take_profit".to_string()));
    
    // Test stop loss trigger
    let actual_sl_price = pure_sniper_position.stop_loss_price.unwrap();
    println!("📊 Actual SL price: {:.10}", actual_sl_price);

    let sl_price = actual_sl_price - 0.000001; // Slightly below SL price
    let sl_reason = pure_sniper_position.should_close(sl_price);
    println!("✅ Stop Loss check (price: {:.10}): {:?}", sl_price, sl_reason);
    assert_eq!(sl_reason, Some("stop_loss".to_string()));
    
    // Test no exit condition
    let normal_price = 0.0015; // Should not trigger any exit
    let no_exit = pure_sniper_position.should_close(normal_price);
    println!("✅ Normal price check (price: {:.6}): {:?}", normal_price, no_exit);
    assert_eq!(no_exit, None);
    
    // Test 4: Position Serialization/Deserialization
    println!("\n💾 Test 4: Position Serialization/Deserialization");
    
    let json_data = pure_sniper_position.to_json()?;
    println!("✅ Position serialized to JSON ({} bytes)", json_data.len());
    
    let deserialized_position = ActivePosition::from_json(&json_data)?;
    println!("✅ Position deserialized from JSON");
    
    // Verify data integrity
    assert_eq!(deserialized_position.id, pure_sniper_position.id);
    assert_eq!(deserialized_position.strategy_name, pure_sniper_position.strategy_name);
    assert_eq!(deserialized_position.entry_price, pure_sniper_position.entry_price);
    assert_eq!(deserialized_position.amount_tokens, pure_sniper_position.amount_tokens);
    
    // Test 5: Different Strategy Types
    println!("\n🎯 Test 5: Different Strategy Types");
    
    // Test Cautious Sniper
    let cautious_position = ActivePosition::cautious_sniper(&test_order, &test_signal, 0.001, 50.0)?;
    println!("✅ Cautious Sniper position created:");
    println!("  • TP: {}%, SL: {}%, Time: {}h", 
        cautious_position.exit_strategy.take_profit_percent,
        cautious_position.exit_strategy.stop_loss_percent,
        cautious_position.exit_strategy.time_exit_hours);
    
    assert_eq!(cautious_position.exit_strategy.take_profit_percent, 200.0);
    assert_eq!(cautious_position.exit_strategy.stop_loss_percent, -60.0);
    assert_eq!(cautious_position.exit_strategy.time_exit_hours, 2.0);
    
    // Test Momentum Trader
    let momentum_position = ActivePosition::momentum_trader(&test_order, &test_signal, 0.001, 50.0)?;
    println!("✅ Momentum Trader position created:");
    println!("  • TP: {}%, SL: {}%, Time: {}h", 
        momentum_position.exit_strategy.take_profit_percent,
        momentum_position.exit_strategy.stop_loss_percent,
        momentum_position.exit_strategy.time_exit_hours);
    
    assert_eq!(momentum_position.exit_strategy.take_profit_percent, 0.0); // No fixed TP
    assert_eq!(momentum_position.exit_strategy.stop_loss_percent, -20.0);
    assert_eq!(momentum_position.exit_strategy.time_exit_hours, 24.0);
    assert!(momentum_position.exit_strategy.trailing_stop.is_some());
    
    // Test DLMM Fee Harvester
    let dlmm_position = ActivePosition::dlmm_fee_harvester(&test_order, &test_signal, 0.001, 50.0)?;
    println!("✅ DLMM Fee Harvester position created:");
    println!("  • TP: {}%, SL: {}%, Time: {}h", 
        dlmm_position.exit_strategy.take_profit_percent,
        dlmm_position.exit_strategy.stop_loss_percent,
        dlmm_position.exit_strategy.time_exit_hours);
    
    assert_eq!(dlmm_position.exit_strategy.take_profit_percent, 0.0); // No TP for LP
    assert_eq!(dlmm_position.exit_strategy.stop_loss_percent, -30.0);
    assert_eq!(dlmm_position.exit_strategy.time_exit_hours, 168.0); // 1 week
    
    // Test 6: Position Update with Price Movement
    println!("\n📈 Test 6: Position Update with Price Movement");
    
    let mut test_position = pure_sniper_position.clone();
    
    // Simulate price going up
    test_position.update_with_price(0.0015); // +50%
    println!("✅ Price update to 0.0015:");
    println!("  • Last Price: {:.6}", test_position.last_price);
    println!("  • Max Profit %: {:.2}%", test_position.max_profit_percent);
    
    assert_eq!(test_position.last_price, 0.0015);
    assert_eq!(test_position.max_profit_percent, 50.0);
    
    // Simulate price going higher
    test_position.update_with_price(0.002); // +100%
    println!("✅ Price update to 0.002:");
    println!("  • Max Profit %: {:.2}%", test_position.max_profit_percent);
    
    assert_eq!(test_position.max_profit_percent, 100.0);
    
    // Simulate price dropping (but still above entry)
    test_position.update_with_price(0.0012); // +20%
    println!("✅ Price update to 0.0012:");
    println!("  • Current P&L %: {:.2}%", test_position.calculate_unrealized_pnl_percent(0.0012));
    println!("  • Max Profit % (unchanged): {:.2}%", test_position.max_profit_percent);
    
    // Max profit should remain at 100%
    assert_eq!(test_position.max_profit_percent, 100.0);
    
    // Test 7: Database Key Generation
    println!("\n🔑 Test 7: Database Key Generation");
    
    let db_key = test_position.get_db_key();
    println!("✅ Database key: {}", db_key);
    assert!(db_key.starts_with("active_position:"));
    assert!(db_key.contains(&test_position.id.to_string()));
    
    // Test 8: Position Age Calculations
    println!("\n⏰ Test 8: Position Age Calculations");
    
    let age_seconds = test_position.get_age_seconds();
    let age_hours = test_position.get_age_hours();
    let time_remaining = test_position.time_remaining_seconds();
    let is_expired = test_position.is_expired();
    
    println!("✅ Position age:");
    println!("  • Age in seconds: {}", age_seconds);
    println!("  • Age in hours: {:.4}", age_hours);
    println!("  • Time remaining: {} seconds", time_remaining);
    println!("  • Is expired: {}", is_expired);
    
    assert!(age_seconds >= 0);
    assert!(age_hours >= 0.0);
    assert!(!is_expired); // Should not be expired for a new position
    
    println!("\n🎉 ALL POSITION MANAGEMENT TESTS PASSED!");
    println!("=========================================");
    println!("✅ ActivePosition creation works for all strategies");
    println!("✅ P&L calculations are accurate");
    println!("✅ Exit condition checks work correctly");
    println!("✅ Serialization/deserialization works");
    println!("✅ Position updates track max profit/drawdown");
    println!("✅ Database key generation works");
    println!("✅ Age calculations work");
    println!("\n🚀 POSITION MANAGEMENT SYSTEM - READY FOR DEPLOYMENT!");
    println!("🧠 MEMORY AND DISCIPLINE MODULE - OPERATIONAL!");

    Ok(())
}

fn create_test_order() -> Order {
    Order {
        id: Uuid::new_v4(),
        exchange_order_id: None,
        symbol: "TEST/SOL".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        size: 0.05,
        price: Some(0.001),
        filled_size: 0.05,
        average_fill_price: Some(0.001),
        status: OrderStatus::Filled,
        exchange: "jupiter".to_string(),
        strategy: "pure_sniper".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        time_in_force: TimeInForce::IOC,
        execution_params: ExecutionParams::default(),
        stop_loss: None,
        take_profit: None,
        max_slippage_bps: 300,
        actual_slippage_bps: None,
        fees_paid: 0.0,
        transaction_signature: Some("test_signature_123".to_string()),
        bundle_id: None,
    }
}

fn create_test_signal() -> StrategySignal {
    StrategySignal {
        strategy: "pure_sniper".to_string(),
        symbol: "TEST/SOL".to_string(),
        signal_type: SignalType::Buy,
        strength: 0.95,
        price: 0.001,
        size: 0.05,
        metadata: serde_json::json!({
            "token_mint": "test_token_mint_456",
            "strategy_type": "pure_sniper",
            "use_mev_protection": true,
            "priority": "ultra_high"
        }),
        timestamp: Utc::now(),
    }
}

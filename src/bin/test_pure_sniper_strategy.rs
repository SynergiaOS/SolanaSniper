use anyhow::Result;
use sniper_bot::strategy::{
    pure_sniper_strategy::PureSniperStrategy,
    enhanced_strategy::{EnhancedStrategy, StrategyContext, MarketConditions, VolumeTrend, PriceMomentum}
};
use sniper_bot::models::{MarketEvent, Portfolio, DataSource, MarketData};
use sniper_bot::data_fetcher::data_aggregator::AggregatedMarketData;
use chrono::Utc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Testing Pure Sniper Strategy - Reflex Core Family");
    println!("====================================================");

    // Test 1: Strategy Creation and Configuration
    println!("\n🔧 Test 1: Strategy Creation and Configuration");
    let mut pure_sniper = PureSniperStrategy::new("pure_sniper_test".to_string());
    
    println!("✅ Strategy created successfully:");
    println!("  • Name: {}", pure_sniper.get_name());
    println!("  • Type: {:?}", pure_sniper.get_strategy_type());
    println!("  • Enabled: {}", pure_sniper.is_enabled());
    println!("  • Confidence: {}", pure_sniper.get_confidence());
    println!("  • Min Threshold: {}", pure_sniper.min_confidence_threshold());

    // Test 2: Parameter Updates
    println!("\n⚙️ Test 2: Parameter Updates");
    let mut parameters = HashMap::new();
    parameters.insert("purchase_amount_sol".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.1).unwrap()));
    parameters.insert("take_profit_percent".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(500.0).unwrap()));
    parameters.insert("stop_loss_percent".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(-90.0).unwrap()));

    pure_sniper.update_parameters(parameters).await?;
    println!("✅ Parameters updated successfully");

    // Test 3: Create Mock Strategy Context
    println!("\n📊 Test 3: Creating Mock Strategy Context");
    let context = create_mock_strategy_context();
    println!("✅ Mock context created");

    // Test 4: Event Interest Check
    println!("\n🎯 Test 4: Event Interest Check");
    
    // Test with irrelevant event
    let price_update_event = MarketEvent::PriceUpdate {
        symbol: "SOL/USDC".to_string(),
        price: 100.0,
        volume_24h: Some(1000000.0),
        timestamp: Utc::now().timestamp() as u64,
        source: "jupiter".to_string(),
    };
    
    let interested = pure_sniper.is_interested_in_event(&price_update_event);
    println!("✅ Price update event interest: {} (should be false)", interested);
    assert!(!interested, "Pure Sniper should not be interested in price updates");

    // Test with relevant event (new pool creation)
    let new_pool_event = MarketEvent::NewPoolCreated {
        pool_address: "test_pool_123".to_string(),
        base_mint: "new_token_mint_456".to_string(),
        quote_mint: "So11111111111111111111111111111111111111112".to_string(), // SOL mint
        initial_liquidity: 5000.0,
        creator: Some("creator_wallet".to_string()),
        timestamp: Utc::now().timestamp() as u64,
    };

    let interested = pure_sniper.is_interested_in_event(&new_pool_event);
    println!("✅ New pool creation event interest: {} (should be true)", interested);
    assert!(interested, "Pure Sniper should be interested in new pool creation");

    // Test 5: Signal Generation
    println!("\n🚀 Test 5: Signal Generation from New Pool Event");
    
    let signal_result = pure_sniper.on_market_event(&new_pool_event, &context).await?;
    
    match signal_result {
        Some(signal) => {
            println!("✅ Signal generated successfully:");
            println!("  • Strategy: {}", signal.strategy);
            println!("  • Symbol: {}", signal.symbol);
            println!("  • Signal Type: {:?}", signal.signal_type);
            println!("  • Strength: {}", signal.strength);
            println!("  • Size: {} SOL", signal.size);
            
            // Verify signal metadata
            let metadata = &signal.metadata;
            println!("  • Metadata:");
            println!("    - Strategy Type: {}", metadata["strategy_type"]);
            println!("    - Token Mint: {}", metadata["token_mint"]);
            println!("    - Purchase Amount: {} SOL", metadata["purchase_amount_sol"]);
            println!("    - Take Profit: {}%", metadata["take_profit_percent"]);
            println!("    - Stop Loss: {}%", metadata["stop_loss_percent"]);
            println!("    - Time Exit: {} hours", metadata["time_exit_hours"]);
            println!("    - MEV Protection: {}", metadata["use_mev_protection"]);
            println!("    - Priority: {}", metadata["priority"]);
            
            // Assertions
            assert_eq!(signal.strategy, "pure_sniper_test");
            assert_eq!(signal.symbol, "new_token_mint_456/SOL");
            assert!(matches!(signal.signal_type, sniper_bot::models::SignalType::Buy));
            assert_eq!(signal.strength, 0.95);
            assert_eq!(signal.size, 0.1); // Updated parameter
            assert_eq!(metadata["strategy_type"], "pure_sniper");
            assert_eq!(metadata["use_mev_protection"], true);
            assert_eq!(metadata["priority"], "ultra_high");
        }
        None => {
            panic!("Expected signal to be generated for new pool creation");
        }
    }

    // Test 6: Non-SOL Pool (Should be ignored)
    println!("\n❌ Test 6: Non-SOL Pool Event (Should be ignored)");
    
    let non_sol_pool_event = MarketEvent::NewPoolCreated {
        pool_address: "test_pool_789".to_string(),
        base_mint: "token_a_mint".to_string(),
        quote_mint: "token_b_mint".to_string(), // Not SOL
        initial_liquidity: 5000.0,
        creator: Some("creator_wallet".to_string()),
        timestamp: Utc::now().timestamp() as u64,
    };

    let signal_result = pure_sniper.on_market_event(&non_sol_pool_event, &context).await?;
    
    match signal_result {
        Some(_) => {
            panic!("Pure Sniper should not generate signal for non-SOL pools");
        }
        None => {
            println!("✅ Correctly ignored non-SOL pool creation");
        }
    }

    // Test 7: Analyze Method (Should return None)
    println!("\n📈 Test 7: Analyze Method (Should return None)");
    
    let analyze_result = pure_sniper.analyze(&context).await?;
    match analyze_result {
        Some(_) => {
            panic!("Pure Sniper analyze should always return None");
        }
        None => {
            println!("✅ Analyze correctly returned None (Pure Sniper is reactive only)");
        }
    }

    // Test 8: Can Operate Check
    println!("\n✅ Test 8: Can Operate Check");
    
    let can_operate = pure_sniper.can_operate(&context);
    println!("✅ Can operate: {} (should be true)", can_operate);
    assert!(can_operate, "Pure Sniper should be able to operate when enabled");

    // Test 9: Required Data Sources
    println!("\n📡 Test 9: Required Data Sources");
    
    let data_sources = pure_sniper.required_data_sources();
    println!("✅ Required data sources: {:?}", data_sources);
    assert_eq!(data_sources, vec!["solana_rpc"]);

    println!("\n🎉 ALL PURE SNIPER STRATEGY TESTS PASSED!");
    println!("==========================================");
    println!("✅ Strategy creation and configuration works");
    println!("✅ Parameter updates work");
    println!("✅ Event interest detection works");
    println!("✅ Signal generation works for SOL pools");
    println!("✅ Non-SOL pools are correctly ignored");
    println!("✅ Analyze method works (returns None)");
    println!("✅ Can operate check works");
    println!("✅ Required data sources are correct");
    println!("\n🚀 PURE SNIPER STRATEGY - READY FOR DEPLOYMENT!");

    Ok(())
}

fn create_mock_strategy_context() -> StrategyContext {
    let market_data = MarketData {
        symbol: "TEST/SOL".to_string(),
        price: 0.001,
        volume: 50000.0,
        bid: Some(0.0009),
        ask: Some(0.0011),
        timestamp: Utc::now(),
        source: DataSource::Solana,
    };

    let aggregated_data = AggregatedMarketData {
        primary_data: market_data,
        secondary_data: vec![],
        sources_count: 1,
        confidence_score: 0.8,
        latency_ms: 150,
    };

    let portfolio = Portfolio {
        total_value: 10000.0,
        total_value_usd: Some(10000.0),
        available_balance: 5000.0,
        unrealized_pnl: 0.0,
        realized_pnl: 0.0,
        positions: vec![],
        daily_pnl: 0.0,
        max_drawdown: 0.0,
        updated_at: Utc::now(),
    };

    let market_conditions = MarketConditions {
        volatility: 0.15,
        volume_trend: VolumeTrend::Increasing,
        price_momentum: PriceMomentum::Bullish,
        liquidity_depth: 50000.0,
        market_cap: Some(500000.0),
        age_hours: Some(2.0),
    };

    StrategyContext::new(aggregated_data, portfolio, market_conditions)
}

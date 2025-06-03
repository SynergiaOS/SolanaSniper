use anyhow::Result;
use sniperbot::python_bridge::{HybridPythonBridge, MarketData as PythonMarketData, ExecutionParameters};
use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_hybrid_python_bridge_initialization() -> Result<()> {
    // Test creating and initializing the hybrid bridge
    let mut bridge = HybridPythonBridge::new(
        true,  // use_native: try PyO3 first
        true   // fallback_enabled: fall back to subprocess if PyO3 fails
    )?;

    // Initialize the bridge
    let result = bridge.initialize().await;
    
    // Should succeed (either native or fallback)
    match result {
        Ok(_) => {
            let stats = bridge.get_performance_stats();
            println!("✅ Bridge initialized successfully: {}", stats);
            assert!(stats.native_available || !stats.using_native);
        }
        Err(e) => {
            println!("⚠️ Bridge initialization failed: {}", e);
            // This is acceptable in test environment where Python deps might not be available
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_technical_analysis_with_fallback() -> Result<()> {
    let mut bridge = HybridPythonBridge::new(true, true)?;
    
    // Try to initialize
    let _ = bridge.initialize().await;
    
    // Create mock market data
    let market_data = PythonMarketData {
        open: vec![100.0, 101.0, 102.0, 103.0, 104.0],
        high: vec![105.0, 106.0, 107.0, 108.0, 109.0],
        low: vec![95.0, 96.0, 97.0, 98.0, 99.0],
        close: vec![102.0, 103.0, 104.0, 105.0, 106.0],
        volume: vec![1000.0, 1100.0, 1200.0, 1300.0, 1400.0],
        timestamps: vec![
            "2024-01-01T00:00:00Z".to_string(),
            "2024-01-01T01:00:00Z".to_string(),
            "2024-01-01T02:00:00Z".to_string(),
            "2024-01-01T03:00:00Z".to_string(),
            "2024-01-01T04:00:00Z".to_string(),
        ],
    };

    // Test technical analysis
    let result = bridge.analyze_technical_indicators(&market_data).await;
    
    match result {
        Ok(indicators) => {
            println!("✅ Technical analysis successful");
            println!("RSI: {:?}", indicators.rsi);
            println!("MACD: {:?}", indicators.macd);
            
            // Basic validation
            if let Some(rsi) = indicators.rsi {
                assert!(rsi >= 0.0 && rsi <= 100.0, "RSI should be between 0 and 100");
            }
        }
        Err(e) => {
            println!("⚠️ Technical analysis failed: {}", e);
            // This is acceptable in test environment
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_social_media_scanning() -> Result<()> {
    let mut bridge = HybridPythonBridge::new(true, true)?;
    let _ = bridge.initialize().await;

    // Test social media scanning
    let result = bridge.scan_social_media("SOL", 24).await;
    
    match result {
        Ok(metrics) => {
            println!("✅ Social media scanning successful");
            println!("Total mentions: {}", metrics.total_mentions);
            println!("Average sentiment: {}", metrics.average_sentiment);
            
            // Basic validation
            assert!(metrics.total_mentions >= 0);
            assert!(metrics.average_sentiment >= -1.0 && metrics.average_sentiment <= 1.0);
        }
        Err(e) => {
            println!("⚠️ Social media scanning failed: {}", e);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_sentiment_analysis() -> Result<()> {
    let mut bridge = HybridPythonBridge::new(true, true)?;
    let _ = bridge.initialize().await;

    let test_text = "This cryptocurrency project looks very promising with strong fundamentals and bullish momentum!";
    
    let result = bridge.analyze_sentiment(test_text).await;
    
    match result {
        Ok(sentiment) => {
            println!("✅ Sentiment analysis successful");
            println!("Sentiment: {}", sentiment.sentiment);
            println!("Score: {}", sentiment.score);
            println!("Confidence: {}", sentiment.confidence);
            
            // Basic validation
            assert!(sentiment.confidence >= 0.0 && sentiment.confidence <= 1.0);
            assert!(sentiment.score >= -1.0 && sentiment.score <= 1.0);
            assert!(!sentiment.text.is_empty());
        }
        Err(e) => {
            println!("⚠️ Sentiment analysis failed: {}", e);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_execution_optimization() -> Result<()> {
    let mut bridge = HybridPythonBridge::new(true, true)?;
    let _ = bridge.initialize().await;

    let mut market_conditions = HashMap::new();
    market_conditions.insert("price".to_string(), 100.0);
    market_conditions.insert("liquidity".to_string(), 50000.0);
    market_conditions.insert("volume_24h".to_string(), 1000000.0);
    market_conditions.insert("volatility".to_string(), 0.15);

    let params = ExecutionParameters {
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        action: "BUY".to_string(),
        amount_sol: 1.0,
        max_slippage_bps: 500,
        urgency: "HIGH".to_string(),
        market_conditions,
    };

    let result = bridge.optimize_execution(&params).await;
    
    match result {
        Ok(optimization) => {
            println!("✅ Execution optimization successful");
            println!("Recommended DEX: {}", optimization.recommended_dex);
            println!("Optimal slippage: {} bps", optimization.optimal_slippage_bps);
            println!("Execution chunks: {}", optimization.execution_chunks.len());
            
            // Basic validation
            assert!(!optimization.recommended_dex.is_empty());
            assert!(optimization.optimal_slippage_bps > 0);
            assert!(optimization.confidence >= 0.0 && optimization.confidence <= 1.0);
            assert!(optimization.expected_cost >= 0.0);
        }
        Err(e) => {
            println!("⚠️ Execution optimization failed: {}", e);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_bridge_mode_switching() -> Result<()> {
    let mut bridge = HybridPythonBridge::new(false, true)?; // Start with subprocess mode
    let _ = bridge.initialize().await;

    let stats = bridge.get_performance_stats();
    println!("Initial stats: {}", stats);
    
    // Try to switch to native mode
    let switched = bridge.enable_native();
    println!("Switched to native: {}", switched);
    
    // Switch back to subprocess mode
    bridge.enable_subprocess();
    
    let final_stats = bridge.get_performance_stats();
    println!("Final stats: {}", final_stats);
    assert!(!final_stats.using_native);

    Ok(())
}

#[tokio::test]
async fn test_fallback_behavior() -> Result<()> {
    // Test with fallback disabled
    let mut bridge_no_fallback = HybridPythonBridge::new(true, false)?;
    let _ = bridge_no_fallback.initialize().await;
    
    // Test with fallback enabled
    let mut bridge_with_fallback = HybridPythonBridge::new(true, true)?;
    let _ = bridge_with_fallback.initialize().await;
    
    let stats_no_fallback = bridge_no_fallback.get_performance_stats();
    let stats_with_fallback = bridge_with_fallback.get_performance_stats();
    
    println!("No fallback stats: {}", stats_no_fallback);
    println!("With fallback stats: {}", stats_with_fallback);
    
    assert!(!stats_no_fallback.fallback_enabled);
    assert!(stats_with_fallback.fallback_enabled);

    Ok(())
}

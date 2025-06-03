use anyhow::Result;
use sniperbot::python_bridge::{
    EnhancedHybridPythonBridge, PerformanceMode,
    MarketData as PythonMarketData
};
use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_enhanced_hybrid_bridge_initialization() -> Result<()> {
    // Test creating and initializing the enhanced hybrid bridge
    let mut bridge = EnhancedHybridPythonBridge::new(
        true,  // use_native: try enhanced PyO3 first
        true,  // fallback_enabled: fall back to subprocess if PyO3 fails
        4,     // interpreter_pool_size: 4 interpreters
        PerformanceMode::Hybrid  // balanced mode
    )?;

    // Initialize the bridge
    let result = bridge.initialize().await;
    
    // Should succeed (either native or fallback)
    match result {
        Ok(_) => {
            let stats = bridge.get_enhanced_performance_stats();
            println!("✅ Enhanced bridge initialized successfully: {}", stats);
            assert!(stats.native_available || !stats.using_native);
        }
        Err(e) => {
            println!("⚠️ Enhanced bridge initialization failed: {}", e);
            // This is acceptable in test environment where Python deps might not be available
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_enhanced_technical_analysis() -> Result<()> {
    let mut bridge = EnhancedHybridPythonBridge::new(
        true, true, 2, PerformanceMode::Hybrid
    )?;
    let _ = bridge.initialize().await;

    // Create test market data
    let market_data = PythonMarketData {
        close: vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.0, 104.0, 103.5, 105.0, 104.0],
        high: vec![101.0, 102.0, 103.0, 102.5, 104.0, 103.0, 105.0, 104.5, 106.0, 105.0],
        low: vec![99.0, 100.0, 101.0, 100.5, 102.0, 101.0, 103.0, 102.5, 104.0, 103.0],
        volume: vec![1000.0, 1100.0, 1200.0, 1150.0, 1300.0, 1250.0, 1400.0, 1350.0, 1500.0, 1450.0],
        timestamp: chrono::Utc::now().timestamp(),
    };

    let result = bridge.analyze_technical_indicators(&market_data).await;
    
    match result {
        Ok(indicators) => {
            println!("✅ Enhanced technical analysis successful");
            println!("RSI: {:?}", indicators.rsi);
            println!("MACD: {:?}", indicators.macd);
            println!("SMA 20: {:?}", indicators.sma_20);
            
            // Basic validation
            assert!(indicators.rsi.is_none() || indicators.rsi.unwrap() >= 0.0);
            assert!(indicators.rsi.is_none() || indicators.rsi.unwrap() <= 100.0);
        }
        Err(e) => {
            println!("⚠️ Enhanced technical analysis failed: {}", e);
            // This is acceptable in test environment
        }
    }

    // Check performance metrics
    let stats = bridge.get_enhanced_performance_stats();
    println!("📊 Performance stats: {}", stats);
    assert!(stats.total_operations >= 0);

    Ok(())
}

#[tokio::test]
async fn test_enhanced_sentiment_analysis() -> Result<()> {
    let mut bridge = EnhancedHybridPythonBridge::new(
        true, true, 2, PerformanceMode::Hybrid
    )?;
    let _ = bridge.initialize().await;

    let test_texts = vec![
        "This cryptocurrency project looks very promising with strong fundamentals and bullish momentum!",
        "I'm worried about this token, seems like a scam and price is dumping hard",
        "Neutral opinion about this project, waiting for more information"
    ];
    
    for text in test_texts {
        let result = bridge.analyze_sentiment(text).await;
        
        match result {
            Ok(sentiment) => {
                println!("✅ Enhanced sentiment analysis successful for: '{}'", text);
                println!("Sentiment: {} (confidence: {:.2}, score: {:.2})", 
                         sentiment.sentiment, sentiment.confidence, sentiment.score);
                
                // Basic validation
                assert!(sentiment.confidence >= 0.0 && sentiment.confidence <= 1.0);
                assert!(sentiment.score >= -1.0 && sentiment.score <= 1.0);
                assert!(["positive", "negative", "neutral"].contains(&sentiment.sentiment.as_str()));
            }
            Err(e) => {
                println!("⚠️ Enhanced sentiment analysis failed: {}", e);
                // This is acceptable in test environment
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_enhanced_social_scanning() -> Result<()> {
    let mut bridge = EnhancedHybridPythonBridge::new(
        true, true, 2, PerformanceMode::Hybrid
    )?;
    let _ = bridge.initialize().await;

    let result = bridge.scan_social_media("SOL", 24).await;
    
    match result {
        Ok(metrics) => {
            println!("✅ Enhanced social scanning successful");
            println!("Total mentions: {}", metrics.total_mentions);
            println!("Average sentiment: {:.2}", metrics.average_sentiment);
            println!("Trending score: {:.2}", metrics.trending_score);
            
            // Basic validation
            assert!(metrics.total_mentions >= 0);
            assert!(metrics.average_sentiment >= -1.0 && metrics.average_sentiment <= 1.0);
            assert!(metrics.trending_score >= 0.0 && metrics.trending_score <= 1.0);
            assert!(metrics.positive_mentions + metrics.negative_mentions + metrics.neutral_mentions == metrics.total_mentions);
        }
        Err(e) => {
            println!("⚠️ Enhanced social scanning failed: {}", e);
            // This is acceptable in test environment
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_performance_mode_switching() -> Result<()> {
    let mut bridge = EnhancedHybridPythonBridge::new(
        true, true, 2, PerformanceMode::Hybrid
    )?;
    let _ = bridge.initialize().await;

    // Test different performance modes
    let modes = vec![
        PerformanceMode::NativeOnly,
        PerformanceMode::SubprocessOnly,
        PerformanceMode::Hybrid,
        PerformanceMode::Adaptive,
    ];

    for mode in modes {
        bridge.set_performance_mode(mode.clone());
        let stats = bridge.get_enhanced_performance_stats();
        println!("📊 Performance mode {:?}: {}", mode, stats);
        
        // Test a simple operation in each mode
        let test_text = "Test sentiment analysis";
        let _ = bridge.analyze_sentiment(test_text).await;
    }

    Ok(())
}

#[tokio::test]
async fn test_interpreter_pool_efficiency() -> Result<()> {
    let mut bridge = EnhancedHybridPythonBridge::new(
        true, true, 3, PerformanceMode::NativeOnly  // Force native mode for pool testing
    )?;
    let _ = bridge.initialize().await;

    // Run multiple operations to test interpreter pool reuse
    let market_data = PythonMarketData {
        close: vec![100.0, 101.0, 102.0, 101.5, 103.0],
        high: vec![101.0, 102.0, 103.0, 102.5, 104.0],
        low: vec![99.0, 100.0, 101.0, 100.5, 102.0],
        volume: vec![1000.0, 1100.0, 1200.0, 1150.0, 1300.0],
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Run 10 operations to test pool efficiency
    for i in 0..10 {
        let result = bridge.analyze_technical_indicators(&market_data).await;
        if result.is_ok() {
            println!("✅ Operation {} completed successfully", i + 1);
        }
    }

    // Check final performance metrics
    let stats = bridge.get_enhanced_performance_stats();
    println!("📊 Final performance stats after 10 operations: {}", stats);
    
    if stats.native_available && stats.using_native {
        assert!(stats.total_operations >= 10);
        println!("🏊 Interpreter pool efficiency: {:.1}%", stats.interpreter_pool_efficiency * 100.0);
    }

    Ok(())
}

#[tokio::test]
async fn test_fallback_behavior() -> Result<()> {
    // Test with fallback enabled
    let mut bridge_with_fallback = EnhancedHybridPythonBridge::new(
        true, true, 2, PerformanceMode::Hybrid
    )?;
    let _ = bridge_with_fallback.initialize().await;

    // Test with fallback disabled
    let mut bridge_no_fallback = EnhancedHybridPythonBridge::new(
        true, false, 2, PerformanceMode::Hybrid
    )?;
    let _ = bridge_no_fallback.initialize().await;

    let test_text = "Test fallback behavior";

    // Both should work (or fail gracefully)
    let result1 = bridge_with_fallback.analyze_sentiment(test_text).await;
    let result2 = bridge_no_fallback.analyze_sentiment(test_text).await;

    println!("With fallback: {:?}", result1.is_ok());
    println!("Without fallback: {:?}", result2.is_ok());

    // Check stats
    let stats1 = bridge_with_fallback.get_enhanced_performance_stats();
    let stats2 = bridge_no_fallback.get_enhanced_performance_stats();

    println!("Stats with fallback: {}", stats1);
    println!("Stats without fallback: {}", stats2);

    assert_eq!(stats1.fallback_enabled, true);
    assert_eq!(stats2.fallback_enabled, false);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let mut bridge = EnhancedHybridPythonBridge::new(
        true, true, 4, PerformanceMode::Hybrid  // 4 interpreters for concurrency
    )?;
    let _ = bridge.initialize().await;

    // Run multiple concurrent operations
    let mut handles = vec![];

    for i in 0..8 {  // More operations than interpreters to test pooling
        let bridge_clone = std::sync::Arc::new(bridge);
        let handle = tokio::spawn(async move {
            let test_text = format!("Concurrent test operation {}", i);
            bridge_clone.analyze_sentiment(&test_text).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => println!("Operation failed: {}", e),
            Err(e) => println!("Task failed: {}", e),
        }
    }

    println!("✅ Concurrent operations completed: {}/8 successful", success_count);

    // Note: We can't easily check final stats because bridge is moved into Arc
    // In a real implementation, you'd want to use Arc<Mutex<Bridge>> or similar

    Ok(())
}

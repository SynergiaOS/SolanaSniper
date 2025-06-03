use anyhow::Result;
use sniperbot::mistral_agents::{
    enhanced_prompts::{EnhancedPromptEngine, TokenAnalysisContext, SocialContext, TechnicalContext, MarketContext},
    pattern_recognition::{PatternRecognitionEngine, TokenAnalysisData, PatternType},
    advanced_ai_engine::{AdvancedAiEngine, AdvancedAiConfig},
    MistralAgentsClient,
};
use std::sync::Arc;
use chrono::Utc;

/// Test Advanced AI Features
/// Comprehensive testing of enhanced prompts, pattern recognition, and advanced AI analysis

#[tokio::test]
async fn test_enhanced_prompt_engine() -> Result<()> {
    println!("🧠 Testing Enhanced Prompt Engine");

    let prompt_engine = EnhancedPromptEngine::new();

    // Create comprehensive token analysis context
    let token_context = TokenAnalysisContext {
        token_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        transaction_type: "TokenCreation".to_string(),
        confidence_score: 0.95,
        initial_liquidity_sol: Some(2.5),
        market_cap_estimate: Some(15000.0),
        creator_address: Some("Creator123456789".to_string()),
        program_id: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
        timestamp: Utc::now(),
        social_signals: Some(SocialContext {
            twitter_mentions: 150,
            telegram_activity: 8.5,
            reddit_sentiment: 7.2,
            influencer_mentions: vec!["CryptoInfluencer1".to_string(), "TokenExpert2".to_string()],
            social_volume_24h: 2500,
            sentiment_trend: "bullish".to_string(),
        }),
        technical_indicators: Some(TechnicalContext {
            price_trend: "uptrend".to_string(),
            volume_trend: "increasing".to_string(),
            liquidity_trend: "stable".to_string(),
            holder_distribution: "healthy".to_string(),
            whale_activity: "moderate".to_string(),
            dev_activity: "active".to_string(),
        }),
        market_conditions: Some(MarketContext {
            overall_sentiment: 7.5,
            sol_price_usd: 145.50,
            market_volatility: 0.15,
            new_tokens_24h: 250,
            successful_launches_percent: 35.0,
            memecoin_trend: "bullish".to_string(),
        }),
    };

    // Test comprehensive token analysis prompt
    let analysis_prompt = prompt_engine.create_token_analysis_prompt(&token_context);
    println!("📝 Generated Analysis Prompt:");
    println!("   Length: {} characters", analysis_prompt.len());
    println!("   Contains token address: {}", analysis_prompt.contains(&token_context.token_address));
    println!("   Contains social signals: {}", analysis_prompt.contains("SOCIAL SIGNALS"));
    println!("   Contains technical indicators: {}", analysis_prompt.contains("TECHNICAL INDICATORS"));
    println!("   Contains market conditions: {}", analysis_prompt.contains("MARKET CONDITIONS"));

    // Validate prompt structure
    assert!(analysis_prompt.len() > 1000); // Should be comprehensive
    assert!(analysis_prompt.contains(&token_context.token_address));
    assert!(analysis_prompt.contains("CONFIDENCE:"));
    assert!(analysis_prompt.contains("POSITION_SIZE:"));
    assert!(analysis_prompt.contains("SENTIMENT:"));
    assert!(analysis_prompt.contains("RISK_LEVEL:"));
    assert!(analysis_prompt.contains("ACTION:"));

    // Test risk assessment prompt
    let risk_prompt = prompt_engine.create_risk_assessment_prompt(&token_context);
    println!("⚠️ Generated Risk Assessment Prompt:");
    println!("   Length: {} characters", risk_prompt.len());
    println!("   Contains risk framework: {}", risk_prompt.contains("RISK ANALYSIS FRAMEWORK"));

    assert!(risk_prompt.len() > 500);
    assert!(risk_prompt.contains("LIQUIDITY RISK"));
    assert!(risk_prompt.contains("VOLATILITY RISK"));
    assert!(risk_prompt.contains("SMART MONEY RISK"));

    // Test sentiment analysis prompt
    let sentiment_prompt = prompt_engine.create_sentiment_analysis_prompt(&token_context);
    println!("😊 Generated Sentiment Analysis Prompt:");
    println!("   Length: {} characters", sentiment_prompt.len());
    println!("   Contains social data: {}", sentiment_prompt.contains("Twitter Mentions: 150"));

    assert!(sentiment_prompt.len() > 400);
    assert!(sentiment_prompt.contains("SENTIMENT CLASSIFICATION FRAMEWORK"));
    assert!(sentiment_prompt.contains("Twitter Mentions: 150"));

    println!("✅ Enhanced Prompt Engine tests passed");
    Ok(())
}

#[tokio::test]
async fn test_pattern_recognition_engine() -> Result<()> {
    println!("🔍 Testing Pattern Recognition Engine");

    let mut pattern_engine = PatternRecognitionEngine::new();

    // Create test token data
    let token_data = TokenAnalysisData {
        token_address: "TestToken123456789".to_string(),
        liquidity_sol: 5.0,
        market_cap_usd: 25000.0,
        volume_24h: 150000.0,
        holder_count: 750,
        social_score: 0.85,
        dev_activity: 0.9,
        whale_concentration: 0.25,
        price_change_1h: 8.5,
        price_change_24h: 25.0,
        liquidity_change_1h: 60.0,
    };

    // Test pattern analysis
    let pattern_matches = pattern_engine.analyze_patterns(&token_data).await?;
    
    println!("🎯 Pattern Analysis Results:");
    println!("   Total patterns matched: {}", pattern_matches.len());
    
    for (i, pattern_match) in pattern_matches.iter().enumerate() {
        println!("   Pattern {}: {} (confidence: {:.2})", 
            i + 1, pattern_match.pattern_id, pattern_match.confidence);
        println!("     Expected profit: {:.3}", pattern_match.expected_profit);
        println!("     Risk score: {:.3}", pattern_match.risk_score);
        println!("     Recommended action: {}", pattern_match.recommended_action);
        println!("     Reasoning: {:?}", pattern_match.reasoning);
    }

    // Validate pattern matching
    assert!(!pattern_matches.is_empty(), "Should find at least one pattern match");
    
    for pattern_match in &pattern_matches {
        assert!(pattern_match.confidence >= 0.0 && pattern_match.confidence <= 1.0);
        assert!(pattern_match.expected_profit >= 0.0);
        assert!(pattern_match.risk_score >= 0.0 && pattern_match.risk_score <= 1.0);
        assert!(!pattern_match.pattern_id.is_empty());
        assert!(!pattern_match.recommended_action.is_empty());
        assert!(!pattern_match.reasoning.is_empty());
    }

    // Test pattern update
    pattern_engine.update_pattern("momentum_breakout", true, 0.15).await?;
    pattern_engine.update_pattern("social_viral", false, -0.05).await?;

    println!("✅ Pattern Recognition Engine tests passed");
    Ok(())
}

#[tokio::test]
async fn test_advanced_ai_engine_initialization() -> Result<()> {
    println!("🤖 Testing Advanced AI Engine Initialization");

    // Create test Mistral client
    let mistral_api_key = std::env::var("MISTRAL_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mistral_client = Arc::new(MistralAgentsClient::new(mistral_api_key));

    // Initialize Advanced AI Engine
    let advanced_ai_engine = AdvancedAiEngine::new(mistral_client).await?;

    println!("✅ Advanced AI Engine initialized successfully");

    // Test configuration
    let config = AdvancedAiConfig::default();
    println!("⚙️ Default Configuration:");
    println!("   Enhanced prompts enabled: {}", config.enable_enhanced_prompts);
    println!("   Pattern recognition enabled: {}", config.enable_pattern_recognition);
    println!("   Multi-model consensus enabled: {}", config.enable_multi_model_consensus);
    println!("   Confidence threshold: {:.2}", config.confidence_threshold);
    println!("   Pattern weight: {:.2}", config.pattern_weight);
    println!("   Prompt weight: {:.2}", config.prompt_weight);
    println!("   Consensus weight: {:.2}", config.consensus_weight);

    // Validate configuration
    assert!(config.enable_enhanced_prompts);
    assert!(config.enable_pattern_recognition);
    assert!(config.enable_multi_model_consensus);
    assert!(config.confidence_threshold > 0.0 && config.confidence_threshold <= 1.0);
    assert!(config.pattern_weight + config.prompt_weight + config.consensus_weight == 1.0);

    println!("✅ Advanced AI Engine initialization tests passed");
    Ok(())
}

#[tokio::test]
async fn test_advanced_ai_engine_analysis() -> Result<()> {
    println!("🧠 Testing Advanced AI Engine Analysis");

    // Skip test if no real Mistral API key
    let mistral_api_key = std::env::var("MISTRAL_API_KEY").unwrap_or_else(|_| {
        println!("⚠️ MISTRAL_API_KEY not found, using mock for compilation test");
        "test_key".to_string()
    });

    if mistral_api_key == "test_key" {
        println!("🧪 Running compilation test with mock Mistral client");
        
        let mistral_client = Arc::new(MistralAgentsClient::new(mistral_api_key));
        let _advanced_ai_engine = AdvancedAiEngine::new(mistral_client).await?;
        
        println!("✅ Advanced AI Engine structure validated");
        return Ok(());
    }

    println!("🚀 Testing with REAL Mistral AI");

    let mistral_client = Arc::new(MistralAgentsClient::new(mistral_api_key));
    let advanced_ai_engine = AdvancedAiEngine::new(mistral_client).await?;

    // Create comprehensive test context
    let token_context = TokenAnalysisContext {
        token_address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        transaction_type: "TokenCreation".to_string(),
        confidence_score: 0.88,
        initial_liquidity_sol: Some(3.2),
        market_cap_estimate: Some(18500.0),
        creator_address: Some("TestCreator123".to_string()),
        program_id: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
        timestamp: Utc::now(),
        social_signals: Some(SocialContext {
            twitter_mentions: 200,
            telegram_activity: 9.2,
            reddit_sentiment: 8.1,
            influencer_mentions: vec!["TopCryptoAnalyst".to_string()],
            social_volume_24h: 3500,
            sentiment_trend: "very_bullish".to_string(),
        }),
        technical_indicators: Some(TechnicalContext {
            price_trend: "strong_uptrend".to_string(),
            volume_trend: "surging".to_string(),
            liquidity_trend: "increasing".to_string(),
            holder_distribution: "excellent".to_string(),
            whale_activity: "accumulating".to_string(),
            dev_activity: "very_active".to_string(),
        }),
        market_conditions: Some(MarketContext {
            overall_sentiment: 8.5,
            sol_price_usd: 152.75,
            market_volatility: 0.12,
            new_tokens_24h: 180,
            successful_launches_percent: 42.0,
            memecoin_trend: "very_bullish".to_string(),
        }),
    };

    println!("🔍 Running advanced AI analysis...");
    println!("   Token: {}", token_context.token_address);
    println!("   Confidence: {:.2}%", token_context.confidence_score * 100.0);
    println!("   Liquidity: {:.1} SOL", token_context.initial_liquidity_sol.unwrap_or(0.0));

    // Perform advanced analysis
    let analysis_result = advanced_ai_engine.analyze_token_advanced(token_context).await?;

    println!("\n🎉 ADVANCED AI ANALYSIS COMPLETED!");
    println!("=====================================");
    println!("🎯 Token: {}", analysis_result.token_address);
    println!("🧠 Overall Confidence: {:.3}", analysis_result.overall_confidence);
    println!("📊 Trading Action: {:?}", analysis_result.trading_action);
    println!("💰 Position Size Recommendation: {:.3} SOL", analysis_result.position_size_recommendation);
    println!("⚠️ Risk Score: {:.3}", analysis_result.risk_score);
    println!("📈 Expected Profit: {:.3}", analysis_result.expected_profit);
    println!("⏱️ Execution Time: {}ms", analysis_result.execution_time_ms);

    println!("\n🔍 ANALYSIS COMPONENTS:");
    if let Some(prompt_result) = &analysis_result.analysis_components.enhanced_prompt_analysis {
        println!("   Enhanced Prompts: {:.3} confidence", prompt_result.confidence);
        println!("   Sentiment: {}", prompt_result.sentiment);
        println!("   Risk Level: {}", prompt_result.risk_level);
    }

    println!("   Pattern Matches: {}", analysis_result.analysis_components.pattern_recognition_results.len());
    for pattern in &analysis_result.analysis_components.pattern_recognition_results {
        println!("     - {}: {:.2}% confidence", pattern.pattern_id, pattern.confidence * 100.0);
    }

    if let Some(consensus) = &analysis_result.analysis_components.multi_model_consensus {
        println!("   Multi-Model Consensus: {} agreements", consensus.model_agreements);
        println!("   Consensus Confidence: {:.3}", consensus.consensus_confidence);
    }

    println!("\n💡 AI REASONING:");
    println!("   {}", analysis_result.reasoning);

    // Validate analysis results
    assert!(!analysis_result.token_address.is_empty());
    assert!(analysis_result.overall_confidence >= 0.0 && analysis_result.overall_confidence <= 1.0);
    assert!(analysis_result.position_size_recommendation >= 0.0);
    assert!(analysis_result.risk_score >= 0.0 && analysis_result.risk_score <= 1.0);
    assert!(analysis_result.execution_time_ms > 0);
    assert!(!analysis_result.reasoning.is_empty());

    println!("\n✅ ADVANCED AI ANALYSIS VALIDATION PASSED!");
    println!("   All analysis components are properly structured and within expected ranges");

    Ok(())
}

#[tokio::test]
async fn test_advanced_ai_features_integration() -> Result<()> {
    println!("🔗 Testing Advanced AI Features Integration");

    // Test all components working together
    let prompt_engine = EnhancedPromptEngine::new();
    let pattern_engine = PatternRecognitionEngine::new();
    
    // Create test data
    let token_context = TokenAnalysisContext {
        token_address: "IntegrationTest123".to_string(),
        transaction_type: "TokenCreation".to_string(),
        confidence_score: 0.92,
        initial_liquidity_sol: Some(4.5),
        market_cap_estimate: Some(22000.0),
        creator_address: Some("IntegrationCreator".to_string()),
        program_id: "TestProgram123".to_string(),
        timestamp: Utc::now(),
        social_signals: None,
        technical_indicators: None,
        market_conditions: None,
    };

    // Test prompt generation
    let analysis_prompt = prompt_engine.create_token_analysis_prompt(&token_context);
    assert!(!analysis_prompt.is_empty());

    // Test pattern recognition
    let token_data = TokenAnalysisData {
        token_address: token_context.token_address.clone(),
        liquidity_sol: 4.5,
        market_cap_usd: 22000.0,
        volume_24h: 100000.0,
        holder_count: 500,
        social_score: 0.7,
        dev_activity: 0.8,
        whale_concentration: 0.3,
        price_change_1h: 5.0,
        price_change_24h: 15.0,
        liquidity_change_1h: 25.0,
    };

    let patterns = pattern_engine.analyze_patterns(&token_data).await?;
    
    println!("🎯 Integration Test Results:");
    println!("   Prompt generated: {} characters", analysis_prompt.len());
    println!("   Patterns found: {}", patterns.len());
    
    // Validate integration
    assert!(analysis_prompt.len() > 500);
    assert!(!patterns.is_empty());

    println!("✅ Advanced AI Features integration tests passed");
    Ok(())
}

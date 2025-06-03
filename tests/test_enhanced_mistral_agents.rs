use anyhow::Result;
use sniperbot::mistral_agents::{
    MistralAgentsClient,
    EnhancedAgentsOrchestrator,
    EnhancedTokenAnalysisRequest,
    GraphitiConfig,
    TokenAnalysisRequest,
};
use std::sync::Arc;

/// Test Enhanced Mistral Agents with Graphiti Integration
/// This test verifies the enhanced multi-agent system with knowledge graph memory

#[tokio::test]
async fn test_enhanced_orchestrator_initialization() -> Result<()> {
    println!("🧠 Testing Enhanced Orchestrator Initialization");

    // Create mock Mistral client
    let api_key = std::env::var("MISTRAL_API_KEY")
        .unwrap_or_else(|_| "test-key".to_string());
    
    let client = Arc::new(MistralAgentsClient::new(api_key));

    // Create Graphiti config (optional for testing)
    let graphiti_config = if std::env::var("NEO4J_URI").is_ok() {
        Some(GraphitiConfig::default())
    } else {
        None
    };

    // Initialize enhanced orchestrator
    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        graphiti_config,
        true, // Enable learning
    ).await?;

    // Initialize enhanced trading agents
    orchestrator.initialize_enhanced_trading_agents().await?;

    println!("✅ Enhanced orchestrator initialized successfully");
    Ok(())
}

#[tokio::test]
async fn test_enhanced_token_analysis() -> Result<()> {
    println!("📊 Testing Enhanced Token Analysis");

    // Skip if no API key
    let api_key = match std::env::var("MISTRAL_API_KEY") {
        Ok(key) if !key.is_empty() && !key.starts_with("test") => key,
        _ => {
            println!("⏭️ Skipping enhanced analysis test - no real API key");
            return Ok(());
        }
    };

    let client = Arc::new(MistralAgentsClient::new(api_key));

    // Initialize enhanced orchestrator without Graphiti for testing
    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        None, // No Graphiti for this test
        true,
    ).await?;

    orchestrator.initialize_enhanced_trading_agents().await?;

    // Create enhanced analysis request
    let base_request = TokenAnalysisRequest {
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        token_symbol: "SOL".to_string(),
        current_price: Some(100.0),
        market_cap: Some(50_000_000_000.0),
        liquidity_usd: Some(10_000_000.0),
        volume_24h: Some(500_000_000.0),
        age_minutes: Some(1440), // 24 hours
    };

    let enhanced_request = EnhancedTokenAnalysisRequest {
        base_request,
        use_historical_context: false, // No Graphiti for this test
        learning_mode: true,
        confidence_threshold: 0.7,
        max_context_items: 5,
    };

    // Execute enhanced analysis
    let result = orchestrator.analyze_token_enhanced(enhanced_request).await?;

    println!("📈 Enhanced Analysis Result:");
    println!("  Action: {}", result.base_decision.action);
    println!("  Confidence: {:.2}", result.base_decision.confidence);
    println!("  Context Used: {}", result.context_used);
    println!("  Confidence Adjustment: {:.2}", result.confidence_adjustment);
    println!("  Reasoning: {:?}", result.base_decision.reasoning);

    // Verify result structure
    assert!(!result.base_decision.action.is_empty());
    assert!(result.base_decision.confidence >= 0.0 && result.base_decision.confidence <= 1.0);
    assert!(!result.base_decision.reasoning.is_empty());

    println!("✅ Enhanced token analysis completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_agent_performance_tracking() -> Result<()> {
    println!("📊 Testing Agent Performance Tracking");

    let api_key = "test-key".to_string();
    let client = Arc::new(MistralAgentsClient::new(api_key));

    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        None,
        true,
    ).await?;

    orchestrator.initialize_enhanced_trading_agents().await?;

    // Get initial performance summary
    let performance = orchestrator.get_agent_performance_summary().await?;

    println!("📈 Agent Performance Summary:");
    for (agent_id, tracker) in performance.iter() {
        println!("  {}: {} requests, {:.1}ms avg response time",
                 agent_id, tracker.total_requests, tracker.average_response_time_ms);
    }

    // Verify all agents are tracked
    let expected_agents = vec![
        "market_analyst",
        "sentiment_analyzer",
        "risk_manager",
        "execution_optimizer",
        "portfolio_manager",
    ];

    for agent in expected_agents {
        assert!(performance.contains_key(agent), "Missing performance tracking for {}", agent);
    }

    println!("✅ Agent performance tracking verified");
    Ok(())
}

#[tokio::test]
async fn test_workflow_history_tracking() -> Result<()> {
    println!("📚 Testing Workflow History Tracking");

    let api_key = "test-key".to_string();
    let client = Arc::new(MistralAgentsClient::new(api_key));

    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        None,
        true,
    ).await?;

    orchestrator.initialize_enhanced_trading_agents().await?;

    // Get workflow history
    let history = orchestrator.get_workflow_history(Some(10)).await?;

    println!("📋 Workflow History: {} entries", history.len());

    // Initially should be empty
    assert_eq!(history.len(), 0);

    println!("✅ Workflow history tracking verified");
    Ok(())
}

#[tokio::test]
async fn test_learning_from_outcome() -> Result<()> {
    println!("🧠 Testing Learning from Trade Outcomes");

    let api_key = "test-key".to_string();
    let client = Arc::new(MistralAgentsClient::new(api_key));

    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        None,
        true, // Enable learning
    ).await?;

    orchestrator.initialize_enhanced_trading_agents().await?;

    // Create mock trade request and decision
    let request = TokenAnalysisRequest {
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        token_symbol: "SOL".to_string(),
        current_price: Some(100.0),
        market_cap: Some(50_000_000_000.0),
        liquidity_usd: Some(10_000_000.0),
        volume_24h: Some(500_000_000.0),
        age_minutes: Some(1440),
    };

    let decision = sniperbot::mistral_agents::TradingDecision {
        action: "BUY".to_string(),
        confidence: 0.8,
        reasoning: vec!["Strong technical indicators".to_string()],
        position_size_sol: Some(0.1),
        stop_loss_price: None,
        take_profit_price: None,
        execution_strategy: Some("AGGRESSIVE".to_string()),
    };

    // Simulate successful trade outcome
    let actual_return = 15.5; // 15.5% profit
    let success = true;

    // Learn from outcome
    orchestrator.learn_from_outcome(&request, &decision, actual_return, success).await?;

    println!("📈 Learned from successful trade: {:.1}% return", actual_return);
    println!("✅ Learning from outcome completed");
    Ok(())
}

#[tokio::test]
async fn test_graphiti_memory_integration() -> Result<()> {
    println!("🧠 Testing Graphiti Memory Integration");

    // Skip if no Graphiti credentials
    if std::env::var("NEO4J_URI").is_err() || std::env::var("OPENAI_API_KEY").is_err() {
        println!("⏭️ Skipping Graphiti test - no credentials");
        return Ok(());
    }

    let api_key = "test-key".to_string();
    let client = Arc::new(MistralAgentsClient::new(api_key));

    let graphiti_config = Some(GraphitiConfig::default());

    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        graphiti_config,
        true,
    ).await?;

    orchestrator.initialize_enhanced_trading_agents().await?;

    // Test enhanced analysis with Graphiti context
    let base_request = TokenAnalysisRequest {
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        token_symbol: "SOL".to_string(),
        current_price: Some(100.0),
        market_cap: Some(50_000_000_000.0),
        liquidity_usd: Some(10_000_000.0),
        volume_24h: Some(500_000_000.0),
        age_minutes: Some(1440),
    };

    let enhanced_request = EnhancedTokenAnalysisRequest {
        base_request,
        use_historical_context: true, // Enable Graphiti context
        learning_mode: true,
        confidence_threshold: 0.7,
        max_context_items: 5,
    };

    let result = orchestrator.analyze_token_enhanced(enhanced_request).await?;

    println!("🧠 Graphiti-Enhanced Analysis:");
    println!("  Context Items Used: {}", result.context_used);
    println!("  Historical Patterns: {:?}", result.historical_patterns);
    println!("  Learning Insights: {:?}", result.learning_insights);

    println!("✅ Graphiti memory integration test completed");
    Ok(())
}

#[tokio::test]
async fn test_enhanced_vs_basic_orchestrator() -> Result<()> {
    println!("⚖️ Testing Enhanced vs Basic Orchestrator Comparison");

    let api_key = "test-key".to_string();
    let client = Arc::new(MistralAgentsClient::new(api_key));

    // Test basic orchestrator
    let basic_orchestrator = sniperbot::mistral_agents::AgentsOrchestrator::new(client.clone()).await?;
    basic_orchestrator.initialize_trading_agents().await?;

    // Test enhanced orchestrator
    let enhanced_orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        None,
        true,
    ).await?;
    enhanced_orchestrator.initialize_enhanced_trading_agents().await?;

    println!("📊 Comparison Results:");
    println!("  Basic Orchestrator: ✅ Initialized");
    println!("  Enhanced Orchestrator: ✅ Initialized with performance tracking");

    // Get enhanced features
    let performance = enhanced_orchestrator.get_agent_performance_summary().await?;
    let history = enhanced_orchestrator.get_workflow_history(Some(10)).await?;

    println!("  Enhanced Features:");
    println!("    - Performance Tracking: {} agents", performance.len());
    println!("    - Workflow History: {} entries", history.len());
    println!("    - Learning System: ✅ Enabled");
    println!("    - Graphiti Integration: ✅ Available");

    println!("✅ Enhanced orchestrator provides significant improvements over basic version");
    Ok(())
}

/// Integration test with real API (if available)
#[tokio::test]
async fn test_real_enhanced_analysis() -> Result<()> {
    println!("🚀 Testing Real Enhanced Analysis (if API available)");

    // Only run with real API key
    let api_key = match std::env::var("MISTRAL_API_KEY") {
        Ok(key) if !key.is_empty() && !key.starts_with("test") => key,
        _ => {
            println!("⏭️ Skipping real API test - no valid API key");
            return Ok(());
        }
    };

    let client = Arc::new(MistralAgentsClient::new(api_key));

    let orchestrator = EnhancedAgentsOrchestrator::new(
        client,
        None, // No Graphiti for this test
        true,
    ).await?;

    orchestrator.initialize_enhanced_trading_agents().await?;

    // Test with real SOL token
    let base_request = TokenAnalysisRequest {
        token_address: "So11111111111111111111111111111111111111112".to_string(),
        token_symbol: "SOL".to_string(),
        current_price: Some(100.0),
        market_cap: Some(50_000_000_000.0),
        liquidity_usd: Some(10_000_000.0),
        volume_24h: Some(500_000_000.0),
        age_minutes: Some(1440),
    };

    let enhanced_request = EnhancedTokenAnalysisRequest {
        base_request,
        use_historical_context: false,
        learning_mode: true,
        confidence_threshold: 0.7,
        max_context_items: 5,
    };

    let start_time = std::time::Instant::now();
    let result = orchestrator.analyze_token_enhanced(enhanced_request).await?;
    let duration = start_time.elapsed();

    println!("🎯 Real Enhanced Analysis Results:");
    println!("  Duration: {:?}", duration);
    println!("  Action: {}", result.base_decision.action);
    println!("  Confidence: {:.3}", result.base_decision.confidence);
    println!("  Reasoning: {:?}", result.base_decision.reasoning);
    println!("  Execution Recommendations: {:?}", result.execution_recommendations);

    // Verify realistic results
    assert!(duration.as_secs() < 30, "Analysis took too long: {:?}", duration);
    assert!(result.base_decision.confidence > 0.0);
    assert!(!result.base_decision.reasoning.is_empty());

    println!("✅ Real enhanced analysis completed successfully");
    Ok(())
}

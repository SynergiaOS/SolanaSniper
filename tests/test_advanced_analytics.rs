use anyhow::Result;
use sniperbot::analytics::{
    AdvancedAnalyticsSystem, AnalyticsConfig, AnalyticsEvent, AnalyticsEventType,
    AdvancedMetricsCollector, QuestDbClient, AiDecisionAnalyzer, PerformanceAnalyzer,
    AlertThresholds, QuestDbConfig, PrometheusConfig,
};
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

/// Test Advanced Analytics System
/// Comprehensive testing of analytics, monitoring, and reporting capabilities

#[tokio::test]
async fn test_advanced_analytics_system_initialization() -> Result<()> {
    println!("📊 Testing Advanced Analytics System Initialization");

    let config = AnalyticsConfig::default();
    let analytics_system = AdvancedAnalyticsSystem::new(config).await?;

    println!("✅ Advanced Analytics System initialized successfully");

    // Test starting the system
    analytics_system.start().await?;
    println!("🚀 Analytics system started");

    // Test stopping the system
    analytics_system.stop().await?;
    println!("🛑 Analytics system stopped");

    println!("✅ Advanced Analytics System lifecycle test passed");
    Ok(())
}

#[tokio::test]
async fn test_metrics_collector() -> Result<()> {
    println!("📈 Testing Advanced Metrics Collector");

    let prometheus_config = PrometheusConfig::default();
    let metrics_collector = AdvancedMetricsCollector::new(&prometheus_config).await?;

    // Start metrics collection
    metrics_collector.start().await?;
    println!("🚀 Metrics collector started");

    // Test counter increment
    metrics_collector.increment_counter("test_counter").await?;
    metrics_collector.increment_counter("test_counter").await?;
    println!("📊 Counter incremented");

    // Test gauge setting
    metrics_collector.set_gauge("test_gauge", 42.5).await?;
    println!("📊 Gauge set");

    // Test histogram recording
    metrics_collector.record_histogram("test_histogram", 100.0).await?;
    metrics_collector.record_histogram("test_histogram", 150.0).await?;
    metrics_collector.record_histogram("test_histogram", 200.0).await?;
    println!("📊 Histogram values recorded");

    // Get metrics snapshot
    let snapshot = metrics_collector.get_metrics_snapshot().await?;
    println!("📸 Metrics Snapshot:");
    println!("   Counters: {}", snapshot.counters.len());
    println!("   Gauges: {}", snapshot.gauges.len());
    println!("   Histograms: {}", snapshot.histograms.len());

    // Validate metrics
    assert!(snapshot.counters.contains_key("test_counter"));
    assert_eq!(snapshot.counters["test_counter"], 2);
    assert!(snapshot.gauges.contains_key("test_gauge"));
    assert_eq!(snapshot.gauges["test_gauge"], 42.5);
    assert!(snapshot.histograms.contains_key("test_histogram"));

    // Test Prometheus export
    let prometheus_output = metrics_collector.export_prometheus_metrics().await?;
    println!("📊 Prometheus Export:");
    println!("   Length: {} characters", prometheus_output.len());
    assert!(prometheus_output.contains("test_counter"));
    assert!(prometheus_output.contains("test_gauge"));

    metrics_collector.stop().await?;
    println!("✅ Metrics Collector test passed");
    Ok(())
}

#[tokio::test]
async fn test_questdb_client() -> Result<()> {
    println!("🗄️ Testing QuestDB Time Series Client");

    let questdb_config = QuestDbConfig::default();
    let questdb_client = QuestDbClient::new(&questdb_config).await?;

    // Start database connection
    questdb_client.start().await?;
    println!("🚀 QuestDB client started");

    // Create test analytics event
    let test_event = AnalyticsEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type: AnalyticsEventType::TradeExecuted,
        source: "test_system".to_string(),
        data: serde_json::json!({
            "token_address": "TestToken123",
            "action": "buy",
            "volume_sol": 0.5,
            "pnl_sol": 0.1,
            "execution_time_ms": 250.0,
            "confidence": 0.85
        }),
        metadata: HashMap::new(),
    };

    // Test event insertion
    questdb_client.insert_event(&test_event).await?;
    println!("💾 Event inserted into QuestDB");

    // Test query execution
    let query = "SELECT * FROM trading_events LIMIT 10";
    let result = questdb_client.query(query).await?;
    println!("🔍 Query executed:");
    println!("   Columns: {:?}", result.columns);
    println!("   Rows: {}", result.row_count);

    // Test metrics retrieval
    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now();
    
    let trading_metrics = questdb_client.get_trading_metrics(start_time, end_time).await?;
    println!("📊 Trading Metrics:");
    println!("   Total trades: {}", trading_metrics.total_trades);
    println!("   Successful trades: {}", trading_metrics.successful_trades);

    let ai_metrics = questdb_client.get_ai_metrics(start_time, end_time).await?;
    println!("🧠 AI Metrics:");
    println!("   Total analyses: {}", ai_metrics.total_analyses);
    println!("   Avg confidence: {:.3}", ai_metrics.avg_confidence);

    questdb_client.stop().await?;
    println!("✅ QuestDB Client test passed");
    Ok(())
}

#[tokio::test]
async fn test_ai_decision_analyzer() -> Result<()> {
    println!("🧠 Testing AI Decision Analyzer");

    let ai_analyzer = AiDecisionAnalyzer::new().await?;

    // Start AI analysis
    ai_analyzer.start().await?;
    println!("🚀 AI Decision Analyzer started");

    // Create test AI analysis event
    let ai_event = AnalyticsEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type: AnalyticsEventType::AiAnalysisCompleted,
        source: "ai_engine".to_string(),
        data: serde_json::json!({
            "token_address": "TestToken456",
            "decision": "buy",
            "confidence": 0.78,
            "reasoning": "Strong momentum pattern detected",
            "patterns": ["momentum_breakout", "social_viral"],
            "analysis_time_ms": 150.0
        }),
        metadata: HashMap::new(),
    };

    // Analyze AI event
    ai_analyzer.analyze_event(&ai_event).await?;
    println!("📊 AI event analyzed");

    // Create test trade completion event
    let trade_event = AnalyticsEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type: AnalyticsEventType::TradeCompleted,
        source: "trading_engine".to_string(),
        data: serde_json::json!({
            "decision_id": ai_event.event_id,
            "pnl_sol": 0.15,
            "predicted_pnl": 0.12,
            "execution_time_ms": 300.0,
            "market_conditions": "bullish"
        }),
        metadata: HashMap::new(),
    };

    // Update decision outcome
    ai_analyzer.analyze_event(&trade_event).await?;
    println!("📈 Decision outcome updated");

    // Get AI metrics summary
    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now();
    let ai_summary = ai_analyzer.get_metrics_summary(start_time, end_time).await?;

    println!("🧠 AI Metrics Summary:");
    println!("   Total analyses: {}", ai_summary.total_analyses);
    println!("   Avg confidence: {:.3}", ai_summary.avg_confidence);
    println!("   Pattern accuracy: {:.3}", ai_summary.pattern_accuracy);
    println!("   Most successful pattern: {}", ai_summary.most_successful_pattern);

    // Validate metrics
    assert!(ai_summary.total_analyses > 0);
    assert!(ai_summary.avg_confidence >= 0.0 && ai_summary.avg_confidence <= 1.0);

    ai_analyzer.stop().await?;
    println!("✅ AI Decision Analyzer test passed");
    Ok(())
}

#[tokio::test]
async fn test_performance_analyzer() -> Result<()> {
    println!("⚡ Testing Performance Analyzer");

    let performance_analyzer = PerformanceAnalyzer::new().await?;

    // Start performance analysis
    performance_analyzer.start().await?;
    println!("🚀 Performance Analyzer started");

    // Create test performance event
    let perf_event = AnalyticsEvent {
        event_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type: AnalyticsEventType::PerformanceAlert,
        source: "system_monitor".to_string(),
        data: serde_json::json!({
            "alert_type": "high_cpu",
            "severity": "high",
            "impact_score": 0.8,
            "cpu_usage": 85.0,
            "memory_usage": 70.0
        }),
        metadata: HashMap::new(),
    };

    // Analyze performance event
    performance_analyzer.analyze_event(&perf_event).await?;
    println!("📊 Performance event analyzed");

    // Get system metrics summary
    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now();
    let system_summary = performance_analyzer.get_system_summary(start_time, end_time).await?;

    println!("⚡ System Metrics Summary:");
    println!("   Avg CPU usage: {:.1}%", system_summary.avg_cpu_usage);
    println!("   Avg memory usage: {:.1}%", system_summary.avg_memory_usage);
    println!("   Avg response time: {:.1}ms", system_summary.avg_response_time_ms);
    println!("   Total errors: {}", system_summary.total_errors);
    println!("   Uptime: {:.1}%", system_summary.uptime_percentage);

    // Validate metrics
    assert!(system_summary.avg_cpu_usage >= 0.0);
    assert!(system_summary.avg_memory_usage >= 0.0);
    assert!(system_summary.uptime_percentage >= 0.0 && system_summary.uptime_percentage <= 100.0);

    performance_analyzer.stop().await?;
    println!("✅ Performance Analyzer test passed");
    Ok(())
}

#[tokio::test]
async fn test_analytics_event_processing() -> Result<()> {
    println!("🔄 Testing Analytics Event Processing");

    let config = AnalyticsConfig::default();
    let analytics_system = AdvancedAnalyticsSystem::new(config).await?;

    // Start analytics system
    analytics_system.start().await?;
    println!("🚀 Analytics system started");

    // Create various test events
    let events = vec![
        AnalyticsEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AnalyticsEventType::TokenDetected,
            source: "helius_monitor".to_string(),
            data: serde_json::json!({
                "token_address": "NewToken789",
                "confidence": 0.92
            }),
            metadata: HashMap::new(),
        },
        AnalyticsEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AnalyticsEventType::AiAnalysisCompleted,
            source: "ai_engine".to_string(),
            data: serde_json::json!({
                "token_address": "NewToken789",
                "confidence": 0.85,
                "decision": "buy",
                "analysis_time_ms": 200.0
            }),
            metadata: HashMap::new(),
        },
        AnalyticsEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: AnalyticsEventType::TradeExecuted,
            source: "execution_engine".to_string(),
            data: serde_json::json!({
                "token_address": "NewToken789",
                "volume_sol": 0.3,
                "execution_time_ms": 180.0
            }),
            metadata: HashMap::new(),
        },
    ];

    // Process events
    for event in events {
        analytics_system.record_event(event).await?;
        println!("📊 Event processed");
    }

    // Generate analytics summary
    let summary = analytics_system.generate_summary(1).await?; // Last 1 hour
    println!("\n📋 ANALYTICS SUMMARY:");
    println!("   Period: {} to {}", 
        summary.period_start.format("%H:%M:%S"),
        summary.period_end.format("%H:%M:%S"));
    
    println!("   Trading Metrics:");
    println!("     Total trades: {}", summary.trading_metrics.total_trades);
    println!("     Win rate: {:.1}%", summary.trading_metrics.win_rate * 100.0);
    
    println!("   AI Metrics:");
    println!("     Total analyses: {}", summary.ai_metrics.total_analyses);
    println!("     Avg confidence: {:.3}", summary.ai_metrics.avg_confidence);
    
    println!("   System Metrics:");
    println!("     Avg CPU: {:.1}%", summary.system_metrics.avg_cpu_usage);
    println!("     Uptime: {:.1}%", summary.system_metrics.uptime_percentage);
    
    println!("   Performance Insights: {}", summary.performance_insights.len());
    println!("   Recommendations: {}", summary.recommendations.len());

    // Get dashboard data
    let dashboard_data = analytics_system.get_dashboard_data().await?;
    println!("\n📊 Dashboard Data:");
    println!("   {}", serde_json::to_string_pretty(&dashboard_data)?);

    analytics_system.stop().await?;
    println!("\n✅ Analytics Event Processing test passed");
    Ok(())
}

#[tokio::test]
async fn test_analytics_configuration() -> Result<()> {
    println!("⚙️ Testing Analytics Configuration");

    // Test default configuration
    let default_config = AnalyticsConfig::default();
    println!("📋 Default Configuration:");
    println!("   Metrics collection: {}", default_config.enable_metrics_collection);
    println!("   Time series storage: {}", default_config.enable_time_series_storage);
    println!("   AI analytics: {}", default_config.enable_ai_analytics);
    println!("   Performance analysis: {}", default_config.enable_performance_analysis);
    println!("   Real-time monitoring: {}", default_config.enable_real_time_monitoring);
    println!("   Retention days: {}", default_config.metrics_retention_days);

    // Validate default values
    assert!(default_config.enable_metrics_collection);
    assert!(default_config.enable_time_series_storage);
    assert!(default_config.enable_ai_analytics);
    assert!(default_config.enable_performance_analysis);
    assert!(default_config.enable_real_time_monitoring);
    assert_eq!(default_config.metrics_retention_days, 30);

    // Test alert thresholds
    let thresholds = &default_config.alert_thresholds;
    println!("🚨 Alert Thresholds:");
    println!("   AI confidence min: {:.2}", thresholds.ai_confidence_min);
    println!("   Trade success rate min: {:.2}", thresholds.trade_success_rate_min);
    println!("   System CPU max: {:.1}%", thresholds.system_cpu_max);
    println!("   System memory max: {:.1}%", thresholds.system_memory_max);

    // Validate thresholds
    assert!(thresholds.ai_confidence_min >= 0.0 && thresholds.ai_confidence_min <= 1.0);
    assert!(thresholds.trade_success_rate_min >= 0.0 && thresholds.trade_success_rate_min <= 1.0);
    assert!(thresholds.system_cpu_max > 0.0 && thresholds.system_cpu_max <= 100.0);
    assert!(thresholds.system_memory_max > 0.0 && thresholds.system_memory_max <= 100.0);

    println!("✅ Analytics Configuration test passed");
    Ok(())
}

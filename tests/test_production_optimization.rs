use anyhow::Result;
use sniperbot::analytics::{
    AnalyticsOrchestrator, TradingEvent, EnrichedEvent, ProductionAnalyticsConfig,
    PrometheusExporter, AdaptiveSampling, SamplingStrategy,
};
use sniperbot::analytics::production::{
    TimeSeriesLoader, DecisionAnalyzer, ThresholdAlerts, ProductionEventType, EventPriority,
};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

/// Test Production Optimization System
/// Comprehensive testing of production-grade analytics with performance optimization

#[tokio::test]
async fn test_analytics_orchestrator_initialization() -> Result<()> {
    println!("🎯 Testing Analytics Orchestrator Initialization");

    // Initialize components
    let prometheus_config = sniperbot::analytics::production::PrometheusConfig {
        adaptive_sampling: true,
        sampling_rate_min: 0.1,
        sampling_rate_max: 1.0,
        memory_limit_mb: 256,
        batch_export_size: 100,
    };

    let prometheus_adapter = Arc::new(PrometheusExporter::new(prometheus_config).await?);
    let questdb_loader = Arc::new(TimeSeriesLoader::new().await?);
    let ai_validator = Arc::new(DecisionAnalyzer::new().await?);
    let alert_engine = Arc::new(ThresholdAlerts::new().await?);

    let orchestrator_config = sniperbot::analytics::production::OrchestratorConfig {
        enable_event_enrichment: true,
        parallel_processing_threads: 4,
        circuit_breaker_threshold: 0.8,
        batch_size: 100,
        flush_interval_ms: 1000,
    };

    // Initialize Analytics Orchestrator
    let orchestrator = AnalyticsOrchestrator::new(
        prometheus_adapter,
        questdb_loader,
        ai_validator,
        alert_engine,
        orchestrator_config,
    ).await?;

    println!("✅ Analytics Orchestrator initialized successfully");

    // Test metrics retrieval
    let metrics = orchestrator.get_metrics().await;
    println!("📊 Initial Metrics:");
    println!("   Events per second: {:.2}", metrics.events_processed_per_second);
    println!("   Avg latency: {:.2}ms", metrics.avg_event_latency_ms);
    println!("   System availability: {:.1}%", metrics.system_availability_percent);

    // Validate metrics structure
    assert!(metrics.system_availability_percent >= 0.0 && metrics.system_availability_percent <= 100.0);
    assert!(metrics.avg_event_latency_ms >= 0.0);

    println!("✅ Analytics Orchestrator initialization test passed");
    Ok(())
}

#[tokio::test]
async fn test_prometheus_adaptive_sampling() -> Result<()> {
    println!("📈 Testing Prometheus Adaptive Sampling");

    let prometheus_config = sniperbot::analytics::production::PrometheusConfig {
        adaptive_sampling: true,
        sampling_rate_min: 0.1,
        sampling_rate_max: 1.0,
        memory_limit_mb: 128,
        batch_export_size: 50,
    };

    let prometheus_exporter = PrometheusExporter::new(prometheus_config).await?;

    // Create test events with different frequencies
    let high_frequency_events = (0..100).map(|i| TradingEvent {
        event_id: format!("high_freq_{}", i),
        timestamp: Utc::now(),
        event_type: ProductionEventType::MarketDataUpdate,
        priority: EventPriority::Medium,
        source: "market_feed".to_string(),
        data: serde_json::json!({
            "price": 100.0 + i as f64,
            "volume": 1000 + i
        }),
        metadata: HashMap::new(),
    }).collect::<Vec<_>>();

    let low_frequency_events = (0..10).map(|i| TradingEvent {
        event_id: format!("low_freq_{}", i),
        timestamp: Utc::now(),
        event_type: ProductionEventType::TradingDecision,
        priority: EventPriority::High,
        source: "ai_engine".to_string(),
        data: serde_json::json!({
            "confidence": 0.8 + (i as f64 * 0.01),
            "action": "buy"
        }),
        metadata: HashMap::new(),
    }).collect::<Vec<_>>();

    // Process high-frequency events
    println!("📊 Processing high-frequency events...");
    for event in high_frequency_events {
        prometheus_exporter.record(&event).await?;
    }

    // Process low-frequency events
    println!("📊 Processing low-frequency events...");
    for event in low_frequency_events {
        prometheus_exporter.record(&event).await?;
    }

    // Get export statistics
    let export_stats = prometheus_exporter.get_export_stats().await;
    println!("📈 Export Statistics:");
    println!("   Total events received: {}", export_stats.total_events_received);
    println!("   Total events sampled: {}", export_stats.total_events_sampled);
    println!("   Sampling efficiency: {:.2}%", export_stats.sampling_efficiency * 100.0);
    println!("   Total exports: {}", export_stats.total_exports);

    // Get sampling configuration
    let sampling_config = prometheus_exporter.get_sampling_config().await;
    println!("🎲 Sampling Configuration:");
    println!("   Current sampling rate: {:.3}", sampling_config.current_sampling_rate);
    println!("   System load factor: {:.2}", sampling_config.system_load_factor);
    println!("   Event types tracked: {}", sampling_config.event_frequency_tracker.len());

    // Validate adaptive sampling
    assert!(export_stats.total_events_received > 0);
    assert!(export_stats.sampling_efficiency >= 0.0 && export_stats.sampling_efficiency <= 1.0);
    assert!(sampling_config.current_sampling_rate >= 0.1 && sampling_config.current_sampling_rate <= 1.0);

    println!("✅ Prometheus Adaptive Sampling test passed");
    Ok(())
}

#[tokio::test]
async fn test_event_processing_pipeline() -> Result<()> {
    println!("🔄 Testing Event Processing Pipeline");

    // Create production analytics configuration
    let config = ProductionAnalyticsConfig::default();
    
    // Initialize components
    let prometheus_adapter = Arc::new(PrometheusExporter::new(config.prometheus).await?);
    let questdb_loader = Arc::new(TimeSeriesLoader::new().await?);
    let ai_validator = Arc::new(DecisionAnalyzer::new().await?);
    let alert_engine = Arc::new(ThresholdAlerts::new().await?);

    // Initialize orchestrator
    let orchestrator = AnalyticsOrchestrator::new(
        prometheus_adapter,
        questdb_loader,
        ai_validator,
        alert_engine,
        config.orchestrator,
    ).await?;

    // Create test events with different priorities
    let test_events = vec![
        TradingEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ProductionEventType::TradingDecision,
            priority: EventPriority::Critical,
            source: "ai_engine".to_string(),
            data: serde_json::json!({
                "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "action": "buy",
                "confidence": 0.92,
                "position_size": 0.5
            }),
            metadata: HashMap::new(),
        },
        TradingEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ProductionEventType::PerformanceMetric,
            priority: EventPriority::Medium,
            source: "system_monitor".to_string(),
            data: serde_json::json!({
                "cpu_usage": 65.0,
                "memory_usage": 70.0,
                "latency_ms": 8.5
            }),
            metadata: HashMap::new(),
        },
        TradingEvent {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: ProductionEventType::AiAnalysisComplete,
            priority: EventPriority::High,
            source: "ai_analyzer".to_string(),
            data: serde_json::json!({
                "analysis_confidence": 0.87,
                "pattern_matches": ["momentum_breakout", "social_viral"],
                "risk_score": 0.3
            }),
            metadata: HashMap::new(),
        },
    ];

    // Process events through orchestrator
    println!("📦 Processing {} events through orchestrator...", test_events.len());
    
    for (i, event) in test_events.iter().enumerate() {
        println!("   Processing event {}: {} (priority: {:?})", 
            i + 1, event.event_id, event.priority);
        
        orchestrator.process_event(event.clone()).await?;
        
        // Small delay to allow processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Wait for processing to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Get updated metrics
    let final_metrics = orchestrator.get_metrics().await;
    println!("\n📊 Final Processing Metrics:");
    println!("   Events processed per second: {:.2}", final_metrics.events_processed_per_second);
    println!("   Average event latency: {:.2}ms", final_metrics.avg_event_latency_ms);
    println!("   P99 event latency: {:.2}ms", final_metrics.p99_event_latency_ms);
    println!("   System availability: {:.1}%", final_metrics.system_availability_percent);

    // Validate processing metrics
    assert!(final_metrics.events_processed_per_second >= 0.0);
    assert!(final_metrics.avg_event_latency_ms >= 0.0);
    assert!(final_metrics.system_availability_percent >= 95.0); // High availability expected

    println!("✅ Event Processing Pipeline test passed");
    Ok(())
}

#[tokio::test]
async fn test_production_configuration() -> Result<()> {
    println!("⚙️ Testing Production Configuration");

    // Test default configuration
    let default_config = ProductionAnalyticsConfig::default();
    
    println!("📋 Default Production Configuration:");
    println!("   Orchestrator threads: {}", default_config.orchestrator.parallel_processing_threads);
    println!("   Circuit breaker threshold: {:.2}", default_config.orchestrator.circuit_breaker_threshold);
    println!("   Batch size: {}", default_config.orchestrator.batch_size);
    
    println!("   Prometheus adaptive sampling: {}", default_config.prometheus.adaptive_sampling);
    println!("   Sampling rate range: {:.1} - {:.1}", 
        default_config.prometheus.sampling_rate_min, 
        default_config.prometheus.sampling_rate_max);
    
    println!("   QuestDB cross-shard aggregation: {}", default_config.questdb.cross_shard_aggregation);
    println!("   Shard count: {}", default_config.questdb.shard_count);
    
    println!("   AI confidence tracking: {}", default_config.ai_validation.confidence_tracking);
    println!("   Closed-loop feedback: {}", default_config.ai_validation.closed_loop_feedback);
    
    println!("   Confidence-driven alerts: {}", default_config.alerting.confidence_driven);
    println!("   Dynamic thresholds: {}", default_config.alerting.dynamic_thresholds);

    // Validate configuration values
    assert!(default_config.orchestrator.parallel_processing_threads > 0);
    assert!(default_config.orchestrator.circuit_breaker_threshold > 0.0 && default_config.orchestrator.circuit_breaker_threshold <= 1.0);
    assert!(default_config.orchestrator.batch_size > 0);
    
    assert!(default_config.prometheus.sampling_rate_min >= 0.0 && default_config.prometheus.sampling_rate_min <= 1.0);
    assert!(default_config.prometheus.sampling_rate_max >= default_config.prometheus.sampling_rate_min);
    assert!(default_config.prometheus.sampling_rate_max <= 1.0);
    
    assert!(default_config.questdb.shard_count > 0);
    assert!(default_config.questdb.compression_level <= 9); // Max compression level
    
    assert!(default_config.ai_validation.learning_rate > 0.0 && default_config.ai_validation.learning_rate < 1.0);
    assert!(default_config.ai_validation.validation_window_hours > 0);

    // Test performance targets
    println!("\n🎯 Performance Targets:");
    println!("   Event latency target: {}ms", default_config.performance.target_event_latency_ms);
    println!("   Query throughput target: {} qps", default_config.performance.target_query_throughput);
    println!("   Alert time target: {}ms", default_config.performance.target_alert_time_ms);
    println!("   Storage cost reduction: {:.1}%", default_config.performance.storage_cost_reduction_percent);

    // Validate performance targets
    assert_eq!(default_config.performance.target_event_latency_ms, 10); // <10ms p99
    assert_eq!(default_config.performance.target_query_throughput, 100_000); // 100k qps
    assert_eq!(default_config.performance.target_alert_time_ms, 200); // <200ms
    assert_eq!(default_config.performance.storage_cost_reduction_percent, 50.0); // 50% reduction

    println!("✅ Production Configuration test passed");
    Ok(())
}

#[tokio::test]
async fn test_performance_optimization_targets() -> Result<()> {
    println!("⚡ Testing Performance Optimization Targets");

    let config = ProductionAnalyticsConfig::default();
    
    // Test performance targets from your specification
    println!("🎯 Performance Optimization Matrix:");
    println!("   Event Latency Target: <{}ms p99", config.performance.target_event_latency_ms);
    println!("   Query Throughput Target: {} qps", config.performance.target_query_throughput);
    println!("   Alert Time Target: <{}ms", config.performance.target_alert_time_ms);
    println!("   Storage Cost Reduction: {:.0}%", config.performance.storage_cost_reduction_percent);

    // Validate against your specified targets
    assert_eq!(config.performance.target_event_latency_ms, 10, "Event latency should be <10ms p99");
    assert_eq!(config.performance.target_query_throughput, 100_000, "Query throughput should be 100k qps");
    assert_eq!(config.performance.target_alert_time_ms, 200, "Alert time should be <200ms");
    assert_eq!(config.performance.storage_cost_reduction_percent, 50.0, "Storage cost reduction should be 50%");

    // Test optimization techniques
    println!("\n🔧 Optimization Techniques:");
    println!("   Kernel bypass networking: {}", config.performance.enable_kernel_bypass);
    println!("   Columnar indexing: {}", config.performance.enable_columnar_indexing);

    assert!(config.performance.enable_kernel_bypass, "Kernel bypass should be enabled for <10ms latency");
    assert!(config.performance.enable_columnar_indexing, "Columnar indexing should be enabled for 100k qps");

    println!("✅ Performance Optimization Targets test passed");
    Ok(())
}

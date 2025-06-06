/*!
🧪 Pipeline Controller Integration Test
Tests the complete pipeline: Soul Meteor → Crawl4AI → Trading Decisions
*/

use sniper_bot::pipeline::{PipelineController};
use sniper_bot::pipeline::controller::PipelineConfig;
use sniper_bot::data_fetcher::soul_meteor_scanner::SoulMeteorScannerConfig;
use sniper_bot::data_fetcher::textual_data_fetcher::TextualDataFetcherConfig;
use sniper_bot::pipeline::decision_engine::DecisionEngineConfig;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 Starting Pipeline Controller Integration Test");

    // Create custom pipeline configuration
    let pipeline_config = PipelineConfig {
        soul_meteor_config: SoulMeteorScannerConfig {
            python_executable: "python3".to_string(),
            script_path: "./pyinstaller_scripts/soul_meteor_scanner.py".to_string(),
            timeout_seconds: 60,
            max_opportunities: 10, // Limit for testing
        },
        crawl4ai_config: TextualDataFetcherConfig {
            executable_path: "./pyinstaller_scripts/crawl4ai_service/dist/crawl4ai_service".to_string(),
            default_data_types: vec!["news".to_string(), "social".to_string()],
            default_time_range_hours: 24,
            default_max_results: 10,
            default_sentiment_analysis: true,
            timeout_seconds: 30,
        },
        decision_config: DecisionEngineConfig {
            max_position_size_sol: 0.5, // Conservative for testing
            min_position_size_sol: 0.05,
            risk_tolerance: 0.7,
            min_confidence: 0.6,
            max_concurrent_positions: 3,
            available_balance_sol: 5.0, // Test balance
        },
        min_sentiment_score: 0.1,
        min_sentiment_confidence: 0.4,
        max_validation_candidates: 5, // Limit for testing
        cycle_interval_seconds: 300,
    };

    // Create pipeline controller
    let mut controller = PipelineController::with_config(pipeline_config);
    
    info!("📋 Pipeline Configuration:");
    info!("  • Soul Meteor: {}", controller.config().soul_meteor_config.script_path);
    info!("  • Crawl4AI: {}", controller.config().crawl4ai_config.executable_path);
    info!("  • Max Candidates: {}", controller.config().max_validation_candidates);
    info!("  • Min Sentiment: {:.2}", controller.config().min_sentiment_score);
    info!("  • Risk Tolerance: {:.2}", controller.config().decision_config.risk_tolerance);

    // Check if pipeline is ready
    if !controller.is_ready() {
        error!("❌ Pipeline not ready - missing components");
        return Err("Pipeline components not available".into());
    }

    info!("✅ Pipeline components verified and ready");

    // Run a single pipeline cycle
    info!("🚀 Executing complete pipeline cycle...");
    
    match controller.run_cycle().await {
        Ok(decisions) => {
            info!("🎉 SUCCESS! Pipeline cycle completed");
            info!("📊 Generated {} trading decisions", decisions.len());
            
            // Display decisions
            if !decisions.is_empty() {
                info!("🎯 TRADING DECISIONS:");
                for (i, decision) in decisions.iter().enumerate() {
                    info!(
                        "  {}. Decision ID: {} | Type: {:?} | Confidence: {:.2} | Priority: {}",
                        i + 1,
                        decision.id,
                        decision.decision_type,
                        decision.confidence,
                        decision.priority
                    );
                    info!("     Reasoning: {}", decision.reasoning);
                    
                    match &decision.decision_type {
                        sniper_bot::pipeline::decision_engine::DecisionType::BuyToken { token_address, target_amount_sol } => {
                            info!("     → BUY {} SOL of token {}", target_amount_sol, &token_address[..8]);
                        }
                        sniper_bot::pipeline::decision_engine::DecisionType::ProvideLiquidity { pool_address, amount_sol, duration_hours } => {
                            info!("     → PROVIDE {} SOL liquidity to {} for {}h", amount_sol, &pool_address[..8], duration_hours);
                        }
                        sniper_bot::pipeline::decision_engine::DecisionType::Monitor { check_interval_minutes, max_monitoring_hours } => {
                            info!("     → MONITOR every {}min for max {}h", check_interval_minutes, max_monitoring_hours);
                        }
                        sniper_bot::pipeline::decision_engine::DecisionType::NoAction { reason } => {
                            info!("     → NO ACTION: {}", reason);
                        }
                        _ => {
                            info!("     → Other decision type");
                        }
                    }
                }
            } else {
                info!("📊 No actionable decisions generated in this cycle");
            }

            // Display pipeline statistics
            let stats = controller.stats();
            info!("📈 PIPELINE STATISTICS:");
            info!("  • Cycles Completed: {}", stats.cycles_completed);
            info!("  • Total Candidates Found: {}", stats.total_candidates_found);
            info!("  • Total Validated: {}", stats.total_candidates_validated);
            info!("  • Total Opportunities: {}", stats.total_opportunities_created);
            info!("  • Total Decisions: {}", stats.total_decisions_made);
            info!("  • Last Cycle Duration: {}ms", stats.last_cycle_duration_ms);
            info!("  • Average Duration: {}ms", stats.average_cycle_duration_ms);

            // Display active opportunities
            let active_opportunities = controller.active_opportunities();
            if !active_opportunities.is_empty() {
                info!("🔥 ACTIVE OPPORTUNITIES:");
                for (id, opportunity) in active_opportunities.iter().take(5) {
                    info!("  • {}: {}", &id[..16], opportunity.summary());
                }
            }

            // Calculate success metrics
            let validation_rate = if stats.total_candidates_found > 0 {
                (stats.total_candidates_validated as f64 / stats.total_candidates_found as f64) * 100.0
            } else {
                0.0
            };

            let decision_rate = if stats.total_candidates_validated > 0 {
                (stats.total_decisions_made as f64 / stats.total_candidates_validated as f64) * 100.0
            } else {
                0.0
            };

            info!("🎯 SUCCESS METRICS:");
            info!("  • Validation Rate: {:.1}%", validation_rate);
            info!("  • Decision Rate: {:.1}%", decision_rate);
            info!("  • Processing Speed: {:.1} candidates/second", 
                  stats.total_candidates_found as f64 / (stats.last_cycle_duration_ms as f64 / 1000.0));

            info!("✅ Pipeline Controller integration test completed successfully!");
        }
        Err(e) => {
            error!("❌ Pipeline cycle failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

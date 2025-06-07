/*!
🧪 Soul Meteor Scanner Integration Test
Tests the Rust ↔ Python integration for Soul Meteor Scanner
*/

use sniper_bot::data_fetcher::soul_meteor_scanner::SoulMeteorScanner;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 Starting Soul Meteor Scanner Integration Test");

    // Create scanner with default config
    let scanner = SoulMeteorScanner::new();
    
    info!("📋 Scanner Configuration:");
    info!("  • Python: {}", scanner.config().python_executable);
    info!("  • Script: {}", scanner.config().script_path);
    info!("  • Timeout: {}s", scanner.config().timeout_seconds);
    info!("  • Max Results: {}", scanner.config().max_opportunities);

    // Check if script is available
    if !scanner.is_available() {
        error!("❌ Soul Meteor Scanner script not found at: {}", scanner.config().script_path);
        return Err("Script not found".into());
    }

    info!("✅ Soul Meteor Scanner script found");

    // Test the scanner
    info!("🚀 Executing Soul Meteor Scanner...");
    
    match scanner.scan_for_opportunities().await {
        Ok(opportunities) => {
            info!("🎉 SUCCESS! Found {} hot opportunities", opportunities.len());
            
            // Display top 5 opportunities
            info!("🏆 TOP 5 OPPORTUNITIES:");
            for (i, opp) in opportunities.iter().take(5).enumerate() {
                info!(
                    "  {}. {} | Score: {:.2} | Liquidity: ${:.0} | Volume: ${:.0} | APR: {:.1}%",
                    i + 1,
                    opp.name,
                    opp.opportunity_score,
                    opp.liquidity_usd,
                    opp.volume_24h,
                    opp.apr
                );
            }

            // Test filtering functions
            info!("🔍 Testing filtering functions...");
            
            let high_score_opportunities = SoulMeteorScanner::filter_candidates(
                &opportunities, 
                50000.0,  // Min $50k liquidity
                3.5       // Min 3.5 score
            );
            
            info!("📊 High-score opportunities (>3.5 score, >$50k liquidity): {}", 
                  high_score_opportunities.len());

            let top_3 = SoulMeteorScanner::get_top_opportunities(&opportunities, 3);
            info!("🥇 Top 3 by score:");
            for (i, opp) in top_3.iter().enumerate() {
                info!("  {}. {} - Score: {:.2}", i + 1, opp.name, opp.opportunity_score);
            }

            info!("✅ Integration test completed successfully!");
        }
        Err(e) => {
            error!("❌ Soul Meteor Scanner failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

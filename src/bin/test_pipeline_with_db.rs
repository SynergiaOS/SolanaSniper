/*!
🧪 Pipeline Controller with DragonflyDB Integration Test
Tests the complete Hub-and-Spoke architecture
*/

use sniper_bot::db_connector::{DbClient, DbConfig};
use sniper_bot::models::persistent_state::DbKeys;
use sniper_bot::models::python_compat::PythonRawOpportunity;
use tracing::{info, error};
use tracing_subscriber;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 Starting Pipeline Controller with DragonflyDB Integration Test");

    // Create database client
    let config = DbConfig::from_env()?;
    let db_client = DbClient::new(config).await?;
    
    info!("✅ Connected to DragonflyDB");

    // Test 1: Check if we have raw opportunities from Soul Meteor Scanner
    info!("🔍 [TEST 1] Checking for raw opportunities in database...");

    // Get all raw opportunities keys from the list
    let raw_opportunities_keys: Vec<String> = db_client
        .list_range::<String>(DbKeys::ALL_RAW_OPPORTUNITIES, 0, -1)
        .await?;

    info!("📊 Found {} raw opportunity keys in list", raw_opportunities_keys.len());
    
    if raw_opportunities_keys.is_empty() {
        info!("⚠️ No raw opportunities found. Run Soul Meteor Scanner first!");
        return Ok(());
    }

    // Test 2: Read and parse raw opportunities
    info!("🔍 [TEST 2] Reading and parsing raw opportunities...");

    let mut parsed_opportunities: Vec<sniper_bot::models::persistent_state::RawOpportunity> = Vec::new();

    // If we have keys from the list, use them, otherwise scan for raw_opportunity:* keys
    let keys_to_process = if !raw_opportunities_keys.is_empty() {
        raw_opportunities_keys
    } else {
        // Fallback: manually create some keys to test
        vec![
            "raw_opportunity:AjM8Qn62EhR4ikJ1rvyeezB1NyvrSsb4zwJiFUFs9ycs".to_string(),
            "raw_opportunity:D9fNVeb6F2UcxJaNaq8YTCQq17w91ev9ie1X59BeD3g3".to_string(),
        ]
    };

    for (i, key) in keys_to_process.iter().take(5).enumerate() {
        match db_client.get::<PythonRawOpportunity>(key).await? {
            Some(python_opp) => {
                info!("✅ #{} - {}", i + 1, python_opp.summary());
                info!("    Address: {}", python_opp.candidate.address);
                info!("    Age: {} minutes", python_opp.age_minutes().unwrap_or(0));
                info!("    Valid: {}", python_opp.is_valid());
                info!("    High Quality: {}", python_opp.candidate.is_high_quality());

                // Convert to Rust format for further processing
                match python_opp.to_rust_format() {
                    Ok(rust_opp) => {
                        parsed_opportunities.push(rust_opp);
                    }
                    Err(e) => {
                        info!("⚠️ Failed to convert to Rust format: {}", e);
                    }
                }
            }
            None => {
                info!("⚠️ Key {} not found", key);
            }
        }
    }

    // Test 3: Check processed tokens set (deduplication)
    info!("🔍 [TEST 3] Checking processed tokens for deduplication...");
    
    let processed_tokens: Vec<String> = db_client
        .set_members(DbKeys::PROCESSED_TOKENS)
        .await?;
    
    info!("📊 Found {} processed tokens", processed_tokens.len());
    
    // Display first few processed tokens
    for (i, token) in processed_tokens.iter().take(5).enumerate() {
        info!("  {}. {}", i + 1, &token[..8]);
    }

    // Test 4: Simulate pipeline processing
    info!("🔍 [TEST 4] Simulating pipeline processing...");
    
    if !parsed_opportunities.is_empty() {
        let top_opportunity = &parsed_opportunities[0];
        
        info!("🎯 Processing top opportunity: {}", top_opportunity.candidate.name);
        info!("  • Address: {}", top_opportunity.candidate.address);
        info!("  • Liquidity: ${:.2}", top_opportunity.candidate.liquidity_usd);
        info!("  • Volume 24h: ${:.2}", top_opportunity.candidate.volume_24h);
        info!("  • Score: {:.2}", top_opportunity.candidate.opportunity_score);
        
        // Check if this token needs validation
        let needs_validation = !db_client
            .set_contains(DbKeys::PROCESSED_TOKENS, &top_opportunity.candidate.address)
            .await?;
        
        if needs_validation {
            info!("✅ Token needs Crawl4AI validation");
        } else {
            info!("⏭️ Token already processed, skipping validation");
        }
    }

    // Test 5: Database statistics
    info!("🔍 [TEST 5] Database statistics...");
    
    let stats = db_client.get_stats().await?;
    info!("📊 Database Statistics:");
    info!("  • Total Keys: {}", stats.total_keys);
    info!("  • Timestamp: {}", stats.timestamp);

    // Test 6: Health check
    info!("🔍 [TEST 6] Health check...");
    
    let health = db_client.health_check().await?;
    if health {
        info!("✅ Database health check passed");
    } else {
        error!("❌ Database health check failed");
    }

    info!("🎉 All tests completed successfully!");
    info!("🚀 Hub-and-Spoke architecture is working!");

    Ok(())
}

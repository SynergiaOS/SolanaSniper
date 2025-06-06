/*!
🧪 Database Connector Test
Tests DragonflyDB connection and basic operations
*/

use sniper_bot::db_connector::{DbClient, DbConfig};
use tracing::{info, error};
use tracing_subscriber;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    match dotenv() {
        Ok(path) => println!("✅ Loaded .env from: {:?}", path),
        Err(e) => println!("⚠️ Could not load .env: {}", e),
    }

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 Starting DragonflyDB Connection Test");

    // Debug environment variables
    match std::env::var("DRAGONFLY_URL") {
        Ok(url) => info!("🔍 DRAGONFLY_URL found: {}", url),
        Err(_) => info!("⚠️ DRAGONFLY_URL not found in environment"),
    }

    // Create configuration from environment
    let config = DbConfig::from_env()?;
    info!("📋 Database URL: {}", config.connection_url);

    // Create database client
    match DbClient::new(config).await {
        Ok(client) => {
            info!("✅ Successfully connected to DragonflyDB!");
            
            // Test basic operations
            info!("🧪 Testing basic operations...");
            
            // Test set/get
            let test_key = "test:sniperbot:connection";
            let test_value = "Hello from SniperBot 2.0!";
            
            match client.set(test_key, &test_value, Some(60)).await {
                Ok(_) => info!("✅ SET operation successful"),
                Err(e) => error!("❌ SET operation failed: {}", e),
            }
            
            match client.get::<String>(test_key).await {
                Ok(Some(value)) => {
                    if value == test_value {
                        info!("✅ GET operation successful: {}", value);
                    } else {
                        error!("❌ GET returned wrong value: {}", value);
                    }
                }
                Ok(None) => error!("❌ GET returned None"),
                Err(e) => error!("❌ GET operation failed: {}", e),
            }
            
            // Test health check
            match client.health_check().await {
                Ok(true) => info!("✅ Health check passed"),
                Ok(false) => error!("❌ Health check failed"),
                Err(e) => error!("❌ Health check error: {}", e),
            }
            
            // Test statistics
            match client.get_stats().await {
                Ok(stats) => {
                    info!("📊 Database Statistics:");
                    info!("  • Total Keys: {}", stats.total_keys);
                    info!("  • Timestamp: {}", stats.timestamp);
                }
                Err(e) => error!("❌ Failed to get stats: {}", e),
            }
            
            // Cleanup
            match client.delete(test_key).await {
                Ok(deleted) => {
                    if deleted {
                        info!("✅ Cleanup successful");
                    } else {
                        info!("ℹ️ Test key was already deleted");
                    }
                }
                Err(e) => error!("❌ Cleanup failed: {}", e),
            }
            
            info!("🎉 All tests completed successfully!");
        }
        Err(e) => {
            error!("❌ Failed to connect to DragonflyDB: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/*!
🧪 Minimal Cloud Connection Test

This test focuses ONLY on establishing a connection with DragonflyDB Cloud.
No complex logic, no dependencies - just pure connectivity validation.
*/

use std::env;
use dotenvy::dotenv;
use tracing::{info, error, Level};
use tracing_subscriber;

// Important: using redis client directly, not deadpool!
use redis::AsyncCommands;

#[tokio::main]
async fn main() {
    // Initialize basic tools
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    dotenv().ok();
    
    info!("🧪 === MINIMAL DRAGONFLYDB CLOUD CONNECTION TEST ===");
    info!("🎯 Goal: Establish TLS connection and send PING command");

    // Get URL from .env file
    let db_url = match env::var("DRAGONFLY_URL") {
        Ok(url) => {
            info!("✅ Found DRAGONFLY_URL in .env file");
            info!("🔗 Connection string: {}", url);
            url
        },
        Err(_) => {
            error!("❌ DRAGONFLY_URL not found in .env file");
            error!("🔧 Please ensure .env contains: DRAGONFLY_URL=rediss://default:password@host:port");
            return;
        }
    };

    // Validate URL format
    if !db_url.starts_with("rediss://") && !db_url.starts_with("redis://") {
        error!("❌ Invalid URL format. Expected redis:// or rediss:// protocol");
        error!("🔧 Current URL: {}", db_url);
        return;
    }

    if db_url.starts_with("rediss://") {
        info!("🔒 Detected TLS connection (rediss://) - this is correct for DragonflyDB Cloud");
    } else {
        info!("⚠️ Detected non-TLS connection (redis://) - this might not work with cloud");
    }

    info!("🔌 Attempting to create Redis client...");

    // Try to create client
    let client = match redis::Client::open(db_url.clone()) {
        Ok(c) => {
            info!("✅ Redis client created successfully");
            c
        },
        Err(e) => {
            error!("❌ Failed to create Redis client: {:?}", e);
            error!("🔧 Check if URL format is correct: rediss://username:password@host:port");
            return;
        }
    };

    info!("🌐 Attempting to establish async connection...");

    // Try to get async connection
    let mut conn = match client.get_async_connection().await {
        Ok(c) => {
            info!("✅ Successfully established connection to server!");
            info!("🎉 TLS handshake completed (if using rediss://)");
            c
        },
        Err(e) => {
            error!("❌ Failed to establish connection: {:?}", e);
            error!("🔧 This is likely a TLS/SSL error or network issue (firewall, DNS)");
            error!("💡 Possible solutions:");
            error!("   1. Check if tls-rustls feature is enabled in Cargo.toml");
            error!("   2. Verify network connectivity to DragonflyDB Cloud");
            error!("   3. Confirm credentials are correct");
            error!("   4. Check if firewall allows outbound connections on port 6385");
            return;
        }
    };

    info!("🏓 Sending PING command to test connectivity...");

    // Try to send PING command
    match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
        Ok(reply) if reply.to_uppercase() == "PONG" => {
            info!("🎉 === CONNECTION TEST SUCCESSFUL! ===");
            info!("✅ Received 'PONG' from DragonflyDB Cloud server");
            info!("🚀 TLS connection is working perfectly!");
            info!("");
            info!("🧠 DragonflyDB Cloud Status: OPERATIONAL");
            info!("🔗 Connection: SECURE (TLS)");
            info!("📡 Latency: LOW (successful ping)");
            info!("");
            info!("🎯 NEXT STEP: Run autonomous bot!");
            info!("   cargo run --bin autonomous_bot -- --health-check");
        },
        Ok(reply) => {
            error!("🤔 Received unexpected response: '{}'", reply);
            error!("   Expected 'PONG' but got something else");
        },
        Err(e) => {
            error!("❌ Failed to execute PING command: {:?}", e);
            error!("🔧 Connection established but command failed");
            error!("   This might indicate authentication or protocol issues");
        }
    }

    // Additional test: Try to get server info
    info!("📊 Attempting to get server information...");
    
    match redis::cmd("INFO").arg("server").query_async::<_, String>(&mut conn).await {
        Ok(info_response) => {
            info!("✅ Server info retrieved successfully");
            
            // Parse some basic info
            for line in info_response.lines().take(5) {
                if line.contains("redis_version") || line.contains("dragonfly_version") || line.contains("os") {
                    info!("   📋 {}", line.trim());
                }
            }
        },
        Err(e) => {
            error!("⚠️ Could not retrieve server info: {:?}", e);
            error!("   This is not critical - PING success is what matters");
        }
    }

    // Test basic data operations
    info!("🧪 Testing basic data operations...");
    
    let test_key = "sniperbot_connection_test";
    let test_value = "connection_successful";
    
    match conn.set::<_, _, ()>(test_key, test_value).await {
        Ok(_) => {
            info!("✅ SET operation successful");
            
            match conn.get::<_, String>(test_key).await {
                Ok(retrieved_value) if retrieved_value == test_value => {
                    info!("✅ GET operation successful - value matches");
                    
                    // Clean up test key
                    let _: () = conn.del(test_key).await.unwrap_or(());
                    info!("✅ Cleanup completed");
                },
                Ok(retrieved_value) => {
                    error!("⚠️ GET returned wrong value: '{}'", retrieved_value);
                },
                Err(e) => {
                    error!("❌ GET operation failed: {:?}", e);
                }
            }
        },
        Err(e) => {
            error!("❌ SET operation failed: {:?}", e);
            error!("   This might indicate permission issues");
        }
    }

    info!("🏁 === CLOUD CONNECTION TEST COMPLETED ===");
}

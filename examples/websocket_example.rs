use sniper_bot::config::Config;
use sniper_bot::data_fetcher::realtime_websocket_manager::RealtimeWebSocketManager;
use sniper_bot::models::{MarketEvent, WebSocketConfig, SubscriptionType, WebSocketSubscription};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Starting SniperBot WebSocket Example");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Create event channel for receiving market events
    let (event_sender, mut event_receiver) = mpsc::channel::<MarketEvent>(1000);

    // Create WebSocket configuration
    let ws_config = WebSocketConfig {
        enabled: true,
        helius_ws_url: Some("wss://atlas-mainnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}".to_string()),
        jupiter_ws_url: None, // Jupiter doesn't have WebSocket yet
        binance_ws_url: Some("wss://stream.binance.com:9443/ws".to_string()),
        reconnect_timeout_seconds: 5,
        max_retries: 3,
        ping_interval_seconds: 30,
        subscriptions: vec![
            WebSocketSubscription {
                subscription_type: SubscriptionType::PriceUpdates,
                symbol: Some("SOL/USDT".to_string()),
                address: None,
                enabled: true,
            },
            WebSocketSubscription {
                subscription_type: SubscriptionType::NewTokens,
                symbol: None,
                address: None,
                enabled: true,
            },
        ],
    };

    // Create WebSocket manager
    let ws_manager = RealtimeWebSocketManager::new(ws_config, event_sender);

    // Start event processing task
    let event_processor = tokio::spawn(async move {
        info!("📡 Starting event processor");
        
        let mut price_updates = 0;
        let mut connection_events = 0;
        let mut raw_messages = 0;
        let mut other_events = 0;

        while let Some(event) = event_receiver.recv().await {
            match event {
                MarketEvent::PriceUpdate { symbol, price, volume_24h, timestamp, source } => {
                    price_updates += 1;
                    info!("💰 Price Update: {} = ${:.4} (Volume: {:?}) from {} at {}", 
                        symbol, price, volume_24h, source, timestamp);
                    
                    // Example: React to significant price changes
                    if symbol == "SOLUSDT" && price > 200.0 {
                        info!("🚨 SOL price above $200! Consider taking action.");
                    }
                }
                
                MarketEvent::NewTransaction { signature, token_address, amount, price, transaction_type, timestamp } => {
                    other_events += 1;
                    info!("🔄 New Transaction: {} for {} tokens of {} (Price: {:?}) Type: {:?} at {}", 
                        signature, amount, token_address, price, transaction_type, timestamp);
                }
                
                MarketEvent::NewTokenListing { token_address, symbol, name, initial_price, initial_liquidity, creator, timestamp } => {
                    other_events += 1;
                    info!("🆕 New Token: {} ({:?}) at {} - Price: {:?}, Liquidity: {:?}, Creator: {:?} at {}", 
                        token_address, symbol, name.unwrap_or_default(), initial_price, initial_liquidity, creator, timestamp);
                }
                
                MarketEvent::WhaleAlert { signature, token_address, amount_usd, transaction_type, wallet_address, timestamp } => {
                    other_events += 1;
                    warn!("🐋 Whale Alert: ${:.2} {:?} of {} by {} ({}) at {}",
                        amount_usd, transaction_type, token_address, wallet_address, signature, timestamp);
                }
                
                MarketEvent::ConnectionStatus { connected, source, error, timestamp } => {
                    connection_events += 1;
                    if connected {
                        info!("✅ Connected to {} at {}", source, timestamp);
                    } else {
                        warn!("❌ Disconnected from {} at {} - Error: {:?}", source, timestamp, error);
                    }
                }
                
                MarketEvent::RawMessage { source, data, timestamp } => {
                    raw_messages += 1;
                    if raw_messages % 10 == 0 { // Log every 10th raw message to avoid spam
                        info!("📨 Raw message #{} from {} at {}: {}", raw_messages, source, timestamp, 
                            if data.len() > 100 { format!("{}...", &data[..100]) } else { data });
                    }
                }
                
                _ => {
                    other_events += 1;
                    info!("📊 Other event: {:?}", event);
                }
            }
            
            // Print statistics every 100 events
            let total_events = price_updates + connection_events + raw_messages + other_events;
            if total_events % 100 == 0 {
                info!("📈 Event Statistics: {} total (Price: {}, Connection: {}, Raw: {}, Other: {})", 
                    total_events, price_updates, connection_events, raw_messages, other_events);
            }
        }
        
        info!("📡 Event processor stopped");
    });

    // Start WebSocket manager (this will run indefinitely)
    let ws_task = tokio::spawn(async move {
        if let Err(e) = ws_manager.start().await {
            error!("WebSocket manager failed: {}", e);
        }
    });

    // Run for a limited time in this example
    info!("🕐 Running WebSocket example for 30 seconds...");
    tokio::time::sleep(Duration::from_secs(30)).await;

    info!("⏹️ Stopping WebSocket example");
    
    // Cancel tasks
    ws_task.abort();
    event_processor.abort();

    // Wait a bit for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    info!("✅ WebSocket example completed");
    Ok(())
}

// Example of how to use WebSocket manager in a real trading bot
#[allow(dead_code)]
async fn trading_bot_example() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::default();
    
    // Create event channel
    let (event_sender, mut event_receiver) = mpsc::channel::<MarketEvent>(1000);
    
    // Create WebSocket manager with config
    let ws_manager = RealtimeWebSocketManager::new(config.websocket, event_sender);
    
    // Start WebSocket manager in background
    let _ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_manager.start().await {
            error!("WebSocket manager error: {}", e);
        }
    });
    
    // Process events for trading decisions
    while let Some(event) = event_receiver.recv().await {
        match event {
            MarketEvent::PriceUpdate { symbol, price, .. } => {
                // Example trading logic
                if symbol == "SOLUSDT" {
                    if price > 250.0 {
                        info!("🔴 SOL price high, consider selling");
                        // Execute sell order
                    } else if price < 150.0 {
                        info!("🟢 SOL price low, consider buying");
                        // Execute buy order
                    }
                }
            }
            
            MarketEvent::NewTokenListing { token_address, initial_liquidity, .. } => {
                // Example new token sniping logic
                if let Some(liquidity) = initial_liquidity {
                    if liquidity > 100_000.0 {
                        info!("🎯 High liquidity new token detected: {}", token_address);
                        // Analyze and potentially snipe
                    }
                }
            }
            
            MarketEvent::WhaleAlert { amount_usd, transaction_type, .. } => {
                // Example whale following logic
                if amount_usd > 1_000_000.0 {
                    info!("🐋 Large whale movement detected: ${:.2} {:?}", amount_usd, transaction_type);
                    // Follow whale strategy
                }
            }
            
            _ => {
                // Handle other events
            }
        }
    }
    
    Ok(())
}

// Example of custom event filtering
#[allow(dead_code)]
fn should_process_event(event: &MarketEvent) -> bool {
    match event {
        MarketEvent::PriceUpdate { symbol, .. } => {
            // Only process SOL and major tokens
            symbol.contains("SOL") || symbol.contains("BTC") || symbol.contains("ETH")
        }
        
        MarketEvent::NewTokenListing { initial_liquidity, .. } => {
            // Only process tokens with significant liquidity
            initial_liquidity.unwrap_or(0.0) > 50_000.0
        }
        
        MarketEvent::WhaleAlert { amount_usd, .. } => {
            // Only process large whale movements
            *amount_usd > 500_000.0
        }
        
        MarketEvent::ConnectionStatus { .. } => true, // Always process connection events
        
        _ => false, // Filter out other events
    }
}

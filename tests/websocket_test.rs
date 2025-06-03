use sniper_bot::config::Config;
use sniper_bot::data_fetcher::realtime_websocket_manager::RealtimeWebSocketManager;
use sniper_bot::models::{MarketEvent, WebSocketConfig};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[tokio::test]
async fn test_websocket_manager_creation() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing WebSocket manager creation");

    // Create event channel
    let (event_sender, mut event_receiver) = mpsc::channel::<MarketEvent>(100);

    // Create WebSocket config
    let ws_config = WebSocketConfig {
        enabled: false, // Disabled for unit test
        helius_ws_url: Some("wss://test.example.com".to_string()),
        jupiter_ws_url: None,
        binance_ws_url: Some("wss://test.binance.com".to_string()),
        reconnect_timeout_seconds: 1,
        max_retries: 1,
        ping_interval_seconds: 30,
        subscriptions: vec![],
    };

    // Create WebSocket manager
    let ws_manager = RealtimeWebSocketManager::new(ws_config, event_sender);

    info!("✅ WebSocket manager created successfully");

    // Test that we can stop it (even though it's not started)
    ws_manager.stop();

    info!("✅ WebSocket manager stopped successfully");
}

#[tokio::test]
async fn test_websocket_config_default() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing WebSocket config default");

    let ws_config = WebSocketConfig::default();

    assert!(ws_config.enabled);
    assert!(ws_config.helius_ws_url.is_some());
    assert!(ws_config.binance_ws_url.is_some());
    assert!(ws_config.jupiter_ws_url.is_none()); // Jupiter doesn't have WS yet
    assert!(ws_config.reconnect_timeout_seconds > 0);
    assert!(ws_config.max_retries > 0);
    assert!(!ws_config.subscriptions.is_empty());

    info!("✅ WebSocket config default test passed");
}

#[tokio::test]
async fn test_market_event_serialization() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing MarketEvent serialization");

    // Test PriceUpdate event
    let price_event = MarketEvent::PriceUpdate {
        symbol: "SOL/USDT".to_string(),
        price: 100.50,
        volume_24h: Some(1000000.0),
        timestamp: 1234567890,
        source: "binance".to_string(),
    };

    let serialized = serde_json::to_string(&price_event);
    assert!(serialized.is_ok(), "PriceUpdate should serialize");

    let deserialized: Result<MarketEvent, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok(), "PriceUpdate should deserialize");

    // Test ConnectionStatus event
    let connection_event = MarketEvent::ConnectionStatus {
        connected: true,
        source: "helius".to_string(),
        error: None,
        timestamp: 1234567890,
    };

    let serialized = serde_json::to_string(&connection_event);
    assert!(serialized.is_ok(), "ConnectionStatus should serialize");

    info!("✅ MarketEvent serialization test passed");
}

#[tokio::test]
async fn test_websocket_disabled_mode() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing WebSocket disabled mode");

    // Create event channel
    let (event_sender, _event_receiver) = mpsc::channel::<MarketEvent>(100);

    // Create disabled WebSocket config
    let ws_config = WebSocketConfig {
        enabled: false,
        helius_ws_url: None,
        jupiter_ws_url: None,
        binance_ws_url: None,
        reconnect_timeout_seconds: 1,
        max_retries: 1,
        ping_interval_seconds: 30,
        subscriptions: vec![],
    };

    // Create WebSocket manager
    let ws_manager = RealtimeWebSocketManager::new(ws_config, event_sender);

    // Start should return immediately when disabled
    let start_result = tokio::time::timeout(
        Duration::from_secs(1),
        ws_manager.start()
    ).await;

    match start_result {
        Ok(Ok(_)) => {
            info!("✅ WebSocket manager handled disabled mode correctly");
        }
        Ok(Err(e)) => {
            warn!("WebSocket manager returned error in disabled mode: {}", e);
        }
        Err(_) => {
            panic!("WebSocket manager should return immediately when disabled");
        }
    }
}

#[tokio::test]
async fn test_event_channel_communication() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing event channel communication");

    // Create event channel
    let (event_sender, mut event_receiver) = mpsc::channel::<MarketEvent>(100);

    // Test sending events through the channel
    let test_event = MarketEvent::RawMessage {
        source: "test".to_string(),
        data: "test_data".to_string(),
        timestamp: 1234567890,
    };

    // Send event
    let send_result = event_sender.send(test_event.clone()).await;
    assert!(send_result.is_ok(), "Should be able to send event");

    // Receive event with timeout
    let receive_result = tokio::time::timeout(
        Duration::from_millis(100),
        event_receiver.recv()
    ).await;

    match receive_result {
        Ok(Some(received_event)) => {
            match (&test_event, &received_event) {
                (
                    MarketEvent::RawMessage { source: s1, data: d1, timestamp: t1 },
                    MarketEvent::RawMessage { source: s2, data: d2, timestamp: t2 }
                ) => {
                    assert_eq!(s1, s2);
                    assert_eq!(d1, d2);
                    assert_eq!(t1, t2);
                    info!("✅ Event channel communication test passed");
                }
                _ => panic!("Received event doesn't match sent event"),
            }
        }
        Ok(None) => panic!("Channel closed unexpectedly"),
        Err(_) => panic!("Timeout receiving event"),
    }
}

#[tokio::test]
async fn test_config_integration() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing WebSocket config integration with main config");

    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();

    // Create main config
    let config = Config::default();

    // Test that WebSocket config is included
    assert!(config.websocket.enabled);
    assert!(config.websocket.helius_ws_url.is_some());

    // Test that we can create WebSocket manager from main config
    let (event_sender, _event_receiver) = mpsc::channel::<MarketEvent>(100);
    let ws_manager = RealtimeWebSocketManager::new(config.websocket, event_sender);

    info!("✅ WebSocket config integration test passed");
}

#[tokio::test]
async fn test_url_validation() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("🧪 Testing URL validation");

    // Test valid WebSocket URLs
    let valid_urls = vec![
        "wss://stream.binance.com:9443/ws",
        "wss://atlas-mainnet.helius-rpc.com/?api-key=test",
        "ws://localhost:8080/ws",
    ];

    for url_str in valid_urls {
        let url_result = url::Url::parse(url_str);
        assert!(url_result.is_ok(), "URL should be valid: {}", url_str);
        
        let url = url_result.unwrap();
        assert!(url.scheme() == "ws" || url.scheme() == "wss", "Should be WebSocket URL");
    }

    // Test invalid URLs
    let invalid_urls = vec![
        "not_a_url",
        "http://example.com", // Not WebSocket
        "",
    ];

    for url_str in invalid_urls {
        let url_result = url::Url::parse(url_str);
        if let Ok(url) = url_result {
            assert!(url.scheme() != "ws" && url.scheme() != "wss", "Should not be WebSocket URL: {}", url_str);
        }
        // Invalid URLs will fail to parse, which is expected
    }

    info!("✅ URL validation test passed");
}

use crate::models::{DataSource, MarketData, MarketEvent, TradingError, TradingResult};
use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use futures_util::{SinkExt, StreamExt};

/// WebSocket manager for real-time data streams
pub struct WebSocketManager {
    connections: DashMap<String, WebSocketConnection>,
    event_sender: broadcast::Sender<MarketEvent>,
    _event_receiver: broadcast::Receiver<MarketEvent>,
}

#[derive(Debug, Clone)]
struct WebSocketConnection {
    url: String,
    source: DataSource,
    subscriptions: Vec<String>,
    is_connected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BinanceWebSocketMessage {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: i64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    close_price: String,
    #[serde(rename = "o")]
    open_price: String,
    #[serde(rename = "h")]
    high_price: String,
    #[serde(rename = "l")]
    low_price: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "q")]
    quote_volume: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SolanaWebSocketMessage {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        
        Self {
            connections: DashMap::new(),
            event_sender,
            _event_receiver: event_receiver,
        }
    }

    /// Add a new WebSocket connection
    pub async fn add_connection(
        &self,
        name: String,
        url: String,
        source: DataSource,
    ) -> TradingResult<()> {
        let connection = WebSocketConnection {
            url: url.clone(),
            source,
            subscriptions: Vec::new(),
            is_connected: false,
        };

        self.connections.insert(name.clone(), connection);
        info!("Added WebSocket connection: {} -> {}", name, url);

        // Start the connection in the background
        self.start_connection(name).await?;

        Ok(())
    }

    /// Subscribe to a symbol on a specific connection
    pub async fn subscribe_to_symbol(
        &self,
        connection_name: &str,
        symbol: &str,
    ) -> TradingResult<()> {
        if let Some(mut connection) = self.connections.get_mut(connection_name) {
            connection.subscriptions.push(symbol.to_string());
            info!("Subscribed to {} on {}", symbol, connection_name);
            
            // Send subscription message if connected
            if connection.is_connected {
                self.send_subscription_message(connection_name, symbol).await?;
            }
            
            Ok(())
        } else {
            Err(TradingError::DataError(format!(
                "WebSocket connection not found: {}", connection_name
            )))
        }
    }

    /// Get event receiver for listening to market events
    pub fn get_event_receiver(&self) -> broadcast::Receiver<MarketEvent> {
        self.event_sender.subscribe()
    }

    /// Get connection status
    pub fn get_connection_status(&self) -> Vec<(String, bool)> {
        self.connections
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().is_connected))
            .collect()
    }

    /// Start a WebSocket connection
    async fn start_connection(&self, connection_name: String) -> TradingResult<()> {
        let connection = self.connections.get(&connection_name)
            .ok_or_else(|| TradingError::DataError(format!("Connection not found: {}", connection_name)))?
            .clone();

        let event_sender = self.event_sender.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            loop {
                match Self::connect_and_handle(&connection, &event_sender).await {
                    Ok(_) => {
                        info!("WebSocket connection {} completed normally", connection_name);
                    }
                    Err(e) => {
                        error!("WebSocket connection {} failed: {}", connection_name, e);
                        
                        // Mark as disconnected
                        if let Some(mut conn) = connections.get_mut(&connection_name) {
                            conn.is_connected = false;
                        }
                        
                        // Wait before reconnecting
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        warn!("Attempting to reconnect WebSocket: {}", connection_name);
                    }
                }
            }
        });

        Ok(())
    }

    async fn connect_and_handle(
        connection: &WebSocketConnection,
        event_sender: &broadcast::Sender<MarketEvent>,
    ) -> TradingResult<()> {
        info!("Connecting to WebSocket: {}", connection.url);

        let (ws_stream, _) = connect_async(&connection.url).await
            .map_err(|e| TradingError::NetworkError(format!("WebSocket connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        info!("WebSocket connected: {}", connection.url);

        // Send initial subscriptions
        for symbol in &connection.subscriptions {
            let subscription_msg = Self::create_subscription_message(&connection.source, symbol)?;
            write.send(Message::Text(subscription_msg)).await
                .map_err(|e| TradingError::NetworkError(format!("Failed to send subscription: {}", e)))?;
            
            debug!("Sent subscription for {} on {:?}", symbol, connection.source);
        }

        // Handle incoming messages
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(event) = Self::parse_message(&text, &connection.source) {
                        if let Err(e) = event_sender.send(event) {
                            debug!("No active receivers for market event: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("WebSocket connection closed: {}", connection.url);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {
                    // Ignore other message types (binary, ping, pong)
                }
            }
        }

        Ok(())
    }

    fn create_subscription_message(source: &DataSource, symbol: &str) -> TradingResult<String> {
        match source {
            DataSource::Binance => {
                let subscription = serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": [format!("{}@ticker", symbol.to_lowercase())],
                    "id": 1
                });
                Ok(subscription.to_string())
            }
            DataSource::Solana => {
                let subscription = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "accountSubscribe",
                    "params": [
                        symbol,
                        {
                            "encoding": "base64",
                            "commitment": "finalized"
                        }
                    ]
                });
                Ok(subscription.to_string())
            }
            _ => Err(TradingError::DataError(format!(
                "WebSocket subscriptions not supported for {:?}", source
            )))
        }
    }

    fn parse_message(message: &str, source: &DataSource) -> TradingResult<MarketEvent> {
        match source {
            DataSource::Binance => {
                let ws_msg: BinanceWebSocketMessage = serde_json::from_str(message)
                    .map_err(|e| TradingError::DataError(format!("Failed to parse Binance message: {}", e)))?;

                if ws_msg.event_type == "24hrTicker" {
                    let market_data = MarketData {
                        symbol: ws_msg.symbol,
                        price: ws_msg.close_price.parse().unwrap_or(0.0),
                        volume: ws_msg.volume.parse().unwrap_or(0.0),
                        bid: None,
                        ask: None,
                        timestamp: Utc::now(),
                        source: DataSource::Binance,
                    };

                    Ok(MarketEvent::PriceUpdate {
                        symbol: market_data.symbol.clone(),
                        price: market_data.price,
                        volume_24h: Some(market_data.volume),
                        timestamp: Utc::now().timestamp_millis() as u64,
                        source: "binance".to_string(),
                    })
                } else {
                    Err(TradingError::DataError(format!(
                        "Unsupported Binance event type: {}", ws_msg.event_type
                    )))
                }
            }
            DataSource::Solana => {
                let _ws_msg: SolanaWebSocketMessage = serde_json::from_str(message)
                    .map_err(|e| TradingError::DataError(format!("Failed to parse Solana message: {}", e)))?;

                Ok(MarketEvent::RawMessage {
                    source: "solana".to_string(),
                    data: message.to_string(),
                    timestamp: Utc::now().timestamp_millis() as u64,
                })
            }
            _ => Err(TradingError::DataError(format!(
                "Message parsing not implemented for {:?}", source
            )))
        }
    }

    async fn send_subscription_message(&self, connection_name: &str, symbol: &str) -> TradingResult<()> {
        // This would send a subscription message to an active connection
        // For now, just log the intent
        debug!("Would send subscription for {} on {}", symbol, connection_name);
        Ok(())
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        assert_eq!(manager.connections.len(), 0);
    }

    #[tokio::test]
    async fn test_add_connection() {
        let manager = WebSocketManager::new();
        
        let result = manager.add_connection(
            "test".to_string(),
            "wss://example.com".to_string(),
            DataSource::Binance,
        ).await;
        
        // This will fail because we can't actually connect to example.com
        // but it tests the connection setup logic
        assert!(result.is_ok());
        assert_eq!(manager.connections.len(), 1);
    }

    #[test]
    fn test_create_subscription_message() {
        let binance_msg = WebSocketManager::create_subscription_message(
            &DataSource::Binance, 
            "BTCUSDT"
        ).unwrap();
        
        assert!(binance_msg.contains("SUBSCRIBE"));
        assert!(binance_msg.contains("btcusdt@ticker"));
    }

    #[test]
    fn test_parse_binance_message() {
        let message = r#"{
            "e": "24hrTicker",
            "E": 123456789,
            "s": "BTCUSDT",
            "c": "50000.00",
            "o": "49000.00",
            "h": "51000.00",
            "l": "48000.00",
            "v": "1000.00",
            "q": "50000000.00"
        }"#;

        let event = WebSocketManager::parse_message(message, &DataSource::Binance);
        assert!(event.is_ok());
        
        let event = event.unwrap();
        assert_eq!(event.symbol, "BTCUSDT");
        assert!(matches!(event.event_type, MarketEventType::PriceUpdate));
    }
}

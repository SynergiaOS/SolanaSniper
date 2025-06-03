use crate::models::{MarketEvent, TransactionType, TradingError, TradingResult, WebSocketConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Real-time WebSocket Manager for SniperBot
/// Handles multiple WebSocket connections for different data sources
pub struct RealtimeWebSocketManager {
    config: WebSocketConfig,
    event_sender: mpsc::Sender<MarketEvent>,
    is_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl RealtimeWebSocketManager {
    /// Create new WebSocket manager
    pub fn new(config: WebSocketConfig, event_sender: mpsc::Sender<MarketEvent>) -> Self {
        Self {
            config,
            event_sender,
            is_running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start all WebSocket connections
    pub async fn start(&self) -> TradingResult<()> {
        if !self.config.enabled {
            info!("WebSocket manager is disabled in config");
            return Ok(());
        }

        self.is_running.store(true, std::sync::atomic::Ordering::SeqCst);
        info!("🚀 Starting RealtimeWebSocketManager");

        let mut handles = Vec::new();

        // Start Helius WebSocket if configured
        if let Some(helius_url) = &self.config.helius_ws_url {
            let handle = self.start_helius_connection(helius_url.clone()).await?;
            handles.push(handle);
        }

        // Start Binance WebSocket if configured
        if let Some(binance_url) = &self.config.binance_ws_url {
            let handle = self.start_binance_connection(binance_url.clone()).await?;
            handles.push(handle);
        }

        // Wait for all connections to complete (they should run indefinitely)
        for handle in handles {
            if let Err(e) = handle.await {
                error!("WebSocket connection task failed: {:?}", e);
            }
        }

        Ok(())
    }

    /// Stop all WebSocket connections
    pub fn stop(&self) {
        info!("🛑 Stopping RealtimeWebSocketManager");
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Start Helius WebSocket connection
    async fn start_helius_connection(&self, url: String) -> TradingResult<tokio::task::JoinHandle<()>> {
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();

        let handle = tokio::spawn(async move {
            let mut retries = 0;
            
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                if retries >= config.max_retries && config.max_retries > 0 {
                    error!("Max Helius WebSocket retries reached");
                    break;
                }

                info!("Connecting to Helius WebSocket: {}", url);
                
                match Self::connect_and_listen_helius(&url, &event_sender, &config).await {
                    Ok(_) => {
                        info!("Helius WebSocket connection closed gracefully");
                        retries = 0;
                    }
                    Err(e) => {
                        error!("Helius WebSocket error: {:?}", e);
                        retries += 1;
                        
                        // Send connection status event
                        let _ = event_sender.send(MarketEvent::ConnectionStatus {
                            connected: false,
                            source: "helius".to_string(),
                            error: Some(e.to_string()),
                            timestamp: Self::current_timestamp(),
                        }).await;
                        
                        sleep(Duration::from_secs(config.reconnect_timeout_seconds)).await;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Start Binance WebSocket connection
    async fn start_binance_connection(&self, url: String) -> TradingResult<tokio::task::JoinHandle<()>> {
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();

        let handle = tokio::spawn(async move {
            let mut retries = 0;
            
            while is_running.load(std::sync::atomic::Ordering::SeqCst) {
                if retries >= config.max_retries && config.max_retries > 0 {
                    error!("Max Binance WebSocket retries reached");
                    break;
                }

                info!("Connecting to Binance WebSocket: {}", url);
                
                match Self::connect_and_listen_binance(&url, &event_sender, &config).await {
                    Ok(_) => {
                        info!("Binance WebSocket connection closed gracefully");
                        retries = 0;
                    }
                    Err(e) => {
                        error!("Binance WebSocket error: {:?}", e);
                        retries += 1;
                        
                        // Send connection status event
                        let _ = event_sender.send(MarketEvent::ConnectionStatus {
                            connected: false,
                            source: "binance".to_string(),
                            error: Some(e.to_string()),
                            timestamp: Self::current_timestamp(),
                        }).await;
                        
                        sleep(Duration::from_secs(config.reconnect_timeout_seconds)).await;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Connect and listen to Helius WebSocket
    async fn connect_and_listen_helius(
        url: &str,
        event_sender: &mpsc::Sender<MarketEvent>,
        config: &WebSocketConfig,
    ) -> TradingResult<()> {
        // Substitute API key in URL
        let resolved_url = url.replace("${HELIUS_API_KEY}", 
            &std::env::var("HELIUS_API_KEY").unwrap_or_default());
        
        let (ws_stream, _) = connect_async(&resolved_url)
            .await
            .map_err(|e| TradingError::NetworkError(format!("Failed to connect to Helius: {}", e)))?;

        info!("✅ Connected to Helius WebSocket");
        
        // Send connection status
        let _ = event_sender.send(MarketEvent::ConnectionStatus {
            connected: true,
            source: "helius".to_string(),
            error: None,
            timestamp: Self::current_timestamp(),
        }).await;

        let (mut write, mut read) = ws_stream.split();

        // Subscribe to account updates (example)
        let subscribe_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "accountSubscribe",
            "params": [
                "11111111111111111111111111111111", // Example account
                {
                    "encoding": "jsonParsed",
                    "commitment": "finalized"
                }
            ]
        }).to_string();

        write.send(tokio_tungstenite::tungstenite::Message::Text(subscribe_msg))
            .await
            .map_err(|e| TradingError::NetworkError(format!("Failed to send subscription: {}", e)))?;

        // Listen for messages
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    if msg.is_text() {
                        let text = msg.into_text().unwrap();
                        
                        if let Ok(parsed_event) = Self::parse_helius_message(&text) {
                            let _ = event_sender.send(parsed_event).await;
                        } else {
                            // Send raw message for debugging
                            let _ = event_sender.send(MarketEvent::RawMessage {
                                source: "helius".to_string(),
                                data: text,
                                timestamp: Self::current_timestamp(),
                            }).await;
                        }
                    } else if msg.is_ping() {
                        let _ = write.send(tokio_tungstenite::tungstenite::Message::Pong(vec![])).await;
                    }
                }
                Err(e) => {
                    return Err(TradingError::NetworkError(format!("Helius WebSocket error: {}", e)));
                }
            }
        }

        Ok(())
    }

    /// Connect and listen to Binance WebSocket
    async fn connect_and_listen_binance(
        url: &str,
        event_sender: &mpsc::Sender<MarketEvent>,
        _config: &WebSocketConfig,
    ) -> TradingResult<()> {
        // For Binance, we'll connect to SOL/USDT ticker
        let binance_url = format!("{}/solusdt@ticker", url);
        
        let (ws_stream, _) = connect_async(&binance_url)
            .await
            .map_err(|e| TradingError::NetworkError(format!("Failed to connect to Binance: {}", e)))?;

        info!("✅ Connected to Binance WebSocket");
        
        // Send connection status
        let _ = event_sender.send(MarketEvent::ConnectionStatus {
            connected: true,
            source: "binance".to_string(),
            error: None,
            timestamp: Self::current_timestamp(),
        }).await;

        let (_write, mut read) = ws_stream.split();

        // Listen for messages
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => {
                    if msg.is_text() {
                        let text = msg.into_text().unwrap();
                        
                        if let Ok(parsed_event) = Self::parse_binance_message(&text) {
                            let _ = event_sender.send(parsed_event).await;
                        } else {
                            // Send raw message for debugging
                            let _ = event_sender.send(MarketEvent::RawMessage {
                                source: "binance".to_string(),
                                data: text,
                                timestamp: Self::current_timestamp(),
                            }).await;
                        }
                    }
                }
                Err(e) => {
                    return Err(TradingError::NetworkError(format!("Binance WebSocket error: {}", e)));
                }
            }
        }

        Ok(())
    }

    /// Parse Helius WebSocket message
    fn parse_helius_message(text: &str) -> TradingResult<MarketEvent> {
        let json: Value = serde_json::from_str(text)
            .map_err(|e| TradingError::DataError(format!("Failed to parse Helius JSON: {}", e)))?;

        // Example parsing - adjust based on actual Helius message format
        if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
            match method {
                "accountNotification" => {
                    // Parse account update
                    Ok(MarketEvent::NewTransaction {
                        signature: "helius_account_update".to_string(),
                        token_address: "unknown".to_string(),
                        amount: 0.0,
                        price: None,
                        transaction_type: TransactionType::Unknown,
                        timestamp: Self::current_timestamp(),
                    })
                }
                _ => Err(TradingError::DataError(format!("Unknown Helius method: {}", method)))
            }
        } else {
            Err(TradingError::DataError("No method in Helius message".to_string()))
        }
    }

    /// Parse Binance WebSocket message
    fn parse_binance_message(text: &str) -> TradingResult<MarketEvent> {
        let json: Value = serde_json::from_str(text)
            .map_err(|e| TradingError::DataError(format!("Failed to parse Binance JSON: {}", e)))?;

        // Parse Binance ticker message
        if let (Some(symbol), Some(price_str)) = (
            json.get("s").and_then(|s| s.as_str()),
            json.get("c").and_then(|c| c.as_str())
        ) {
            let price = price_str.parse::<f64>()
                .map_err(|e| TradingError::DataError(format!("Invalid price: {}", e)))?;
            
            let volume_24h = json.get("v")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse::<f64>().ok());

            Ok(MarketEvent::PriceUpdate {
                symbol: symbol.to_string(),
                price,
                volume_24h,
                timestamp: Self::current_timestamp(),
                source: "binance".to_string(),
            })
        } else {
            Err(TradingError::DataError("Invalid Binance ticker format".to_string()))
        }
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

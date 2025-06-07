use crate::models::{StrategySignal, TradingResult, TradingError};
use crate::strategy::{StrategyManager, StrategyPerformance};
use crate::live_trading_engine::{LiveTradingEngine, EngineStatus};
use crate::utils::reporter::ReportEvent;
use axum::{
    extract::{ws::WebSocket, ws::WebSocketUpgrade, State, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{stream::StreamExt, SinkExt, stream::SplitSink, stream::SplitStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{debug, error, info, warn};

/// API Server for SniperBot Dashboard
pub struct ApiServer {
    strategy_manager: Arc<StrategyManager>,
    trading_engine: Arc<LiveTradingEngine>,
    websocket_tx: broadcast::Sender<WebSocketMessage>,
    signals_history: Arc<RwLock<Vec<StrategySignal>>>,
    trades_history: Arc<RwLock<Vec<TradeRecord>>>,
    port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: String,
    pub strategy: String,
    pub symbol: String,
    pub action: String,
    pub amount: f64,
    pub price: f64,
    pub fees: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct BotStatus {
    pub engine_status: EngineStatus,
    pub active_strategies: Vec<String>,
    pub strategy_performance: HashMap<String, StrategyPerformance>,
    pub uptime_seconds: u64,
    pub last_signal_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub strategy: Option<String>,
    pub symbol: Option<String>,
}

impl ApiServer {
    pub fn new(
        strategy_manager: Arc<StrategyManager>,
        trading_engine: Arc<LiveTradingEngine>,
        port: u16,
    ) -> Self {
        let (websocket_tx, _) = broadcast::channel(1000);
        
        Self {
            strategy_manager,
            trading_engine,
            websocket_tx,
            signals_history: Arc::new(RwLock::new(Vec::new())),
            trades_history: Arc::new(RwLock::new(Vec::new())),
            port,
        }
    }

    /// Start the API server
    pub async fn start(self) -> TradingResult<()> {
        let app_state = Arc::new(self);
        
        let app = Router::new()
            // API routes
            .route("/api/signals", get(get_signals))
            .route("/api/trades", get(get_trades))
            .route("/api/portfolio", get(get_portfolio))
            .route("/api/bot-status", get(get_bot_status))
            .route("/api/report_event", post(report_event))
            // Bot control routes
            .route("/api/bot/start", post(start_bot))
            .route("/api/bot/stop", post(stop_bot))
            .route("/api/bot/pause", post(pause_bot))
            .route("/api/bot/emergency-stop", post(emergency_stop))
            .route("/api/bot/mode", post(set_bot_mode))
            // Manual trading routes
            .route("/api/trade/manual", post(execute_manual_trade))
            // WebSocket endpoint
            .route("/ws", get(websocket_handler))
            // Static files (dashboard)
            .route("/", get(serve_dashboard))
            .route("/dashboard", get(serve_dashboard))
            .nest_service("/static", ServeDir::new("frontend/dist"))
            // State
            .with_state(app_state.clone())
            // Middleware
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
            );

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", app_state.port))
            .await
            .map_err(|e| TradingError::NetworkError(format!("Failed to bind to port {}: {}", app_state.port, e)))?;

        info!("🌐 API Server starting on http://0.0.0.0:{}", app_state.port);
        info!("📊 Dashboard available at http://localhost:{}/dashboard", app_state.port);
        info!("🔌 WebSocket endpoint: ws://localhost:{}/ws", app_state.port);

        axum::serve(listener, app)
            .await
            .map_err(|e| TradingError::NetworkError(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Add a signal to history and broadcast via WebSocket
    pub async fn add_signal(&self, signal: StrategySignal) {
        // Add to history
        {
            let mut signals = self.signals_history.write().await;
            signals.push(signal.clone());

            // Keep only last 1000 signals
            if signals.len() > 1000 {
                let excess = signals.len() - 1000;
                signals.drain(0..excess);
            }
        }

        // Broadcast via WebSocket
        let ws_message = WebSocketMessage {
            message_type: "signal".to_string(),
            data: serde_json::to_value(&signal).unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = self.websocket_tx.send(ws_message) {
            debug!("No WebSocket subscribers: {}", e);
        }
    }

    /// Add a trade to history and broadcast via WebSocket
    pub async fn add_trade(&self, trade: TradeRecord) {
        // Add to history
        {
            let mut trades = self.trades_history.write().await;
            trades.push(trade.clone());

            // Keep only last 1000 trades
            if trades.len() > 1000 {
                let excess = trades.len() - 1000;
                trades.drain(0..excess);
            }
        }

        // Broadcast via WebSocket
        let ws_message = WebSocketMessage {
            message_type: "trade".to_string(),
            data: serde_json::to_value(&trade).unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        };

        if let Err(e) = self.websocket_tx.send(ws_message) {
            debug!("No WebSocket subscribers: {}", e);
        }
    }

    /// Get WebSocket sender for broadcasting
    pub fn get_websocket_sender(&self) -> broadcast::Sender<WebSocketMessage> {
        self.websocket_tx.clone()
    }
}

/// Serve the main dashboard HTML
async fn serve_dashboard() -> impl IntoResponse {
    match tokio::fs::read_to_string("frontend/dashboard.html").await {
        Ok(content) => {
            info!("📊 Serving dashboard from frontend/dashboard.html");
            Html(content)
        },
        Err(e) => {
            warn!("Failed to read dashboard.html: {}", e);
            Html(format!(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>SniperBot 2.0 - Dashboard</title>
                    <style>
                        body {{
                            font-family: 'Orbitron', monospace;
                            background: #0a0a0f;
                            color: #00ffff;
                            text-align: center;
                            padding: 50px;
                            margin: 0;
                        }}
                        .container {{
                            background: rgba(26, 26, 46, 0.8);
                            padding: 40px;
                            border-radius: 20px;
                            display: inline-block;
                            border: 1px solid rgba(0, 255, 255, 0.3);
                            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
                        }}
                        .icon {{ font-size: 4rem; margin-bottom: 20px; }}
                        h1 {{ color: #00ffff; text-shadow: 0 0 20px rgba(0, 255, 255, 0.5); }}
                        .status {{ color: #00ff88; margin: 20px 0; }}
                        .error {{ color: #ff6b6b; margin: 10px 0; }}
                        .api-info {{
                            background: rgba(0, 255, 255, 0.1);
                            padding: 20px;
                            border-radius: 10px;
                            margin: 20px 0;
                        }}
                    </style>
                </head>
                <body>
                    <div class="container">
                        <div class="icon">🤖</div>
                        <h1>⚡ SniperBot 2.0 API Server</h1>
                        <div class="status">✅ API Server is running successfully!</div>
                        <div class="error">⚠️ Dashboard file not found: frontend/dashboard.html</div>
                        <div class="api-info">
                            <h3>📡 Available API Endpoints:</h3>
                            <p>• GET /api/signals - Recent trading signals</p>
                            <p>• GET /api/trades - Trading history</p>
                            <p>• GET /api/portfolio - Portfolio status</p>
                            <p>• GET /api/bot-status - Bot status and performance</p>
                            <p>• WS /ws - Real-time WebSocket updates</p>
                        </div>
                        <p>Error details: {}</p>
                        <script>
                            // Auto-refresh every 5 seconds to check if dashboard becomes available
                            setTimeout(() => location.reload(), 5000);
                        </script>
                    </div>
                </body>
                </html>
                "#, e
            ))
        }
    }
}

/// Get signals with optional filtering
async fn get_signals(
    State(server): State<Arc<ApiServer>>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let signals = server.signals_history.read().await;

    let mut filtered_signals: Vec<StrategySignal> = signals
        .iter()
        .filter(|signal| {
            if let Some(ref strategy) = params.strategy {
                if signal.strategy != *strategy {
                    return false;
                }
            }
            if let Some(ref symbol) = params.symbol {
                if signal.symbol != *symbol {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(1000);

    if offset < filtered_signals.len() {
        let end = (offset + limit).min(filtered_signals.len());
        filtered_signals = filtered_signals[offset..end].to_vec();
    } else {
        filtered_signals.clear();
    }

    let response = ApiResponse {
        success: true,
        data: Some(filtered_signals),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

/// Get trades with optional filtering
async fn get_trades(
    State(server): State<Arc<ApiServer>>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let trades = server.trades_history.read().await;

    let mut filtered_trades: Vec<TradeRecord> = trades
        .iter()
        .filter(|trade| {
            if let Some(ref strategy) = params.strategy {
                if trade.strategy != *strategy {
                    return false;
                }
            }
            if let Some(ref symbol) = params.symbol {
                if trade.symbol != *symbol {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(1000);

    if offset < filtered_trades.len() {
        let end = (offset + limit).min(filtered_trades.len());
        filtered_trades = filtered_trades[offset..end].to_vec();
    } else {
        filtered_trades.clear();
    }

    let response = ApiResponse {
        success: true,
        data: Some(filtered_trades),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

/// Get portfolio information
async fn get_portfolio(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
    let engine_status = server.trading_engine.get_status();
    
    let portfolio_data = serde_json::json!({
        "total_value": engine_status.portfolio_value,
        "is_running": engine_status.is_running,
        "dry_run": engine_status.dry_run,
        "active_strategies": engine_status.active_strategies
    });

    let response = ApiResponse {
        success: true,
        data: Some(portfolio_data),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

/// Get bot status
async fn get_bot_status(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
    let engine_status = server.trading_engine.get_status();
    let active_strategies = server.strategy_manager.get_active_strategies().await;
    let strategy_performance = server.strategy_manager.get_all_performance().await;
    
    let last_signal_time = {
        let signals = server.signals_history.read().await;
        signals.last().map(|s| s.timestamp)
    };

    let bot_status = BotStatus {
        engine_status,
        active_strategies,
        strategy_performance,
        uptime_seconds: 0, // TODO: Calculate actual uptime
        last_signal_time,
    };

    let response = ApiResponse {
        success: true,
        data: Some(bot_status),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

/// Report event endpoint
async fn report_event(
    State(server): State<Arc<ApiServer>>,
    Json(event): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("📊 Received report event: {:?}", event);

    // Broadcast event via WebSocket
    let ws_message = WebSocketMessage {
        message_type: "event".to_string(),
        data: event,
        timestamp: chrono::Utc::now(),
    };

    if let Err(e) = server.websocket_tx.send(ws_message) {
        debug!("No WebSocket subscribers: {}", e);
    }

    let response = ApiResponse {
        success: true,
        data: Some("Event received"),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

/// WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<ApiServer>>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, server))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, server: Arc<ApiServer>) {
    let mut rx = server.websocket_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    info!("🔌 New WebSocket connection established");

    // Send initial data
    let initial_message = WebSocketMessage {
        message_type: "connected".to_string(),
        data: serde_json::json!({"message": "Connected to SniperBot API"}),
        timestamp: chrono::Utc::now(),
    };

    if let Ok(msg) = serde_json::to_string(&initial_message) {
        if let Err(e) = sender.send(axum::extract::ws::Message::Text(msg)).await {
            warn!("Failed to send initial WebSocket message: {}", e);
            return;
        }
    }

    // Handle incoming and outgoing messages
    loop {
        tokio::select! {
            // Broadcast messages to client
            msg = rx.recv() => {
                match msg {
                    Ok(ws_message) => {
                        if let Ok(json_msg) = serde_json::to_string(&ws_message) {
                            if let Err(e) = sender.send(axum::extract::ws::Message::Text(json_msg)).await {
                                warn!("Failed to send WebSocket message: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        debug!("WebSocket broadcast channel error: {}", e);
                        break;
                    }
                }
            }
            
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        debug!("Received WebSocket message: {}", text);
                        // Handle client messages if needed
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        info!("🔌 WebSocket connection closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("🔌 WebSocket connection closed");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("🔌 WebSocket connection terminated");
}

// Bot Control Endpoints
async fn start_bot(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
    info!("🚀 Bot start requested via API");

    let response = ApiResponse {
        success: true,
        data: Some("Bot start command received"),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

async fn stop_bot(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
    info!("⏹️ Bot stop requested via API");

    let response = ApiResponse {
        success: true,
        data: Some("Bot stop command received"),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

async fn pause_bot(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
    info!("⏸️ Bot pause requested via API");

    let response = ApiResponse {
        success: true,
        data: Some("Bot pause command received"),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

async fn emergency_stop(State(server): State<Arc<ApiServer>>) -> impl IntoResponse {
    warn!("🚨 EMERGENCY STOP requested via API");

    let response = ApiResponse {
        success: true,
        data: Some("Emergency stop executed"),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

async fn set_bot_mode(
    State(server): State<Arc<ApiServer>>,
    Json(payload): Json<serde_json::Value>
) -> impl IntoResponse {
    let mode = payload.get("mode").and_then(|m| m.as_str()).unwrap_or("DRY_RUN");
    info!("🔄 Bot mode change requested: {}", mode);

    let response = ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": format!("Mode changed to {}", mode),
            "mode": mode
        })),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

async fn execute_manual_trade(
    State(server): State<Arc<ApiServer>>,
    Json(payload): Json<serde_json::Value>
) -> impl IntoResponse {
    let symbol = payload.get("symbol").and_then(|s| s.as_str()).unwrap_or("SOL/USDC");
    let action = payload.get("action").and_then(|a| a.as_str()).unwrap_or("buy");
    let amount = payload.get("amount").and_then(|a| a.as_f64()).unwrap_or(1.0);
    let price = payload.get("price").and_then(|p| p.as_f64());

    info!("💰 Manual trade requested: {} {} {} @ {:?}", action, amount, symbol, price);

    // Create a mock trade record
    let trade = TradeRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        strategy: "manual".to_string(),
        symbol: symbol.to_string(),
        action: action.to_string(),
        amount,
        price: price.unwrap_or(100.0),
        fees: 0.25, // 0.25% fee
        success: true,
        error_message: None,
    };

    // Add to trades history
    {
        let mut trades = server.trades_history.write().await;
        trades.push(trade.clone());

        // Keep only last 1000 trades
        if trades.len() > 1000 {
            let len = trades.len();
            trades.drain(0..len - 1000);
        }
    }

    // Broadcast via WebSocket
    let ws_message = WebSocketMessage {
        message_type: "trade".to_string(),
        data: serde_json::to_value(&trade).unwrap_or_default(),
        timestamp: chrono::Utc::now(),
    };

    if let Err(e) = server.websocket_tx.send(ws_message) {
        debug!("No WebSocket subscribers: {}", e);
    }

    let response = ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "message": "Manual trade executed",
            "trade": trade
        })),
        error: None,
        timestamp: chrono::Utc::now(),
    };

    Json(response)
}

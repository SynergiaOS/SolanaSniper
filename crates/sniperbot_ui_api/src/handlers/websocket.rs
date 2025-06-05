use axum::{
    extract::{ws::WebSocket, ws::WebSocketUpgrade, State},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn, error};
use crate::models::AppState;

/// WebSocket handler for real-time updates
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    info!("🔌 WebSocket connection request received");
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let mut rx = state.ws_tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    info!("🔌 New WebSocket connection established");

    // Send initial connection message
    let initial_message = serde_json::json!({
        "type": "connected",
        "message": "Connected to SniperBot API",
        "timestamp": chrono::Utc::now()
    });

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
                    Ok(json_msg) => {
                        if let Err(e) = sender.send(axum::extract::ws::Message::Text(json_msg)).await {
                            warn!("Failed to send WebSocket message: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("WebSocket broadcast channel error: {}", e);
                        break;
                    }
                }
            }
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        info!("📨 Received WebSocket message: {}", text);
                        // Echo back for now (can be extended for client commands)
                        let response = serde_json::json!({
                            "type": "echo",
                            "data": text,
                            "timestamp": chrono::Utc::now()
                        });
                        if let Ok(response_str) = serde_json::to_string(&response) {
                            if let Err(e) = sender.send(axum::extract::ws::Message::Text(response_str)).await {
                                warn!("Failed to send echo response: {}", e);
                                break;
                            }
                        }
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        info!("🔌 WebSocket connection closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("🔌 WebSocket connection closed");
                        break;
                    }
                    _ => {
                        // Ignore other message types (binary, ping, pong)
                    }
                }
            }
        }
    }

    info!("🔌 WebSocket connection terminated");
}

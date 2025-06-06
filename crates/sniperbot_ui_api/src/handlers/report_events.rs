use axum::{extract::State, response::IntoResponse, Json};
use tracing::{info, error};
use serde::{Deserialize, Serialize};
use crate::models::ReportEvent;
use crate::models::AppState;

/// Batch payload structure that Reporter sends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEventPayload {
    pub events: Vec<ReportEvent>,
    pub timestamp: String,
    pub source: String,
}

/// POST /api/report_event - Endpoint to receive events from SniperBot core
/// Supports both single events and batch events from Reporter
pub async fn receive_report_event(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Try to parse as batch payload first
    if let Ok(batch_payload) = serde_json::from_value::<BatchEventPayload>(payload.clone()) {
        info!("📨 Received batch of {} events from {}", batch_payload.events.len(), batch_payload.source);

        // Store all events in our state
        let mut events_store = state.report_events.lock().await;
        for event in &batch_payload.events {
            events_store.push(event.clone());

            // Broadcast each event to WebSocket clients
            if let Ok(json_event) = serde_json::to_string(event) {
                let _ = state.ws_tx.send(json_event); // Ignore error if no receivers
            }
        }

        return format!("Batch of {} events received successfully", batch_payload.events.len());
    }

    // Try to parse as single event
    if let Ok(event) = serde_json::from_value::<ReportEvent>(payload) {
        info!("📨 Received single report event: {:?}", event);

        // Store the event in our state
        state.report_events.lock().await.push(event.clone());

        // Broadcast to WebSocket clients if any are connected
        if let Ok(json_event) = serde_json::to_string(&event) {
            let _ = state.ws_tx.send(json_event); // Ignore error if no receivers
        }

        return "Single event received successfully".to_string();
    }

    error!("❌ Failed to parse payload as either batch or single event");
    "Invalid payload format".to_string()
}

/// GET /api/events - Get all stored events
pub async fn get_all_events(State(state): State<AppState>) -> Json<Vec<ReportEvent>> {
    let events = state.report_events.lock().await;
    info!("📊 GET /api/events - returning {} events", events.len());
    Json(events.clone())
}

/// GET /api/signals - Get signals from stored events (dashboard compatibility)
pub async fn get_signals(State(state): State<AppState>) -> Json<serde_json::Value> {
    let events = state.report_events.lock().await;

    // Filter for SignalGenerated events and convert to dashboard format
    let signals: Vec<serde_json::Value> = events
        .iter()
        .filter_map(|event| {
            if let ReportEvent::SignalGenerated {
                strategy,
                symbol,
                signal_type,
                strength,
                metadata,
                timestamp
            } = event {
                Some(serde_json::json!({
                    "strategy": strategy,
                    "symbol": symbol,
                    "signal_type": signal_type,
                    "strength": strength,
                    "price": 100.0, // Mock price for now
                    "timestamp": timestamp.to_rfc3339(),
                    "metadata": metadata
                }))
            } else {
                None
            }
        })
        .collect();

    Json(serde_json::json!({
        "success": true,
        "data": signals
    }))
}

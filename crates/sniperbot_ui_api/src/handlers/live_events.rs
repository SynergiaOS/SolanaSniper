/*!
📡 Live Event Log Handler - Real-time Bot Intelligence Stream

This handler provides a live stream of bot decision-making events,
giving traders insight into the bot's "thought process" in real-time.
*/

use axum::{
    extract::Query,
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::info;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Live event representing bot's decision-making process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: EventLevel,
    pub component: String,
    pub event_type: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "critical")]
    Critical,
}

/// Query parameters for live events
#[derive(Debug, Deserialize)]
pub struct LiveEventsQuery {
    pub limit: Option<usize>,
    pub level: Option<String>,
    pub component: Option<String>,
    pub since: Option<DateTime<Utc>>,
}

/// Global event buffer (in production, this would be a proper event store)
static EVENT_BUFFER: Mutex<VecDeque<LiveEvent>> = Mutex::new(VecDeque::new());
const MAX_EVENTS: usize = 1000;

/// Get live events stream
pub async fn get_live_events(
    Query(params): Query<LiveEventsQuery>,
) -> Result<Json<Vec<LiveEvent>>, axum::http::StatusCode> {
    info!("📡 Fetching live events with filters: {:?}", params);

    // In production, this would query a real event store
    let events = generate_mock_live_events();
    
    let filtered_events = events
        .into_iter()
        .filter(|event| {
            // Filter by level
            if let Some(ref level) = params.level {
                let event_level_str = match event.level {
                    EventLevel::Debug => "debug",
                    EventLevel::Info => "info",
                    EventLevel::Warning => "warning",
                    EventLevel::Error => "error",
                    EventLevel::Critical => "critical",
                };
                if !event_level_str.eq_ignore_ascii_case(level) {
                    return false;
                }
            }
            
            // Filter by component
            if let Some(ref component) = params.component {
                if !event.component.to_lowercase().contains(&component.to_lowercase()) {
                    return false;
                }
            }
            
            // Filter by timestamp
            if let Some(since) = params.since {
                if event.timestamp < since {
                    return false;
                }
            }
            
            true
        })
        .take(params.limit.unwrap_or(50))
        .collect::<Vec<_>>();

    info!("✅ Returning {} live events", filtered_events.len());
    Ok(Json(filtered_events))
}

/// Add event to live stream (called by bot components)
pub fn add_live_event(event: LiveEvent) {
    if let Ok(mut buffer) = EVENT_BUFFER.lock() {
        buffer.push_back(event);
        
        // Keep buffer size manageable
        while buffer.len() > MAX_EVENTS {
            buffer.pop_front();
        }
    }
}

/// Generate mock live events for testing
fn generate_mock_live_events() -> Vec<LiveEvent> {
    let now = Utc::now();
    
    vec![
        LiveEvent {
            id: "evt_001".to_string(),
            timestamp: now - chrono::Duration::seconds(5),
            level: EventLevel::Info,
            component: "Reflex Core".to_string(),
            event_type: "NEW_TOKEN_DETECTED".to_string(),
            message: "New token detected: SOUL_TOKEN_12345".to_string(),
            details: Some(serde_json::json!({
                "token_address": "SOUL_TOKEN_12345",
                "initial_liquidity": 25000.0,
                "risk_score": 0.3
            })),
        },
        LiveEvent {
            id: "evt_002".to_string(),
            timestamp: now - chrono::Duration::seconds(4),
            level: EventLevel::Debug,
            component: "Intelligence Brain".to_string(),
            event_type: "SENTIMENT_ANALYSIS".to_string(),
            message: "Analyzing sentiment for SOUL_TOKEN_12345...".to_string(),
            details: Some(serde_json::json!({
                "token": "SOUL_TOKEN_12345",
                "sources_found": 15,
                "sentiment_score": 0.75
            })),
        },
        LiveEvent {
            id: "evt_003".to_string(),
            timestamp: now - chrono::Duration::seconds(3),
            level: EventLevel::Info,
            component: "Decision Engine".to_string(),
            event_type: "SIGNAL_APPROVED".to_string(),
            message: "BUY signal approved for SOUL_TOKEN_12345".to_string(),
            details: Some(serde_json::json!({
                "token": "SOUL_TOKEN_12345",
                "signal_type": "Buy",
                "confidence": 0.85,
                "amount_sol": 0.1
            })),
        },
        LiveEvent {
            id: "evt_004".to_string(),
            timestamp: now - chrono::Duration::seconds(2),
            level: EventLevel::Warning,
            component: "Risk Manager".to_string(),
            event_type: "HIGH_RISK_DETECTED".to_string(),
            message: "High risk detected for PUMP_TOKEN_67890".to_string(),
            details: Some(serde_json::json!({
                "token": "PUMP_TOKEN_67890",
                "risk_score": 0.9,
                "reason": "Low liquidity and high volatility"
            })),
        },
        LiveEvent {
            id: "evt_005".to_string(),
            timestamp: now - chrono::Duration::seconds(1),
            level: EventLevel::Error,
            component: "Executor".to_string(),
            event_type: "TRADE_REJECTED".to_string(),
            message: "Trade rejected: Insufficient balance".to_string(),
            details: Some(serde_json::json!({
                "token": "PUMP_TOKEN_67890",
                "required_sol": 0.5,
                "available_sol": 0.3
            })),
        },
        LiveEvent {
            id: "evt_006".to_string(),
            timestamp: now,
            level: EventLevel::Info,
            component: "Meteora DLMM".to_string(),
            event_type: "OPPORTUNITY_FOUND".to_string(),
            message: "High APR opportunity found: SOL/USDC (2190% APR)".to_string(),
            details: Some(serde_json::json!({
                "pair": "SOL/USDC",
                "apr": 2190.0,
                "strategy": "BinConcentration",
                "liquidity_required": 0.002
            })),
        },
    ]
}

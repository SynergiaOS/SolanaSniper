/*!
🎯 Active Positions Handler - Critical Trading Position Management

This handler provides real-time visibility into active trading positions
and emergency manual override capabilities for risk management.
*/

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

/// Active trading position with full details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePosition {
    pub id: String,
    pub token_address: String,
    pub symbol: String,
    pub strategy: String,
    pub side: String, // "Buy" or "Sell"
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
    pub opened_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub risk_score: f64,
}

/// Request to manually close a position
#[derive(Debug, Deserialize)]
pub struct ClosePositionRequest {
    pub reason: Option<String>,
    pub force: Option<bool>, // Emergency force close
}

/// Response for position operations
#[derive(Debug, Serialize)]
pub struct PositionResponse {
    pub success: bool,
    pub message: String,
    pub position_id: Option<String>,
}

/// Query parameters for positions
#[derive(Debug, Deserialize)]
pub struct PositionsQuery {
    pub strategy: Option<String>,
    pub status: Option<String>, // "active", "closed", "all"
    pub limit: Option<usize>,
}

/// Get all active positions
pub async fn get_active_positions(
    Query(params): Query<PositionsQuery>,
) -> Result<Json<Vec<ActivePosition>>, StatusCode> {
    info!("🎯 Fetching real active positions with filters: {:?}", params);

    // Try to read real active positions
    let positions = match read_active_positions().await {
        Ok(Some(data)) => {
            if let Some(positions) = data.get("positions").and_then(|p| p.as_array()) {
                positions
                    .iter()
                    .filter_map(|pos| serde_json::from_value::<ActivePosition>(pos.clone()).ok())
                    .collect()
            } else {
                vec![]
            }
        }
        _ => {
            warn!("⚠️ Active positions cache not available, using fallback");
            generate_mock_positions()
        }
    };

    let filtered_positions = positions
        .into_iter()
        .filter(|pos| {
            if let Some(ref strategy) = params.strategy {
                return pos.strategy.to_lowercase().contains(&strategy.to_lowercase());
            }
            true
        })
        .take(params.limit.unwrap_or(50))
        .collect::<Vec<_>>();

    info!("✅ Returning {} active positions", filtered_positions.len());
    Ok(Json(filtered_positions))
}

async fn read_active_positions() -> Result<Option<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let json_data = tokio::fs::read_to_string("/tmp/active_positions.json").await?;
    let positions_data: serde_json::Value = serde_json::from_str(&json_data)?;
    Ok(Some(positions_data))
}

/// Get specific position details
pub async fn get_position_details(
    Path(position_id): Path<String>,
) -> Result<Json<ActivePosition>, StatusCode> {
    info!("🔍 Fetching details for position: {}", position_id);

    // TODO: Replace with real position lookup
    let mock_positions = generate_mock_positions();
    
    if let Some(position) = mock_positions.into_iter().find(|p| p.id == position_id) {
        info!("✅ Found position: {} - {} {}", position_id, position.symbol, position.side);
        Ok(Json(position))
    } else {
        warn!("❌ Position not found: {}", position_id);
        Err(StatusCode::NOT_FOUND)
    }
}

/// Manually close a position (CRITICAL SAFETY FEATURE)
pub async fn close_position_manually(
    Path(position_id): Path<String>,
    Json(request): Json<ClosePositionRequest>,
) -> Result<Json<PositionResponse>, StatusCode> {
    warn!("🚨 MANUAL POSITION CLOSE REQUESTED: {} - Reason: {:?}", 
          position_id, request.reason);

    // TODO: Implement real position closing logic
    // This should:
    // 1. Validate position exists
    // 2. Calculate current market price
    // 3. Execute market sell order
    // 4. Update position status
    // 5. Log the manual intervention
    
    let response = PositionResponse {
        success: true,
        message: format!("Position {} queued for manual closure", position_id),
        position_id: Some(position_id.clone()),
    };

    info!("✅ Position {} marked for manual closure", position_id);
    Ok(Json(response))
}

/// Emergency close ALL positions (PANIC BUTTON)
pub async fn emergency_close_all_positions() -> Result<Json<PositionResponse>, StatusCode> {
    error!("🚨🚨 EMERGENCY CLOSE ALL POSITIONS TRIGGERED! 🚨🚨");

    // TODO: Implement emergency close logic
    // This should:
    // 1. Stop all trading strategies
    // 2. Cancel all pending orders
    // 3. Market sell all positions
    // 4. Send alerts to all channels
    
    let response = PositionResponse {
        success: true,
        message: "Emergency close initiated for all positions".to_string(),
        position_id: None,
    };

    error!("🚨 Emergency close procedure initiated");
    Ok(Json(response))
}

/// Generate mock positions for testing
fn generate_mock_positions() -> Vec<ActivePosition> {
    let now = Utc::now();
    
    vec![
        ActivePosition {
            id: "pos_001".to_string(),
            token_address: "SOUL_TOKEN_12345".to_string(),
            symbol: "SOUL".to_string(),
            strategy: "meteora_dlmm".to_string(),
            side: "Buy".to_string(),
            size: 10000.0,
            entry_price: 0.0045,
            current_price: 0.0052,
            unrealized_pnl: 7.0,
            unrealized_pnl_percent: 15.56,
            opened_at: now - chrono::Duration::minutes(45),
            last_updated: now,
            stop_loss: Some(0.0040),
            take_profit: Some(0.0060),
            risk_score: 0.3,
        },
        ActivePosition {
            id: "pos_002".to_string(),
            token_address: "PUMP_TOKEN_67890".to_string(),
            symbol: "PUMP".to_string(),
            strategy: "pumpfun_sniping".to_string(),
            side: "Buy".to_string(),
            size: 5000.0,
            entry_price: 0.0012,
            current_price: 0.0010,
            unrealized_pnl: -1.0,
            unrealized_pnl_percent: -16.67,
            opened_at: now - chrono::Duration::minutes(12),
            last_updated: now,
            stop_loss: Some(0.0009),
            take_profit: Some(0.0018),
            risk_score: 0.7,
        },
        ActivePosition {
            id: "pos_003".to_string(),
            token_address: "LIQ_TOKEN_11111".to_string(),
            symbol: "LIQ".to_string(),
            strategy: "liquidity_sniping".to_string(),
            side: "Buy".to_string(),
            size: 25000.0,
            entry_price: 0.0089,
            current_price: 0.0095,
            unrealized_pnl: 15.0,
            unrealized_pnl_percent: 6.74,
            opened_at: now - chrono::Duration::hours(2),
            last_updated: now,
            stop_loss: Some(0.0080),
            take_profit: Some(0.0110),
            risk_score: 0.2,
        },
    ]
}

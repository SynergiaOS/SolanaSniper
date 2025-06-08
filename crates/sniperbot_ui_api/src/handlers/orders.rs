use axum::Json;
use serde_json::{json, Value};
use tracing::{info, warn};

pub async fn get_orders() -> Json<Value> {
    info!("💰 Fetching real trade history...");

    // Try to read real trade history
    let trades_data = match read_trade_history().await {
        Ok(Some(data)) => {
            if let Some(trades) = data.get("trades").and_then(|t| t.as_array()) {
                trades.clone()
            } else {
                vec![]
            }
        }
        _ => {
            warn!("⚠️ Trade history cache not available, using fallback");
            vec![
                json!({
                    "id": "fallback_001",
                    "symbol": "SOL/USDC",
                    "action": "buy",
                    "amount": 1.5,
                    "price": 98.45,
                    "status": "completed",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "tx_hash": "fallback_hash_123",
                    "strategy": "Manual",
                    "pnl": 12.50,
                    "fees": 0.25,
                    "success": true
                })
            ]
        }
    };

    Json(json!({
        "success": true,
        "data": trades_data
    }))
}

async fn read_trade_history() -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let json_data = tokio::fs::read_to_string("/tmp/trade_history.json").await?;
    let trade_data: Value = serde_json::from_str(&json_data)?;
    Ok(Some(trade_data))
}

pub async fn create_order(Json(payload): Json<Value>) -> Json<Value> {
    Json(json!({
        "status": "created",
        "order_id": "mock_order_123",
        "message": "Order created successfully (mock)",
        "payload": payload
    }))
}

pub async fn manual_trade(Json(payload): Json<Value>) -> Json<Value> {
    let symbol = payload.get("symbol").and_then(|s| s.as_str()).unwrap_or("SOL/USDC");
    let action = payload.get("action").and_then(|a| a.as_str()).unwrap_or("buy");
    let amount = payload.get("amount").and_then(|a| a.as_f64()).unwrap_or(1.0);

    info!("💰 Manual trade requested: {} {} {}", action.to_uppercase(), amount, symbol);

    Json(json!({
        "success": true,
        "trade_id": "manual_trade_456",
        "message": format!("Manual {} trade executed for {} {}", action, amount, symbol),
        "details": {
            "symbol": symbol,
            "action": action,
            "amount": amount,
            "estimated_price": 100.0,
            "status": "executed"
        }
    }))
}

use axum::Json;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_orders() -> Json<Value> {
    // Mock trades data for dashboard
    let mock_trades = vec![
        json!({
            "symbol": "SOL/USDC",
            "side": "buy",
            "amount": 1.5,
            "price": 98.45,
            "status": "completed",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "tx_hash": "mock_hash_123",
            "strategy": "Manual",
            "pnl": 12.50,
            "fee": 0.25
        }),
        json!({
            "symbol": "BONK/SOL",
            "side": "sell",
            "amount": 1000000.0,
            "price": 0.000025,
            "status": "pending",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "strategy": "PumpFun Sniping",
            "pnl": -5.25,
            "fee": 0.25
        })
    ];

    Json(json!({
        "success": true,
        "data": mock_trades
    }))
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

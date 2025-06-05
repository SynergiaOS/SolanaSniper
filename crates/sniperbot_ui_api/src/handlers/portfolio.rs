use axum::Json;
use serde_json::{json, Value};

pub async fn get_portfolio() -> Json<Value> {
    Json(json!({
        "sol_balance": 1.5,
        "total_usd_value": 150.0,
        "active_positions": [],
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}

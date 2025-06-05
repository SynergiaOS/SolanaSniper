use axum::{extract::Path, Json};
use serde_json::{json, Value};

pub async fn get_market_data(Path(symbol): Path<String>) -> Json<Value> {
    Json(json!({
        "symbol": symbol,
        "price": 100.0,
        "volume_24h": 1000000.0,
        "change_24h": 5.2,
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}

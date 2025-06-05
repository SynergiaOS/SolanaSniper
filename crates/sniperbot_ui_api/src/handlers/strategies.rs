use axum::{Json, extract::Path};
use serde_json::{json, Value};
use tracing::info;

pub async fn get_strategies() -> Json<Value> {
    Json(json!({
        "strategies": [
            {
                "name": "pumpfun_sniping",
                "enabled": true,
                "status": "active",
                "last_signal": chrono::Utc::now().to_rfc3339(),
                "profit_24h": 25.50,
                "trades_24h": 12
            },
            {
                "name": "liquidity_sniping",
                "enabled": true,
                "status": "active",
                "last_signal": null,
                "profit_24h": 15.25,
                "trades_24h": 8
            },
            {
                "name": "meteora_dlmm",
                "enabled": false,
                "status": "inactive",
                "last_signal": chrono::Utc::now().to_rfc3339(),
                "profit_24h": 0.0,
                "trades_24h": 0
            }
        ]
    }))
}

pub async fn toggle_strategy(Path(strategy): Path<String>) -> Json<Value> {
    info!("🔄 Strategy toggle requested: {}", strategy);

    // Mock toggle logic
    let enabled = strategy != "meteora_dlmm"; // Mock: meteora_dlmm is disabled

    Json(json!({
        "success": true,
        "strategy": strategy,
        "enabled": !enabled, // Toggle the current state
        "message": format!("Strategy {} {}", strategy, if !enabled { "enabled" } else { "disabled" })
    }))
}

pub async fn reset_strategies() -> Json<Value> {
    info!("🔄 Strategy reset requested");

    Json(json!({
        "success": true,
        "message": "All strategy statistics have been reset",
        "reset_timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

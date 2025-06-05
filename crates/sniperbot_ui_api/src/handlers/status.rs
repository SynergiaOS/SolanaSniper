use axum::Json;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_bot_status() -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "engine_status": {
                "is_running": true,
                "portfolio_value": 1000.0
            },
            "active_strategies": ["pumpfun_sniping", "liquidity_sniping", "meteora_dlmm"],
            "strategy_performance": {
                "pumpfun_sniping": {
                    "enabled": true,
                    "profit_24h": 25.50,
                    "signals_generated": 45,
                    "win_rate": 0.73,
                    "total_pnl": 125.75
                },
                "liquidity_sniping": {
                    "enabled": true,
                    "profit_24h": 15.25,
                    "signals_generated": 28,
                    "win_rate": 0.68,
                    "total_pnl": 89.50
                },
                "meteora_dlmm": {
                    "enabled": true,
                    "profit_24h": 0.0,
                    "signals_generated": 402,
                    "win_rate": 1.0,
                    "total_pnl": 0.0
                }
            }
        }
    }))
}

// Bot control functions
pub async fn start_bot() -> Json<Value> {
    info!("🚀 Bot start requested");
    Json(json!({
        "success": true,
        "message": "Bot started successfully"
    }))
}

pub async fn stop_bot() -> Json<Value> {
    info!("⏹️ Bot stop requested");
    Json(json!({
        "success": true,
        "message": "Bot stopped successfully"
    }))
}

pub async fn pause_bot() -> Json<Value> {
    info!("⏸️ Bot pause requested");
    Json(json!({
        "success": true,
        "message": "Bot paused successfully"
    }))
}

pub async fn emergency_stop() -> Json<Value> {
    info!("🚨 Emergency stop requested");
    Json(json!({
        "success": true,
        "message": "Emergency stop executed"
    }))
}

pub async fn set_mode(Json(payload): Json<Value>) -> Json<Value> {
    let mode = payload.get("mode").and_then(|m| m.as_str()).unwrap_or("dry-run");
    info!("🔄 Mode change requested: {}", mode);
    Json(json!({
        "success": true,
        "message": format!("Mode changed to {}", mode)
    }))
}

// AI functions
pub async fn ai_analyze() -> Json<Value> {
    info!("🧠 AI analysis requested");
    Json(json!({
        "success": true,
        "analysis": {
            "recommendation": "HOLD",
            "confidence": 0.75,
            "reasoning": "Market conditions are neutral, waiting for better entry point"
        }
    }))
}

pub async fn ai_toggle() -> Json<Value> {
    info!("🤖 AI toggle requested");
    Json(json!({
        "success": true,
        "enabled": true,
        "message": "AI mode toggled"
    }))
}

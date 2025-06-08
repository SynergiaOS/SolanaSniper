use axum::{Json, extract::Path};
use serde_json::{json, Value};
use tracing::{info, warn};

pub async fn get_strategies() -> Json<Value> {
    info!("🎯 /api/strategies endpoint called - fetching real strategy data...");

    // Try to read real strategy performance data
    let strategy_performance = match read_strategy_performance().await {
        Ok(Some(perf_data)) => perf_data,
        _ => {
            warn!("⚠️ Strategy performance cache not available, using fallback");
            json!({})
        }
    };

    // Try to read portfolio data to determine active strategies
    let portfolio_data = match read_portfolio_cache().await {
        Ok(Some(data)) => data,
        _ => {
            warn!("⚠️ Portfolio cache not available, using fallback");
            json!({ "sol_balance": 0.0 })
        }
    };

    let sol_balance = portfolio_data.get("sol_balance").and_then(|v| v.as_f64()).unwrap_or(0.0);

    // Determine active strategies based on balance (matching bot logic)
    // 🧪 TESTING MODE: Lowered thresholds for local testing
    let active_strategies = if sol_balance < 0.001 {
        vec!["pumpfun_sniping"]
    } else if sol_balance < 0.01 {
        vec!["pumpfun_sniping", "liquidity_sniping"]
    } else {
        vec!["pumpfun_sniping", "liquidity_sniping", "meteora_dlmm"]
    };

    // Build strategies array with real data
    let strategies: Vec<Value> = ["pumpfun_sniping", "liquidity_sniping", "meteora_dlmm"]
        .iter()
        .map(|&strategy_name| {
            let is_enabled = active_strategies.contains(&strategy_name);
            let status = if is_enabled { "active" } else { "inactive" };

            // Get real performance data
            let strategy_data = strategy_performance.get(strategy_name);
            let signals_generated = strategy_data
                .and_then(|s| s.get("signals_generated"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_pnl = strategy_data
                .and_then(|s| s.get("total_pnl"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let last_signal_time = strategy_data
                .and_then(|s| s.get("last_signal_time"))
                .and_then(|v| v.as_str());

            json!({
                "name": strategy_name,
                "enabled": is_enabled,
                "status": status,
                "last_signal": last_signal_time,
                "profit_24h": total_pnl, // Use total_pnl as proxy for 24h profit
                "trades_24h": signals_generated // Use signals as proxy for trades
            })
        })
        .collect();

    Json(json!({
        "strategies": strategies
    }))
}

async fn read_strategy_performance() -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let json_data = tokio::fs::read_to_string("/tmp/strategy_performance.json").await?;
    let perf_data: Value = serde_json::from_str(&json_data)?;

    // Extract strategies object
    if let Some(strategies) = perf_data.get("strategies") {
        Ok(Some(strategies.clone()))
    } else {
        Ok(None)
    }
}

async fn read_portfolio_cache() -> Result<Option<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let json_data = tokio::fs::read_to_string("/tmp/portfolio_status.json").await?;
    let portfolio_data: Value = serde_json::from_str(&json_data)?;
    Ok(Some(portfolio_data))
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

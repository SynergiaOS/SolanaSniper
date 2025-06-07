pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer, services::ServeDir};
use tracing::info;
use models::AppState;

pub async fn create_app() -> Router {
    info!("🌐 Creating SniperBot UI API server");

    let state = AppState::new();

    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/api/v1/status", get(handlers::status::get_bot_status))
        .route("/api/v1/portfolio", get(handlers::portfolio::get_portfolio))
        .route("/api/v1/orders", get(handlers::orders::get_orders))
        .route("/api/v1/orders", post(handlers::orders::create_order))
        .route("/api/v1/strategies", get(handlers::strategies::get_strategies))
        .route("/api/v1/market-data/:symbol", get(handlers::market_data::get_market_data))
        // KLUCZOWY ENDPOINT dla Reporter z SniperBot core
        .route("/api/report_event", post(handlers::report_events::receive_report_event))
        .route("/api/events", get(handlers::report_events::get_all_events))
        // WebSocket endpoint for real-time updates
        .route("/ws", get(handlers::websocket::websocket_handler))
        // Dashboard compatibility endpoints (matching frontend expectations)
        .route("/api/bot-status", get(handlers::status::get_bot_status))
        .route("/api/signals", get(handlers::report_events::get_signals))
        .route("/api/trades", get(handlers::orders::get_orders))
        // Bot control endpoints
        .route("/api/bot/start", post(handlers::status::start_bot))
        .route("/api/bot/stop", post(handlers::status::stop_bot))
        .route("/api/bot/pause", post(handlers::status::pause_bot))
        .route("/api/bot/emergency-stop", post(handlers::status::emergency_stop))
        .route("/api/bot/mode", post(handlers::status::set_mode))
        // Manual trading
        .route("/api/trade/manual", post(handlers::orders::manual_trade))
        // AI endpoints
        .route("/api/ai/analyze", post(handlers::status::ai_analyze))
        .route("/api/ai/toggle", post(handlers::status::ai_toggle))
        // Strategy endpoints
        .route("/api/strategy/:strategy/toggle", post(handlers::strategies::toggle_strategy))
        .route("/api/strategy/reset", post(handlers::strategies::reset_strategies))
        // Position management endpoints (CRITICAL SAFETY FEATURES)
        .route("/api/positions", get(handlers::positions::get_active_positions))
        .route("/api/positions/:id", get(handlers::positions::get_position_details))
        .route("/api/positions/:id/close", post(handlers::positions::close_position_manually))
        .route("/api/positions/emergency-close-all", post(handlers::positions::emergency_close_all_positions))
        // Live event stream (BOT INTELLIGENCE VISIBILITY)
        .route("/api/live-events", get(handlers::live_events::get_live_events))
        // Serve static frontend files
        .nest_service("/", ServeDir::new("frontend/dist").fallback(ServeDir::new("frontend/dist/index.html")))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let app = create_app().await;
    let addr = format!("0.0.0.0:{}", port);
    
    info!("🚀 Starting SniperBot UI API server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

pub async fn create_app() -> Router {
    info!("🌐 Creating SniperBot UI API server");

    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/api/v1/status", get(handlers::status::get_bot_status))
        .route("/api/v1/portfolio", get(handlers::portfolio::get_portfolio))
        .route("/api/v1/orders", get(handlers::orders::get_orders))
        .route("/api/v1/orders", post(handlers::orders::create_order))
        .route("/api/v1/strategies", get(handlers::strategies::get_strategies))
        .route("/api/v1/market-data/:symbol", get(handlers::market_data::get_market_data))
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

use sniperbot_ui_api::create_app;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Create the application
    let app = create_app().await;

    // Define the address to bind to
    let addr = SocketAddr::from(([127, 0, 0, 1], 8084));
    
    info!("🚀 SniperBot API Server starting on {}", addr);
    
    // Create a TcpListener
    let listener = TcpListener::bind(addr).await?;
    
    info!("✅ API Server listening on http://{}", addr);
    info!("📊 WebSocket endpoint: ws://{}/ws", addr);
    info!("🔗 Health check: http://{}/health", addr);
    
    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

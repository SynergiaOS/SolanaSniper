use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use super::ReportEvent;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Store all incoming report events from the bot
    pub report_events: Arc<Mutex<Vec<ReportEvent>>>,
    /// WebSocket broadcast channel for real-time updates
    pub ws_tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> Self {
        let (ws_tx, _) = broadcast::channel(1000);
        
        Self {
            report_events: Arc::new(Mutex::new(Vec::new())),
            ws_tx,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

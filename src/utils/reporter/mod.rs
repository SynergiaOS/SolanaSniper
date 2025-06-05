use crate::models::{StrategySignal, TradingResult, TradingError, AIEnhancedSignal};
use reqwest::Client as HttpClient;
use serde::Serialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Event types that can be reported to the dashboard
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ReportEvent {
    SignalGenerated {
        strategy: String,
        symbol: String,
        signal_type: String,
        strength: f64,
        metadata: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    TradeExecuted {
        strategy: String,
        symbol: String,
        action: String,
        amount: f64,
        price: f64,
        fees: f64,
        success: bool,
        error_message: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    BalanceUpdate {
        total_value: f64,
        available_balance: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
        daily_pnl: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RiskAlert {
        alert_type: String,
        severity: String,
        message: String,
        strategy: Option<String>,
        symbol: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ErrorOccurred {
        error_type: String,
        error_message: String,
        component: String,
        strategy: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    PerformanceMetric {
        metric_name: String,
        metric_value: f64,
        strategy: Option<String>,
        symbol: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    AIDecision {
        strategy: String,
        symbol: String,
        ai_recommendation: String,
        confidence: f64,
        reasoning: String,
        risk_assessment: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    MarketOpportunity {
        opportunity_type: String,
        symbol: String,
        confidence_score: f64,
        estimated_profit: Option<f64>,
        market_cap: Option<f64>,
        volume_24h: Option<f64>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Configuration for the reporter
#[derive(Debug, Clone)]
pub struct ReporterConfig {
    pub enabled: bool,
    pub dashboard_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub batch_size: usize,
    pub flush_interval_seconds: u64,
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dashboard_url: "https://your-replit-backend.replit.app/api/report_event".to_string(),
            api_key: None,
            timeout_seconds: 10,
            retry_attempts: 3,
            batch_size: 10,
            flush_interval_seconds: 30,
        }
    }
}

/// Reporter for sending events to external dashboard
pub struct Reporter {
    config: ReporterConfig,
    http_client: HttpClient,
    event_sender: mpsc::Sender<ReportEvent>,
    event_receiver: Option<mpsc::Receiver<ReportEvent>>,
}

impl Reporter {
    /// Create a new reporter instance
    pub fn new(config: ReporterConfig) -> Self {
        debug!("🔧 Creating Reporter with URL: {}", config.dashboard_url);
        let (event_sender, event_receiver) = mpsc::channel::<ReportEvent>(1000);

        Self {
            config,
            http_client: HttpClient::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            event_sender,
            event_receiver: Some(event_receiver),
        }
    }

    /// Get a sender for reporting events
    pub fn get_sender(&self) -> mpsc::Sender<ReportEvent> {
        self.event_sender.clone()
    }

    /// Start the reporter background service
    pub async fn start(&mut self) -> TradingResult<()> {
        if !self.config.enabled {
            info!("📊 Reporter disabled in configuration");
            return Ok(());
        }

        info!("📊 Starting Reporter service...");
        info!("📊 Dashboard URL: {}", self.config.dashboard_url);

        let mut event_receiver = self.event_receiver.take()
            .ok_or_else(|| TradingError::ConfigurationError("Reporter already started".to_string()))?;

        let config = self.config.clone();
        let http_client = self.http_client.clone();

        tokio::spawn(async move {
            let mut event_batch = Vec::new();
            let mut flush_interval = tokio::time::interval(Duration::from_secs(config.flush_interval_seconds));
            flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    // Receive new events
                    Some(event) = event_receiver.recv() => {
                        event_batch.push(event);
                        
                        // Send batch if it reaches the configured size
                        if event_batch.len() >= config.batch_size {
                            Self::send_event_batch(&http_client, &config, &mut event_batch).await;
                        }
                    }
                    
                    // Periodic flush of remaining events
                    _ = flush_interval.tick() => {
                        if !event_batch.is_empty() {
                            Self::send_event_batch(&http_client, &config, &mut event_batch).await;
                        }
                    }
                }
            }
        });

        info!("✅ Reporter service started successfully");
        Ok(())
    }

    /// Send a batch of events to the dashboard
    async fn send_event_batch(
        http_client: &HttpClient,
        config: &ReporterConfig,
        event_batch: &mut Vec<ReportEvent>,
    ) {
        if event_batch.is_empty() {
            return;
        }

        debug!("📊 Sending batch of {} events to dashboard", event_batch.len());

        let payload = serde_json::json!({
            "events": event_batch,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "sniperbot"
        });

        for attempt in 1..=config.retry_attempts {
            match Self::send_request(http_client, config, &payload).await {
                Ok(_) => {
                    debug!("✅ Successfully sent {} events to dashboard", event_batch.len());
                    event_batch.clear();
                    return;
                }
                Err(e) => {
                    warn!("⚠️ Failed to send events (attempt {}/{}): {}", 
                        attempt, config.retry_attempts, e);
                    
                    if attempt < config.retry_attempts {
                        tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                    }
                }
            }
        }

        error!("❌ Failed to send events after {} attempts, dropping batch", config.retry_attempts);
        event_batch.clear();
    }

    /// Send HTTP request to dashboard
    async fn send_request(
        http_client: &HttpClient,
        config: &ReporterConfig,
        payload: &serde_json::Value,
    ) -> TradingResult<()> {
        debug!("📊 Sending POST request to URL: {}", config.dashboard_url);
        debug!("🔍 DEBUG: URL length: {}, URL content: '{}'", config.dashboard_url.len(), config.dashboard_url);

        // Parse URL to check if it's valid
        match reqwest::Url::parse(&config.dashboard_url) {
            Ok(parsed_url) => {
                debug!("✅ URL parsed successfully: scheme={}, host={:?}, path={}",
                       parsed_url.scheme(), parsed_url.host_str(), parsed_url.path());
            }
            Err(e) => {
                debug!("❌ URL parsing failed: {}", e);
            }
        }

        let mut request = http_client
            .post(&config.dashboard_url)
            .header("Content-Type", "application/json")
            .json(payload);

        // Add API key if configured
        if let Some(api_key) = &config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let timeout_duration = Duration::from_secs(config.timeout_seconds);
        
        match timeout(timeout_duration, request.send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    Err(TradingError::NetworkError(format!(
                        "Dashboard API error {}: {}", status, error_text
                    )))
                }
            }
            Ok(Err(e)) => {
                Err(TradingError::NetworkError(format!("HTTP request failed: {}", e)))
            }
            Err(_) => {
                Err(TradingError::NetworkError(format!(
                    "Request timeout after {} seconds", config.timeout_seconds
                )))
            }
        }
    }

    /// Report a strategy signal
    pub async fn report_signal(&self, signal: &StrategySignal) -> TradingResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = ReportEvent::SignalGenerated {
            strategy: signal.strategy.clone(),
            symbol: signal.symbol.clone(),
            signal_type: format!("{:?}", signal.signal_type),
            strength: signal.strength,
            metadata: signal.metadata.clone(),
            timestamp: signal.timestamp,
        };

        self.send_event(event).await
    }

    /// Report an AI-enhanced signal decision
    pub async fn report_ai_decision(&self, enhanced_signal: &AIEnhancedSignal) -> TradingResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = ReportEvent::AIDecision {
            strategy: enhanced_signal.original_signal.strategy.clone(),
            symbol: enhanced_signal.original_signal.symbol.clone(),
            ai_recommendation: enhanced_signal.ai_recommendation.action.clone(),
            confidence: enhanced_signal.ai_confidence,
            reasoning: enhanced_signal.ai_recommendation.rationale.clone(),
            risk_assessment: format!("Risk Score: {:.2}, Final Action: {}",
                                   enhanced_signal.risk_score, enhanced_signal.final_action),
            timestamp: enhanced_signal.processing_timestamp,
        };

        self.send_event(event).await
    }

    /// Report a trade execution
    pub async fn report_trade(
        &self,
        strategy: String,
        symbol: String,
        action: String,
        amount: f64,
        price: f64,
        fees: f64,
        success: bool,
        error_message: Option<String>,
    ) -> TradingResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = ReportEvent::TradeExecuted {
            strategy,
            symbol,
            action,
            amount,
            price,
            fees,
            success,
            error_message,
            timestamp: chrono::Utc::now(),
        };

        self.send_event(event).await
    }

    /// Report a balance update
    pub async fn report_balance_update(
        &self,
        total_value: f64,
        available_balance: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
        daily_pnl: f64,
    ) -> TradingResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = ReportEvent::BalanceUpdate {
            total_value,
            available_balance,
            unrealized_pnl,
            realized_pnl,
            daily_pnl,
            timestamp: chrono::Utc::now(),
        };

        self.send_event(event).await
    }

    /// Report a risk alert
    pub async fn report_risk_alert(
        &self,
        alert_type: String,
        severity: String,
        message: String,
        strategy: Option<String>,
        symbol: Option<String>,
    ) -> TradingResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let event = ReportEvent::RiskAlert {
            alert_type,
            severity,
            message,
            strategy,
            symbol,
            timestamp: chrono::Utc::now(),
        };

        self.send_event(event).await
    }

    /// Send an event to the reporting queue
    async fn send_event(&self, event: ReportEvent) -> TradingResult<()> {
        if let Err(e) = self.event_sender.send(event).await {
            warn!("Failed to queue report event: {}", e);
            return Err(TradingError::DataError("Failed to queue report event".to_string()));
        }
        Ok(())
    }
}

/// Helper functions for creating common report events
impl ReportEvent {
    /// Create a signal generated event
    pub fn signal_generated(signal: &StrategySignal) -> Self {
        Self::SignalGenerated {
            strategy: signal.strategy.clone(),
            symbol: signal.symbol.clone(),
            signal_type: format!("{:?}", signal.signal_type),
            strength: signal.strength,
            metadata: signal.metadata.clone(),
            timestamp: signal.timestamp,
        }
    }

    /// Create an AI decision event
    pub fn ai_decision(
        strategy: String,
        symbol: String,
        ai_recommendation: String,
        confidence: f64,
        reasoning: String,
        risk_assessment: String,
    ) -> Self {
        Self::AIDecision {
            strategy,
            symbol,
            ai_recommendation,
            confidence,
            reasoning,
            risk_assessment,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a performance metric event
    pub fn performance_metric(
        metric_name: String,
        metric_value: f64,
        strategy: Option<String>,
        symbol: Option<String>,
    ) -> Self {
        Self::PerformanceMetric {
            metric_name,
            metric_value,
            strategy,
            symbol,
            timestamp: chrono::Utc::now(),
        }
    }
}

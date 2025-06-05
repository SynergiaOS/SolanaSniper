use serde::{Deserialize, Serialize};

/// ReportEvent structure that mirrors the one from sniperbot_core
/// This is used to receive events from the bot via POST /api/report_event
#[derive(Debug, Clone, Serialize, Deserialize)]
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

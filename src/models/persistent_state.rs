/*!
🗄️ Persistent State Models - Data structures for DragonflyDB storage

This module defines all data structures that will be stored in DragonflyDB
for the Hub-and-Spoke architecture of SniperBot 2.0.
*/

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::data_fetcher::soul_meteor_scanner::HotCandidate;
use crate::pipeline::opportunity::ValidatedOpportunity;
use crate::pipeline::decision_engine::TradingDecision;

/// Raw opportunity from Soul Meteor Scanner (stored in raw_opportunities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawOpportunity {
    /// Unique identifier
    pub id: String,
    /// Hot candidate data from Soul Meteor
    pub candidate: HotCandidate,
    /// Discovery timestamp
    pub discovered_at: DateTime<Utc>,
    /// Expiration timestamp (for TTL)
    pub expires_at: DateTime<Utc>,
    /// Processing status
    pub status: RawOpportunityStatus,
}

/// Status of raw opportunity processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RawOpportunityStatus {
    /// Newly discovered, awaiting validation
    Pending,
    /// Currently being validated by Crawl4AI
    Validating,
    /// Validation completed (success or failure)
    Processed,
    /// Expired and should be cleaned up
    Expired,
}

/// Validated opportunity (stored in validated_opportunities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredValidatedOpportunity {
    /// Unique identifier
    pub id: String,
    /// Original raw opportunity ID
    pub raw_opportunity_id: String,
    /// Validated opportunity data
    pub opportunity: ValidatedOpportunity,
    /// Validation timestamp
    pub validated_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
}

/// Trading decision (stored in trading_queue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTradingDecision {
    /// Unique decision ID
    pub id: String,
    /// Related opportunity ID
    pub opportunity_id: String,
    /// Trading decision data
    pub decision: TradingDecision,
    /// Queue timestamp
    pub queued_at: DateTime<Utc>,
    /// Execution status
    pub status: DecisionStatus,
    /// Execution attempts
    pub attempts: u32,
    /// Last attempt timestamp
    pub last_attempt_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Status of trading decision execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionStatus {
    /// Queued for execution
    Queued,
    /// Currently being executed
    Executing,
    /// Successfully executed
    Executed,
    /// Failed execution (will retry)
    Failed,
    /// Permanently failed (no more retries)
    Abandoned,
    /// Cancelled by user or system
    Cancelled,
}

/// Active trading position (stored in active_positions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePosition {
    /// Unique position ID
    pub id: String,
    /// Related decision ID
    pub decision_id: String,
    /// Related opportunity ID
    pub opportunity_id: String,
    /// Position type
    pub position_type: PositionType,
    /// Token information
    pub token_address: String,
    /// Position size in SOL
    pub size_sol: f64,
    /// Entry price
    pub entry_price: f64,
    /// Current price (updated periodically)
    pub current_price: f64,
    /// Unrealized P&L in SOL
    pub unrealized_pnl_sol: f64,
    /// Stop loss price (if set)
    pub stop_loss_price: Option<f64>,
    /// Take profit price (if set)
    pub take_profit_price: Option<f64>,
    /// Position opened timestamp
    pub opened_at: DateTime<Utc>,
    /// Last price update timestamp
    pub last_updated_at: DateTime<Utc>,
    /// Position status
    pub status: PositionStatus,
}

/// Type of trading position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionType {
    /// Token purchase (sniper strategy)
    TokenPurchase {
        /// Amount of tokens purchased
        token_amount: f64,
        /// Transaction signature
        transaction_signature: String,
    },
    /// Liquidity provision
    LiquidityProvision {
        /// Pool address
        pool_address: String,
        /// LP token amount
        lp_token_amount: f64,
        /// Expected duration in hours
        duration_hours: u32,
        /// Transaction signature
        transaction_signature: String,
    },
    /// Arbitrage position
    Arbitrage {
        /// Buy transaction signature
        buy_signature: String,
        /// Sell transaction signature (if completed)
        sell_signature: Option<String>,
    },
}

/// Status of trading position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PositionStatus {
    /// Position is open and active
    Open,
    /// Position is being closed
    Closing,
    /// Position closed with profit
    ClosedProfit,
    /// Position closed with loss
    ClosedLoss,
    /// Position closed at break-even
    ClosedBreakeven,
    /// Position liquidated (stop loss triggered)
    Liquidated,
    /// Position failed to open properly
    Failed,
}

/// Historical performance record (stored in performance_history)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    /// Unique record ID
    pub id: String,
    /// Related position ID
    pub position_id: String,
    /// Strategy used
    pub strategy: String,
    /// Token traded
    pub token_address: String,
    /// Position size in SOL
    pub size_sol: f64,
    /// Entry price
    pub entry_price: f64,
    /// Exit price
    pub exit_price: f64,
    /// Realized P&L in SOL
    pub realized_pnl_sol: f64,
    /// Realized P&L percentage
    pub realized_pnl_percentage: f64,
    /// Position duration in minutes
    pub duration_minutes: i64,
    /// Fees paid in SOL
    pub fees_sol: f64,
    /// Position opened timestamp
    pub opened_at: DateTime<Utc>,
    /// Position closed timestamp
    pub closed_at: DateTime<Utc>,
    /// Closing reason
    pub closing_reason: ClosingReason,
}

/// Reason for closing position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClosingReason {
    /// Take profit triggered
    TakeProfit,
    /// Stop loss triggered
    StopLoss,
    /// Manual close by user
    Manual,
    /// Time-based exit
    TimeExit,
    /// Strategy signal to close
    StrategySignal,
    /// Emergency close
    Emergency,
}

/// System metrics (stored in system_metrics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Metric timestamp
    pub timestamp: DateTime<Utc>,
    /// Total opportunities discovered
    pub total_opportunities_discovered: u64,
    /// Total opportunities validated
    pub total_opportunities_validated: u64,
    /// Total decisions made
    pub total_decisions_made: u64,
    /// Total positions opened
    pub total_positions_opened: u64,
    /// Total positions closed
    pub total_positions_closed: u64,
    /// Total realized P&L in SOL
    pub total_realized_pnl_sol: f64,
    /// Current active positions count
    pub active_positions_count: u32,
    /// Average validation time in seconds
    pub avg_validation_time_secs: f64,
    /// Average decision time in seconds
    pub avg_decision_time_secs: f64,
    /// Success rate percentage
    pub success_rate_percentage: f64,
}

/// Database key patterns for organized storage
pub struct DbKeys;

impl DbKeys {
    /// Raw opportunities: raw_opportunity:{token_address}
    pub fn raw_opportunity(token_address: &str) -> String {
        format!("raw_opportunity:{}", token_address)
    }

    /// Validated opportunities: validated_opportunity:{opportunity_id}
    pub fn validated_opportunity(opportunity_id: &str) -> String {
        format!("validated_opportunity:{}", opportunity_id)
    }

    /// Trading decisions queue: trading_queue
    pub const TRADING_QUEUE: &'static str = "trading_queue";

    /// Active positions: active_position:{position_id}
    pub fn active_position(position_id: &str) -> String {
        format!("active_position:{}", position_id)
    }

    /// Performance history: performance_record:{record_id}
    pub fn performance_record(record_id: &str) -> String {
        format!("performance_record:{}", record_id)
    }

    /// System metrics: system_metrics:{date}
    pub fn system_metrics(date: &str) -> String {
        format!("system_metrics:{}", date)
    }

    /// Processed tokens set (for deduplication): processed_tokens
    pub const PROCESSED_TOKENS: &'static str = "processed_tokens";

    /// Active tokens set: active_tokens
    pub const ACTIVE_TOKENS: &'static str = "active_tokens";

    /// All raw opportunities list: all_raw_opportunities
    pub const ALL_RAW_OPPORTUNITIES: &'static str = "all_raw_opportunities";

    /// All validated opportunities list: all_validated_opportunities
    pub const ALL_VALIDATED_OPPORTUNITIES: &'static str = "all_validated_opportunities";

    /// All active positions list: all_active_positions
    pub const ALL_ACTIVE_POSITIONS: &'static str = "all_active_positions";

    /// Trading decisions queue: trading_decisions_queue
    pub const TRADING_DECISIONS_QUEUE: &'static str = "trading_decisions_queue";

    /// Bot statistics: bot_stats
    pub const BOT_STATS: &'static str = "bot_stats";

    /// Configuration cache: config_cache
    pub const CONFIG_CACHE: &'static str = "config_cache";

    /// Pipeline statistics: pipeline_stats
    pub const PIPELINE_STATS: &'static str = "pipeline_stats";

    /// Health check status: health_status
    pub const HEALTH_STATUS: &'static str = "health_status";
}

impl RawOpportunity {
    /// Create new raw opportunity from hot candidate
    pub fn from_candidate(candidate: HotCandidate, ttl_hours: u32) -> Self {
        let now = Utc::now();
        let id = format!("{}_{}", candidate.address, now.timestamp());
        
        Self {
            id,
            candidate,
            discovered_at: now,
            expires_at: now + chrono::Duration::hours(ttl_hours as i64),
            status: RawOpportunityStatus::Pending,
        }
    }

    /// Check if opportunity has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

impl ActivePosition {
    /// Calculate current P&L percentage
    pub fn calculate_pnl_percentage(&self) -> f64 {
        if self.entry_price == 0.0 {
            0.0
        } else {
            ((self.current_price - self.entry_price) / self.entry_price) * 100.0
        }
    }

    /// Check if stop loss should be triggered
    pub fn should_trigger_stop_loss(&self) -> bool {
        if let Some(stop_loss) = self.stop_loss_price {
            self.current_price <= stop_loss
        } else {
            false
        }
    }

    /// Check if take profit should be triggered
    pub fn should_trigger_take_profit(&self) -> bool {
        if let Some(take_profit) = self.take_profit_price {
            self.current_price >= take_profit
        } else {
            false
        }
    }
}

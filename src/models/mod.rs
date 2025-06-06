use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub source: DataSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DataSource {
    Binance,
    Coinbase,
    Kraken,
    Solana,
    Ethereum,
    Polygon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub symbol: String,
    pub side: PositionSide,
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub strategy: String,
    pub exchange: String,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_loss: f64,
    pub risk_reward_ratio: f64,
    pub position_size_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub exchange_order_id: Option<String>,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub size: f64,
    pub price: Option<f64>,
    pub filled_size: f64,
    pub average_fill_price: Option<f64>,
    pub status: OrderStatus,
    pub exchange: String,
    pub strategy: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub time_in_force: TimeInForce,
    pub execution_params: ExecutionParams,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_slippage_bps: u16,
    pub actual_slippage_bps: Option<u16>,
    pub fees_paid: f64,
    pub transaction_signature: Option<String>,
    pub bundle_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    StopLossLimit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    GTD, // Good Till Date
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionParams {
    pub use_jito: bool,
    pub priority_fee_lamports: u64,
    pub compute_unit_limit: u32,
    pub compute_unit_price: u64,
    pub max_retries: u8,
    pub retry_delay_ms: u64,
    pub timeout_ms: u64,
    pub use_versioned_transaction: bool,
}

impl Default for ExecutionParams {
    fn default() -> Self {
        Self {
            use_jito: true,
            priority_fee_lamports: 10000, // 0.00001 SOL
            compute_unit_limit: 200000,
            compute_unit_price: 1000,
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_ms: 30000,
            use_versioned_transaction: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoBundle {
    pub id: String,
    pub transactions: Vec<String>, // Base64 encoded transactions
    pub status: BundleStatus,
    pub submitted_at: DateTime<Utc>,
    pub landed_at: Option<DateTime<Utc>>,
    pub tip_lamports: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BundleStatus {
    Pending,
    Submitted,
    Landed,
    Failed,
    Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub size: f64,
    pub price: f64,
    pub fee: f64,
    pub fee_currency: String,
    pub timestamp: DateTime<Utc>,
    pub exchange: String,
    pub strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub total_value: f64,
    pub total_value_usd: Option<f64>,
    pub available_balance: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub positions: Vec<Position>,
    pub daily_pnl: f64,
    pub max_drawdown: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    pub strategy: String,
    pub symbol: String,
    pub signal_type: SignalType,
    pub strength: f64, // 0.0 to 1.0
    pub price: f64,
    pub size: f64,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    StopLoss,
    TakeProfit,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::Buy => write!(f, "BUY"),
            SignalType::Sell => write!(f, "SELL"),
            SignalType::Hold => write!(f, "HOLD"),
            SignalType::StopLoss => write!(f, "STOP_LOSS"),
            SignalType::TakeProfit => write!(f, "TAKE_PROFIT"),
        }
    }
}

// AI-Enhanced Signal Processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEnhancedSignal {
    pub original_signal: StrategySignal,
    pub ai_recommendation: AIRecommendation,
    pub ai_confidence: f64,
    pub ai_analysis: String,
    pub final_action: String, // "EXECUTE", "HOLD", "REJECT"
    pub risk_score: f64,
    pub market_context: serde_json::Value,
    pub processing_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRecommendation {
    pub action: String, // "BUY", "SELL", "HOLD", "NO_ACTION", "REJECT"
    pub confidence: f64, // 0.0 - 1.0
    pub rationale: String,
    pub risk_score: f64, // 0.0 - 1.0 (AI-assessed risk level)
    pub target_price: Option<f64>,
    pub stop_loss_price: Option<f64>,
    pub strategy_parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub source: String, // talib_minimal, social_scanner, sentiment_analyzer
    pub symbol: String,
    pub result_type: String,
    pub data: serde_json::Value,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

// Legacy MarketEventType for backward compatibility
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LegacyMarketEventType {
    PriceUpdate,
    VolumeSpike,
    OrderBookUpdate,
    NewListing,
    Delisting,
    TradingHalt,
    TradingResume,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum TradingError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Exchange error: {0}")]
    ExchangeError(String),

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: f64, available: f64 },

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    // Execution-specific errors
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Slippage too high: expected {expected_bps}bps, actual {actual_bps}bps")]
    SlippageTooHigh { expected_bps: u16, actual_bps: u16 },

    #[error("Transaction timeout after {timeout_ms}ms")]
    TransactionTimeout { timeout_ms: u64 },

    #[error("Jito bundle failed: {0}")]
    JitoBundleFailed(String),

    #[error("MEV attack detected: {0}")]
    MevAttackDetected(String),

    #[error("Insufficient SOL for fees: required {required}, available {available}")]
    InsufficientSolForFees { required: f64, available: f64 },

    #[error("Price impact too high: {impact_percentage}%")]
    PriceImpactTooHigh { impact_percentage: f64 },

    #[error("Liquidity insufficient for order size")]
    InsufficientLiquidity,

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("AI error: {0}")]
    AIError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

pub type TradingResult<T> = Result<T, TradingError>;

impl From<String> for TradingError {
    fn from(msg: String) -> Self {
        TradingError::DataError(msg)
    }
}

// Balance Management structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub sol_balance: f64,
    pub token_balances: std::collections::HashMap<String, TokenBalance>,
    pub total_value_usd: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub balance: f64,
    pub decimals: u8,
    pub value_usd: Option<f64>,
    pub locked_amount: f64, // Amount locked in pending orders
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub order_id: Uuid,
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub filled_size: f64,
    pub filled_price: Option<f64>,
    pub fees_paid: f64,
    pub slippage_bps: Option<u16>,
    pub execution_time_ms: u64,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// WebSocket Events for Real-Time Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEvent {
    /// Price update for a specific token
    PriceUpdate {
        symbol: String,
        price: f64,
        volume_24h: Option<f64>,
        timestamp: u64,
        source: String, // "helius", "jupiter", "binance", etc.
    },

    /// New transaction detected
    NewTransaction {
        signature: String,
        token_address: String,
        amount: f64,
        price: Option<f64>,
        transaction_type: TransactionType,
        timestamp: u64,
    },

    /// Liquidity pool update
    LiquidityUpdate {
        pool_address: String,
        token_a: String,
        token_b: String,
        liquidity_a: f64,
        liquidity_b: f64,
        price: f64,
        timestamp: u64,
    },

    /// New token listing detected
    NewTokenListing {
        token_address: String,
        symbol: Option<String>,
        name: Option<String>,
        initial_price: Option<f64>,
        initial_liquidity: Option<f64>,
        creator: Option<String>,
        timestamp: u64,
    },

    /// Large transaction alert (whale movement)
    WhaleAlert {
        signature: String,
        token_address: String,
        amount_usd: f64,
        transaction_type: TransactionType,
        wallet_address: String,
        timestamp: u64,
    },

    /// Connection status change
    ConnectionStatus {
        connected: bool,
        source: String,
        error: Option<String>,
        timestamp: u64,
    },

    /// Raw message for debugging/fallback
    RawMessage {
        source: String,
        data: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Transfer,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub helius_ws_url: Option<String>,
    pub jupiter_ws_url: Option<String>,
    pub binance_ws_url: Option<String>,
    pub reconnect_timeout_seconds: u64,
    pub max_retries: u32,
    pub ping_interval_seconds: u64,
    pub subscriptions: Vec<WebSocketSubscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSubscription {
    pub subscription_type: SubscriptionType,
    pub symbol: Option<String>,
    pub address: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionType {
    PriceUpdates,
    NewTransactions,
    LiquidityChanges,
    NewTokens,
    WhaleAlerts,
    AccountUpdates,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            helius_ws_url: Some("wss://atlas-mainnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}".to_string()),
            jupiter_ws_url: None, // Jupiter doesn't have WS yet
            binance_ws_url: Some("wss://stream.binance.com:9443/ws".to_string()),
            reconnect_timeout_seconds: 5,
            max_retries: 10,
            ping_interval_seconds: 30,
            subscriptions: vec![
                WebSocketSubscription {
                    subscription_type: SubscriptionType::PriceUpdates,
                    symbol: Some("SOL/USDC".to_string()),
                    address: None,
                    enabled: true,
                },
                WebSocketSubscription {
                    subscription_type: SubscriptionType::NewTokens,
                    symbol: None,
                    address: None,
                    enabled: true,
                },
            ],
        }
    }
}

// AI Decision Engine structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: Option<String>,
    pub address: String,
    pub price: f64,
    pub market_cap: Option<f64>,
    pub volume_24h: Option<f64>,
    pub liquidity: Option<f64>,
    pub age_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedAnalytics {
    pub technical_score: f64,
    pub social_score: f64,
    pub sentiment_score: f64,
    pub risk_score: f64,
    pub overall_confidence: f64,
    pub signals: Vec<AnalyticsResult>,
    // NEW: Textual intelligence from Crawl4AI
    pub textual_data: Option<TextualData>,
    pub news_impact_score: Option<f64>,
}

// Crawl4AI Integration Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextualData {
    pub token_symbol: String,
    pub timestamp: i64,
    pub data_type: String,
    pub sources: Vec<TextualSource>,
    pub aggregated_sentiment: SentimentSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextualSource {
    pub url: String,
    pub title: String,
    pub content: String,
    pub sentiment_score: f64,
    pub keywords: Vec<String>,
    pub source_type: String,
    pub source_name: String,
    pub credibility_score: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSummary {
    pub overall_score: f64,
    pub positive_mentions: u32,
    pub negative_mentions: u32,
    pub neutral_mentions: u32,
    pub trending_keywords: Vec<String>,
}

// Crawl4AI Request/Response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crawl4AIRequest {
    pub token_symbol: String,
    pub token_address: Option<String>,
    pub data_types: Vec<String>,
    pub time_range_hours: u32,
    pub max_results: u32,
    pub sentiment_analysis: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crawl4AIResponse {
    pub status: String,
    pub data: Option<TextualData>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub sources_scraped: u32,
    pub total_items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    pub volatility: f64,
    pub liquidity_depth: f64,
    pub volume_trend: String,
    pub price_momentum: String,
    pub market_cap: Option<f64>,
    pub age_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    pub total_balance_sol: f64,
    pub available_balance_sol: f64,
    pub total_value_usd: f64,
    pub active_positions: u32,
    pub daily_pnl: f64,
    pub max_drawdown: f64,
    pub risk_exposure: f64,
}

// Persistent state models for DragonflyDB
pub mod persistent_state;
pub use persistent_state::*;

// Python compatibility models
pub mod python_compat;
pub use python_compat::*;

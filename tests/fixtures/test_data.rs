// Test data generators and fixtures
// Provides realistic test data for tokens, trades, and market conditions

use std::collections::HashMap;

/// Test token data
#[derive(Debug, Clone)]
pub struct TestToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub supply: u64,
    pub price_usd: f64,
    pub liquidity_usd: f64,
    pub volume_24h_usd: f64,
    pub age_minutes: u32,
    pub holder_count: u32,
    pub creator_verified: bool,
    pub risk_score: f64,
}

impl TestToken {
    pub fn sol() -> Self {
        Self {
            address: "So11111111111111111111111111111111111111112".to_string(),
            symbol: "SOL".to_string(),
            name: "Wrapped SOL".to_string(),
            decimals: 9,
            supply: 1_000_000_000_000_000_000,
            price_usd: 100.0,
            liquidity_usd: 50_000_000.0,
            volume_24h_usd: 500_000_000.0,
            age_minutes: 0, // Genesis token
            holder_count: 1_000_000,
            creator_verified: true,
            risk_score: 0.1,
        }
    }

    pub fn usdc() -> Self {
        Self {
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            supply: 50_000_000_000_000_000,
            price_usd: 1.0,
            liquidity_usd: 100_000_000.0,
            volume_24h_usd: 1_000_000_000.0,
            age_minutes: 0, // Established token
            holder_count: 500_000,
            creator_verified: true,
            risk_score: 0.05,
        }
    }

    pub fn new_meme_token(index: u32) -> Self {
        Self {
            address: format!("Meme{:040}", index),
            symbol: format!("MEME{}", index),
            name: format!("Meme Token {}", index),
            decimals: 9,
            supply: 1_000_000_000_000_000,
            price_usd: 0.0001 + (index as f64 * 0.00001),
            liquidity_usd: 1000.0 + (index as f64 * 100.0),
            volume_24h_usd: 5000.0 + (index as f64 * 500.0),
            age_minutes: (index % 10) + 1, // 1-10 minutes old
            holder_count: 50 + (index % 200),
            creator_verified: index % 5 == 0, // 20% verified
            risk_score: 0.3 + ((index % 70) as f64 / 100.0), // 0.3-0.99
        }
    }

    pub fn high_risk_token() -> Self {
        Self {
            address: "Risk1111111111111111111111111111111111111111".to_string(),
            symbol: "RISK".to_string(),
            name: "High Risk Token".to_string(),
            decimals: 9,
            supply: 1_000_000_000_000_000,
            price_usd: 0.00001,
            liquidity_usd: 100.0, // Very low liquidity
            volume_24h_usd: 50.0, // Very low volume
            age_minutes: 1,       // Very new
            holder_count: 5,      // Very few holders
            creator_verified: false,
            risk_score: 0.95, // Very high risk
        }
    }

    pub fn stable_token() -> Self {
        Self {
            address: "Stable111111111111111111111111111111111111111".to_string(),
            symbol: "STABLE".to_string(),
            name: "Stable Token".to_string(),
            decimals: 9,
            supply: 10_000_000_000_000_000,
            price_usd: 1.0,
            liquidity_usd: 1_000_000.0,  // High liquidity
            volume_24h_usd: 5_000_000.0, // High volume
            age_minutes: 1440,           // 24 hours old
            holder_count: 10000,         // Many holders
            creator_verified: true,
            risk_score: 0.1, // Low risk
        }
    }
}

/// Test trade data
#[derive(Debug, Clone)]
pub struct TestTrade {
    pub id: String,
    pub timestamp: u64,
    pub token: TestToken,
    pub action: TradeAction,
    pub amount_sol: f64,
    pub price_usd: f64,
    pub tx_signature: String,
    pub strategy: String,
    pub outcome: TradeOutcome,
    pub pnl_sol: Option<f64>,
    pub execution_time_ms: u64,
    pub gas_used: u64,
    pub slippage_percent: f64,
}

#[derive(Debug, Clone)]
pub enum TradeAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub enum TradeOutcome {
    Success,
    Failure,
    Partial,
}

impl TestTrade {
    pub fn successful_microbot_trade(index: u32) -> Self {
        let token = TestToken::new_meme_token(index);
        Self {
            id: format!("trade_{}", index),
            timestamp: 1640995200 + (index as u64 * 300), // 5 minute intervals
            token: token.clone(),
            action: TradeAction::Buy,
            amount_sol: 0.32, // 80% of 0.4 SOL
            price_usd: token.price_usd,
            tx_signature: format!("sig_{:064}", index),
            strategy: "microbot".to_string(),
            outcome: TradeOutcome::Success,
            pnl_sol: Some(0.05 + (index as f64 * 0.001)), // Profitable
            execution_time_ms: 150 + (index as u64 % 100),
            gas_used: 150000 + (index as u64 % 50000),
            slippage_percent: 0.5 + ((index % 20) as f64 * 0.1),
        }
    }

    pub fn failed_trade(index: u32) -> Self {
        let token = TestToken::high_risk_token();
        Self {
            id: format!("failed_trade_{}", index),
            timestamp: 1640995200 + (index as u64 * 600),
            token: token.clone(),
            action: TradeAction::Buy,
            amount_sol: 0.32,
            price_usd: token.price_usd,
            tx_signature: format!("fail_sig_{:064}", index),
            strategy: "microbot".to_string(),
            outcome: TradeOutcome::Failure,
            pnl_sol: Some(-0.02),    // Loss
            execution_time_ms: 5000, // Slow execution
            gas_used: 200000,
            slippage_percent: 15.0, // High slippage
        }
    }

    pub fn partial_trade(index: u32) -> Self {
        let token = TestToken::new_meme_token(index);
        Self {
            id: format!("partial_trade_{}", index),
            timestamp: 1640995200 + (index as u64 * 450),
            token: token.clone(),
            action: TradeAction::Buy,
            amount_sol: 0.16, // Only 50% filled
            price_usd: token.price_usd,
            tx_signature: format!("partial_sig_{:064}", index),
            strategy: "microbot".to_string(),
            outcome: TradeOutcome::Partial,
            pnl_sol: Some(0.01), // Small profit
            execution_time_ms: 800,
            gas_used: 175000,
            slippage_percent: 3.0,
        }
    }
}

/// Test market conditions
#[derive(Debug, Clone)]
pub struct TestMarketCondition {
    pub timestamp: u64,
    pub sol_price_usd: f64,
    pub total_volume_24h: f64,
    pub active_traders: u32,
    pub new_tokens_created: u32,
    pub average_liquidity: f64,
    pub volatility_index: f64,
    pub mev_activity_level: f64,
    pub network_congestion: f64,
}

impl TestMarketCondition {
    pub fn bull_market() -> Self {
        Self {
            timestamp: 1640995200,
            sol_price_usd: 120.0,              // High SOL price
            total_volume_24h: 2_000_000_000.0, // High volume
            active_traders: 50000,
            new_tokens_created: 500, // Many new tokens
            average_liquidity: 50000.0,
            volatility_index: 0.8, // High volatility
            mev_activity_level: 0.7,
            network_congestion: 0.6,
        }
    }

    pub fn bear_market() -> Self {
        Self {
            timestamp: 1640995200,
            sol_price_usd: 80.0,             // Low SOL price
            total_volume_24h: 500_000_000.0, // Low volume
            active_traders: 15000,
            new_tokens_created: 50, // Few new tokens
            average_liquidity: 10000.0,
            volatility_index: 0.3, // Low volatility
            mev_activity_level: 0.2,
            network_congestion: 0.2,
        }
    }

    pub fn high_volatility() -> Self {
        Self {
            timestamp: 1640995200,
            sol_price_usd: 100.0,
            total_volume_24h: 3_000_000_000.0, // Very high volume
            active_traders: 75000,
            new_tokens_created: 1000, // Many new tokens
            average_liquidity: 25000.0,
            volatility_index: 0.95,  // Very high volatility
            mev_activity_level: 0.9, // High MEV activity
            network_congestion: 0.8, // High congestion
        }
    }
}

/// Test opportunity data
#[derive(Debug, Clone)]
pub struct TestOpportunity {
    pub token: TestToken,
    pub detected_at: u64,
    pub confidence_score: f64,
    pub expected_profit_sol: f64,
    pub risk_assessment: RiskAssessment,
    pub time_sensitivity: TimeSensitivity,
    pub competition_level: f64,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub overall_score: f64,
    pub liquidity_risk: f64,
    pub volatility_risk: f64,
    pub age_risk: f64,
    pub holder_risk: f64,
    pub contract_risk: f64,
}

#[derive(Debug, Clone)]
pub enum TimeSensitivity {
    Immediate, // < 30 seconds
    Fast,      // < 2 minutes
    Medium,    // < 10 minutes
    Low,       // > 10 minutes
}

#[derive(Debug, Clone)]
pub enum RecommendedAction {
    Buy,
    Sell,
    Hold,
    Skip,
}

impl TestOpportunity {
    pub fn high_confidence_opportunity() -> Self {
        let token = TestToken::new_meme_token(1);
        Self {
            token: token.clone(),
            detected_at: 1640995200,
            confidence_score: 0.9,
            expected_profit_sol: 0.08,
            risk_assessment: RiskAssessment {
                overall_score: 0.2, // Low risk
                liquidity_risk: 0.1,
                volatility_risk: 0.3,
                age_risk: 0.2,
                holder_risk: 0.15,
                contract_risk: 0.1,
            },
            time_sensitivity: TimeSensitivity::Fast,
            competition_level: 0.3, // Low competition
            recommended_action: RecommendedAction::Buy,
        }
    }

    pub fn risky_opportunity() -> Self {
        let token = TestToken::high_risk_token();
        Self {
            token: token.clone(),
            detected_at: 1640995200,
            confidence_score: 0.4,
            expected_profit_sol: 0.15, // High potential profit
            risk_assessment: RiskAssessment {
                overall_score: 0.8, // High risk
                liquidity_risk: 0.9,
                volatility_risk: 0.8,
                age_risk: 0.9,
                holder_risk: 0.95,
                contract_risk: 0.7,
            },
            time_sensitivity: TimeSensitivity::Immediate,
            competition_level: 0.9, // High competition
            recommended_action: RecommendedAction::Skip,
        }
    }
}

/// Helper functions for generating test data
pub fn generate_test_tokens(count: usize) -> Vec<TestToken> {
    (0..count)
        .map(|i| TestToken::new_meme_token(i as u32))
        .collect()
}

pub fn generate_test_trades(count: usize) -> Vec<TestTrade> {
    (0..count)
        .map(|i| match i % 4 {
            0 => TestTrade::successful_microbot_trade(i as u32),
            1 => TestTrade::failed_trade(i as u32),
            2 => TestTrade::partial_trade(i as u32),
            _ => TestTrade::successful_microbot_trade(i as u32),
        })
        .collect()
}

pub fn generate_test_opportunities(count: usize) -> Vec<TestOpportunity> {
    (0..count)
        .map(|i| {
            if i % 3 == 0 {
                TestOpportunity::risky_opportunity()
            } else {
                TestOpportunity::high_confidence_opportunity()
            }
        })
        .collect()
}

pub fn create_test_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("strategy".to_string(), "microbot".to_string());
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("environment".to_string(), "test".to_string());
    metadata.insert("timestamp".to_string(), "2024-01-01T00:00:00Z".to_string());
    metadata
}

/// Test wallet addresses
pub const TEST_WALLET_1: &str = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
pub const TEST_WALLET_2: &str = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
pub const TEST_WALLET_3: &str = "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5";

/// Test transaction signatures
pub const TEST_TX_1: &str =
    "5j7s8K9mN2pQ3rT4uV5wX6yZ7a8B9c0D1e2F3g4H5i6J7k8L9m0N1o2P3q4R5s6T7u8V9w0X1y2Z3a4B5c6D7e8F";
pub const TEST_TX_2: &str =
    "2a3B4c5D6e7F8g9H0i1J2k3L4m5N6o7P8q9R0s1T2u3V4w5X6y7Z8a9B0c1D2e3F4g5H6i7J8k9L0m1N2o3P4q5R";

/// Test program IDs
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
pub const RAYDIUM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const SERUM_PROGRAM_ID: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

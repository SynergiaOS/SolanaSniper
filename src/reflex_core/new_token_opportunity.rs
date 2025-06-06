use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a brand new token opportunity detected on-chain
/// This is for tokens that are 0-60 seconds old
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTokenOpportunity {
    /// Token mint address
    pub token_address: String,
    
    /// Pool address where token was created
    pub pool_address: String,
    
    /// Token symbol (if available)
    pub token_symbol: Option<String>,
    
    /// Initial liquidity in SOL
    pub initial_liquidity_sol: f64,
    
    /// Initial liquidity in USD (estimated)
    pub initial_liquidity_usd: f64,
    
    /// Pool creation transaction signature
    pub creation_tx_signature: String,
    
    /// Block slot when pool was created
    pub creation_slot: u64,
    
    /// Timestamp when opportunity was detected
    pub detected_at: DateTime<Utc>,
    
    /// Age in seconds since pool creation
    pub age_seconds: u64,
    
    /// DEX where pool was created (Raydium, PumpFun, etc.)
    pub dex: String,
    
    /// Risk score (0.0 = highest risk, 1.0 = lowest risk)
    pub risk_score: f64,
    
    /// Whether this token has burned mint authority
    pub mint_authority_burned: bool,
    
    /// Whether this token has burned freeze authority
    pub freeze_authority_burned: bool,
    
    /// Initial market cap in USD (if calculable)
    pub initial_market_cap_usd: Option<f64>,
}

impl NewTokenOpportunity {
    /// Check if this opportunity is still fresh (< 60 seconds old)
    pub fn is_fresh(&self) -> bool {
        self.age_seconds < 60
    }
    
    /// Check if this opportunity passes basic safety checks
    pub fn is_safe(&self) -> bool {
        // Basic safety criteria for new tokens
        self.mint_authority_burned 
            && self.freeze_authority_burned
            && self.initial_liquidity_sol >= 1.0  // At least 1 SOL liquidity
            && self.risk_score >= 0.3  // Minimum risk threshold
    }
    
    /// Calculate priority score for execution order
    pub fn priority_score(&self) -> f64 {
        let mut score = 0.0;
        
        // Higher liquidity = higher priority
        score += (self.initial_liquidity_sol / 10.0).min(1.0) * 0.3;
        
        // Lower age = higher priority
        score += (1.0 - (self.age_seconds as f64 / 60.0)) * 0.4;
        
        // Higher risk score = higher priority
        score += self.risk_score * 0.3;
        
        score.min(1.0)
    }
    
    /// Generate Redis key for storing this opportunity
    pub fn redis_key(&self) -> String {
        format!("new_token_opportunity:{}", self.token_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fresh_opportunity() {
        let opp = NewTokenOpportunity {
            token_address: "test123".to_string(),
            pool_address: "pool123".to_string(),
            token_symbol: Some("TEST".to_string()),
            initial_liquidity_sol: 5.0,
            initial_liquidity_usd: 1000.0,
            creation_tx_signature: "sig123".to_string(),
            creation_slot: 12345,
            detected_at: Utc::now(),
            age_seconds: 30,
            dex: "Raydium".to_string(),
            risk_score: 0.8,
            mint_authority_burned: true,
            freeze_authority_burned: true,
            initial_market_cap_usd: Some(50000.0),
        };
        
        assert!(opp.is_fresh());
        assert!(opp.is_safe());
        assert!(opp.priority_score() > 0.5);
    }
}

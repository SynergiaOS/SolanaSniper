/*!
🎯 Opportunity Structures - Data models for validated trading opportunities

This module defines the core data structures that represent fully validated
trading opportunities combining quantitative and qualitative intelligence.
*/

use crate::data_fetcher::soul_meteor_scanner::HotCandidate;
use crate::models::TextualData;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Status of an opportunity in the pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpportunityStatus {
    /// Newly discovered from Soul Meteor Scanner
    Discovered,
    /// Currently being validated by Crawl4AI
    Validating,
    /// Successfully validated with positive sentiment
    Validated,
    /// Rejected due to negative sentiment or lack of data
    Rejected,
    /// Approved for trading execution
    Approved,
    /// Currently being executed
    Executing,
    /// Successfully executed
    Executed,
    /// Execution failed
    Failed,
}

/// Sentiment analysis result from Crawl4AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentReport {
    /// Overall sentiment score (-1.0 to 1.0)
    pub aggregated_score: f64,
    /// Confidence level of the sentiment analysis (0.0 to 1.0)
    pub confidence: f64,
    /// Detected patterns or keywords
    pub patterns: Vec<String>,
    /// Number of sources analyzed
    pub sources_count: u32,
    /// Textual data from Crawl4AI
    pub textual_data: Option<TextualData>,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

impl Default for SentimentReport {
    fn default() -> Self {
        Self {
            aggregated_score: 0.0,
            confidence: 0.0,
            patterns: vec![],
            sources_count: 0,
            textual_data: None,
            analyzed_at: Utc::now(),
        }
    }
}

/// Fully validated trading opportunity combining quantitative and qualitative data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedOpportunity {
    /// Unique identifier for this opportunity
    pub id: String,
    /// Hot candidate from Soul Meteor Scanner
    pub candidate: HotCandidate,
    /// Sentiment analysis from Crawl4AI
    pub sentiment: SentimentReport,
    /// Current status in the pipeline
    pub status: OpportunityStatus,
    /// Combined opportunity score (0.0 to 10.0)
    pub combined_score: f64,
    /// Recommended strategy type
    pub recommended_strategy: StrategyType,
    /// Risk assessment
    pub risk_level: RiskLevel,
    /// Discovery timestamp
    pub discovered_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Strategy recommendation for the opportunity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    /// Buy the token (sniper strategy)
    TokenSniper,
    /// Provide liquidity to earn fees
    LiquidityProvider,
    /// Arbitrage opportunity
    Arbitrage,
    /// Hold and monitor
    Monitor,
    /// Avoid this opportunity
    Avoid,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    /// Very low risk, high confidence
    VeryLow,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Very high risk, avoid
    VeryHigh,
}

impl ValidatedOpportunity {
    /// Create a new validated opportunity
    pub fn new(candidate: HotCandidate, sentiment: SentimentReport) -> Self {
        let id = format!("{}_{}", candidate.address, Utc::now().timestamp());
        let combined_score = Self::calculate_combined_score(&candidate, &sentiment);
        let recommended_strategy = Self::determine_strategy(&candidate, &sentiment);
        let risk_level = Self::assess_risk(&candidate, &sentiment);
        
        Self {
            id,
            candidate,
            sentiment,
            status: OpportunityStatus::Validated,
            combined_score,
            recommended_strategy,
            risk_level,
            discovered_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Calculate combined score from quantitative and qualitative data
    fn calculate_combined_score(candidate: &HotCandidate, sentiment: &SentimentReport) -> f64 {
        // Weighted combination of opportunity score and sentiment
        let quantitative_weight = 0.6;
        let qualitative_weight = 0.4;
        
        // Normalize opportunity score to 0-10 scale
        let normalized_opportunity = (candidate.opportunity_score / 4.0) * 10.0;
        
        // Normalize sentiment score to 0-10 scale
        let normalized_sentiment = ((sentiment.aggregated_score + 1.0) / 2.0) * 10.0;
        
        // Apply confidence factor
        let confidence_factor = sentiment.confidence;
        
        let combined = (normalized_opportunity * quantitative_weight) + 
                      (normalized_sentiment * qualitative_weight * confidence_factor);
        
        combined.min(10.0).max(0.0)
    }

    /// Determine recommended strategy based on data
    fn determine_strategy(candidate: &HotCandidate, sentiment: &SentimentReport) -> StrategyType {
        // High APR suggests liquidity provision
        if candidate.apr >= 25.0 && candidate.liquidity_usd >= 100000.0 {
            return StrategyType::LiquidityProvider;
        }
        
        // High volume with positive sentiment suggests sniping
        if candidate.volume_24h >= 1000000.0 && sentiment.aggregated_score > 0.3 {
            return StrategyType::TokenSniper;
        }
        
        // Low sentiment or data suggests monitoring
        if sentiment.aggregated_score < 0.1 || sentiment.confidence < 0.5 {
            return StrategyType::Monitor;
        }
        
        // Negative sentiment suggests avoiding
        if sentiment.aggregated_score < -0.2 {
            return StrategyType::Avoid;
        }
        
        // Default to monitoring for unclear cases
        StrategyType::Monitor
    }

    /// Assess risk level based on multiple factors
    fn assess_risk(candidate: &HotCandidate, sentiment: &SentimentReport) -> RiskLevel {
        let mut risk_score = 0;
        
        // Liquidity risk
        if candidate.liquidity_usd < 50000.0 {
            risk_score += 2;
        } else if candidate.liquidity_usd < 200000.0 {
            risk_score += 1;
        }
        
        // Sentiment risk
        if sentiment.aggregated_score < -0.1 {
            risk_score += 2;
        } else if sentiment.aggregated_score < 0.2 {
            risk_score += 1;
        }
        
        // Confidence risk
        if sentiment.confidence < 0.3 {
            risk_score += 2;
        } else if sentiment.confidence < 0.6 {
            risk_score += 1;
        }
        
        // Volume/Liquidity ratio risk (too high suggests manipulation)
        let volume_liquidity_ratio = candidate.volume_24h / candidate.liquidity_usd;
        if volume_liquidity_ratio > 50.0 {
            risk_score += 2;
        } else if volume_liquidity_ratio > 20.0 {
            risk_score += 1;
        }
        
        match risk_score {
            0..=1 => RiskLevel::VeryLow,
            2..=3 => RiskLevel::Low,
            4..=5 => RiskLevel::Medium,
            6..=7 => RiskLevel::High,
            _ => RiskLevel::VeryHigh,
        }
    }

    /// Update the status of the opportunity
    pub fn update_status(&mut self, new_status: OpportunityStatus) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }

    /// Check if opportunity is actionable
    pub fn is_actionable(&self) -> bool {
        matches!(self.status, OpportunityStatus::Validated | OpportunityStatus::Approved) &&
        matches!(self.recommended_strategy, StrategyType::TokenSniper | StrategyType::LiquidityProvider) &&
        !matches!(self.risk_level, RiskLevel::VeryHigh)
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "{} | Score: {:.1} | Strategy: {:?} | Risk: {:?} | Liquidity: ${:.0} | Volume: ${:.0}",
            self.candidate.name,
            self.combined_score,
            self.recommended_strategy,
            self.risk_level,
            self.candidate.liquidity_usd,
            self.candidate.volume_24h
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opportunity_creation() {
        let candidate = HotCandidate {
            name: "TEST-SOL".to_string(),
            address: "test123".to_string(),
            liquidity_usd: 100000.0,
            volume_24h: 500000.0,
            fees_24h: 1000.0,
            fee_tvl_ratio_24h: 1.0,
            apr: 10.0,
            apy: 100.0,
            opportunity_score: 3.5,
            mint_x: "mint1".to_string(),
            mint_y: "mint2".to_string(),
            current_price: 1.0,
        };

        let sentiment = SentimentReport {
            aggregated_score: 0.5,
            confidence: 0.8,
            patterns: vec!["POSITIVE".to_string()],
            sources_count: 5,
            textual_data: None,
            analyzed_at: Utc::now(),
        };

        let opportunity = ValidatedOpportunity::new(candidate, sentiment);
        
        assert!(opportunity.combined_score > 0.0);
        assert!(opportunity.is_actionable());
        assert_eq!(opportunity.status, OpportunityStatus::Validated);
    }

    #[test]
    fn test_risk_assessment() {
        let low_liquidity_candidate = HotCandidate {
            name: "RISKY-SOL".to_string(),
            address: "risky123".to_string(),
            liquidity_usd: 10000.0, // Low liquidity
            volume_24h: 100000.0,
            fees_24h: 100.0,
            fee_tvl_ratio_24h: 1.0,
            apr: 5.0,
            apy: 50.0,
            opportunity_score: 2.0,
            mint_x: "mint1".to_string(),
            mint_y: "mint2".to_string(),
            current_price: 1.0,
        };

        let negative_sentiment = SentimentReport {
            aggregated_score: -0.3, // Negative sentiment
            confidence: 0.4, // Low confidence
            patterns: vec!["NEGATIVE".to_string()],
            sources_count: 2,
            textual_data: None,
            analyzed_at: Utc::now(),
        };

        let opportunity = ValidatedOpportunity::new(low_liquidity_candidate, negative_sentiment);
        
        assert!(matches!(opportunity.risk_level, RiskLevel::High | RiskLevel::VeryHigh));
        assert_eq!(opportunity.recommended_strategy, StrategyType::Avoid);
    }
}

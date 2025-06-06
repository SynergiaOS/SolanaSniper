/*!
🐍 Python Compatibility Models
Data structures that match the JSON format from Python scripts
*/

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Raw opportunity as stored by Python Soul Meteor Scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonRawOpportunity {
    pub id: String,
    pub candidate: PythonHotCandidate,
    pub discovered_at: String, // ISO format string from Python
    pub expires_at: String,    // ISO format string from Python
    pub status: String,        // "Pending", "Validating", etc.
}

/// Hot candidate data from Python Soul Meteor Scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonHotCandidate {
    pub address: String,
    pub name: String,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub fees_24h: f64,
    pub fee_tvl_ratio_24h: f64,
    pub apr: f64,
    pub apy: f64,
    pub mint_x: String,
    pub mint_y: String,
    pub current_price: f64,
    pub is_blacklisted: bool,
    pub hide: bool,
    pub opportunity_score: f64,
}

impl PythonRawOpportunity {
    /// Convert to Rust native format
    pub fn to_rust_format(&self) -> Result<crate::models::persistent_state::RawOpportunity, String> {
        // Parse discovered_at timestamp
        let discovered_at = if let Ok(dt) = DateTime::parse_from_rfc3339(&self.discovered_at) {
            dt.with_timezone(&Utc)
        } else if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&self.discovered_at, "%Y-%m-%dT%H:%M:%S%.f") {
            DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc)
        } else {
            return Err(format!("Failed to parse discovered_at: {}", self.discovered_at));
        };

        // Parse expires_at timestamp
        let expires_at = if let Ok(dt) = DateTime::parse_from_rfc3339(&self.expires_at) {
            dt.with_timezone(&Utc)
        } else if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&self.expires_at, "%Y-%m-%dT%H:%M:%S%.f") {
            DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc)
        } else {
            return Err(format!("Failed to parse expires_at: {}", self.expires_at));
        };

        // Convert status
        let status = match self.status.as_str() {
            "Pending" => crate::models::persistent_state::RawOpportunityStatus::Pending,
            "Validating" => crate::models::persistent_state::RawOpportunityStatus::Validating,
            "Processed" => crate::models::persistent_state::RawOpportunityStatus::Processed,
            "Expired" => crate::models::persistent_state::RawOpportunityStatus::Expired,
            _ => crate::models::persistent_state::RawOpportunityStatus::Pending,
        };

        // Convert candidate
        let candidate = crate::data_fetcher::soul_meteor_scanner::HotCandidate {
            address: self.candidate.address.clone(),
            name: self.candidate.name.clone(),
            liquidity_usd: self.candidate.liquidity_usd,
            volume_24h: self.candidate.volume_24h,
            fees_24h: self.candidate.fees_24h,
            fee_tvl_ratio_24h: self.candidate.fee_tvl_ratio_24h,
            apr: self.candidate.apr,
            apy: self.candidate.apy,
            mint_x: self.candidate.mint_x.clone(),
            mint_y: self.candidate.mint_y.clone(),
            current_price: self.candidate.current_price,
            is_blacklisted: self.candidate.is_blacklisted,
            hide: self.candidate.hide,
            opportunity_score: self.candidate.opportunity_score,
        };

        Ok(crate::models::persistent_state::RawOpportunity {
            id: self.id.clone(),
            candidate,
            discovered_at,
            expires_at,
            status,
        })
    }

    /// Get summary for logging
    pub fn summary(&self) -> String {
        format!(
            "{} (${:.0}k liq, ${:.0}k vol, {:.1}% APR, score: {:.2})",
            self.candidate.name,
            self.candidate.liquidity_usd / 1000.0,
            self.candidate.volume_24h / 1000.0,
            self.candidate.apr,
            self.candidate.opportunity_score
        )
    }

    /// Check if opportunity is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        // Try RFC3339 first (with timezone)
        if let Ok(expires_at) = DateTime::parse_from_rfc3339(&self.expires_at) {
            return Utc::now() < expires_at.with_timezone(&Utc);
        }

        // If that fails, try parsing as naive datetime and assume UTC
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&self.expires_at, "%Y-%m-%dT%H:%M:%S%.f") {
            let expires_at_utc = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
            return Utc::now() < expires_at_utc;
        }

        false
    }

    /// Get age in minutes
    pub fn age_minutes(&self) -> Option<i64> {
        // Try RFC3339 first (with timezone)
        if let Ok(discovered_at) = DateTime::parse_from_rfc3339(&self.discovered_at) {
            let duration = Utc::now().signed_duration_since(discovered_at.with_timezone(&Utc));
            return Some(duration.num_minutes());
        }

        // If that fails, try parsing as naive datetime and assume UTC
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&self.discovered_at, "%Y-%m-%dT%H:%M:%S%.f") {
            let discovered_at_utc = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
            let duration = Utc::now().signed_duration_since(discovered_at_utc);
            return Some(duration.num_minutes());
        }

        None
    }
}

impl PythonHotCandidate {
    /// Check if this is a high-quality opportunity
    pub fn is_high_quality(&self) -> bool {
        self.opportunity_score >= 3.0 
            && self.liquidity_usd >= 20000.0 
            && !self.is_blacklisted 
            && !self.hide
    }

    /// Get volume to liquidity ratio
    pub fn volume_liquidity_ratio(&self) -> f64 {
        if self.liquidity_usd > 0.0 {
            self.volume_24h / self.liquidity_usd
        } else {
            0.0
        }
    }

    /// Check if this looks like a pump and dump
    pub fn is_suspicious(&self) -> bool {
        // Very high volume/liquidity ratio might indicate P&D
        let vol_liq_ratio = self.volume_liquidity_ratio();
        vol_liq_ratio > 50.0 || self.fee_tvl_ratio_24h > 1000.0
    }
}

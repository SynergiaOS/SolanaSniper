use crate::config::AppConfig;
use crate::data_fetcher::{
    pumpfun_client::PumpFunClient,
    raydium_client::RaydiumClient,
    meteora_client::MeteoraClient,
};
use crate::models::TradingResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn, instrument};
use chrono::{DateTime, Utc};

/// Market Scanner for actively discovering trading opportunities
/// Scans multiple DEXs and sources for tokens meeting strategy criteria
pub struct MarketScanner {
    config: AppConfig,
    pumpfun_client: Arc<PumpFunClient>,
    raydium_client: Arc<RaydiumClient>,
    meteora_client: Arc<MeteoraClient>,
    opportunity_sender: mpsc::Sender<PotentialOpportunity>,
    scan_interval: Duration,
    last_scan_time: Option<DateTime<Utc>>,
}

/// Represents a potential trading opportunity discovered by the scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialOpportunity {
    pub token_address: String,
    pub symbol: String,
    pub source: String, // "pumpfun", "raydium", "meteora"
    pub opportunity_type: OpportunityType,
    pub market_cap: Option<f64>,
    pub volume_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub holder_count: Option<u32>,
    pub price: f64,
    pub price_change_24h: Option<f64>,
    pub confidence_score: f64, // 0.0 - 1.0
    pub discovered_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    NewToken,           // Newly launched token
    LiquiditySpike,     // Sudden liquidity increase
    VolumeSpike,        // Unusual volume activity
    PriceBreakout,      // Technical breakout pattern
    ArbitrageGap,       // Price discrepancy between DEXs
    DLMMRebalance,      // DLMM bin rebalancing opportunity
}

/// Scanning criteria for filtering opportunities
#[derive(Debug, Clone)]
pub struct ScanCriteria {
    pub min_market_cap: Option<f64>,
    pub max_market_cap: Option<f64>,
    pub min_volume_24h: Option<f64>,
    pub min_liquidity: Option<f64>,
    pub min_holder_count: Option<u32>,
    pub max_age_hours: Option<u32>, // For new tokens
    pub min_confidence_score: f64,
}

impl MarketScanner {
    /// Create new market scanner
    pub fn new(
        config: AppConfig,
        pumpfun_client: Arc<PumpFunClient>,
        raydium_client: Arc<RaydiumClient>,
        meteora_client: Arc<MeteoraClient>,
        opportunity_sender: mpsc::Sender<PotentialOpportunity>,
    ) -> Self {
        Self {
            config,
            pumpfun_client,
            raydium_client,
            meteora_client,
            opportunity_sender,
            scan_interval: Duration::from_secs(30), // Scan every 30 seconds
            last_scan_time: None,
        }
    }

    /// Start the market scanner
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> TradingResult<()> {
        info!("🔍 Starting Market Scanner");
        
        let mut scan_timer = interval(self.scan_interval);
        
        loop {
            scan_timer.tick().await;
            
            if let Err(e) = self.scan_for_opportunities().await {
                error!("Market scanner error: {}", e);
                // Continue scanning even on errors
            }
        }
    }

    /// Scan all sources for trading opportunities
    #[instrument(skip(self))]
    async fn scan_for_opportunities(&mut self) -> TradingResult<()> {
        debug!("🔍 Scanning for opportunities...");
        
        let scan_start = Utc::now();
        let mut total_opportunities = 0;

        // Scan PumpFun for new tokens
        match self.scan_pumpfun().await {
            Ok(count) => {
                total_opportunities += count;
                debug!("Found {} opportunities on PumpFun", count);
            }
            Err(e) => warn!("PumpFun scan failed: {}", e),
        }

        // Scan Raydium for liquidity changes
        match self.scan_raydium().await {
            Ok(count) => {
                total_opportunities += count;
                debug!("Found {} opportunities on Raydium", count);
            }
            Err(e) => warn!("Raydium scan failed: {}", e),
        }

        // Scan Meteora for DLMM opportunities
        match self.scan_meteora().await {
            Ok(count) => {
                total_opportunities += count;
                debug!("Found {} opportunities on Meteora", count);
            }
            Err(e) => warn!("Meteora scan failed: {}", e),
        }

        self.last_scan_time = Some(scan_start);
        
        if total_opportunities > 0 {
            info!("🎯 Scan complete: {} opportunities found", total_opportunities);
        }

        Ok(())
    }

    /// Scan PumpFun for new token opportunities
    async fn scan_pumpfun(&self) -> TradingResult<usize> {
        let criteria = self.get_pumpfun_criteria();
        let tokens = self.pumpfun_client.get_trending_tokens(Some(50)).await?;
        
        let mut opportunities = Vec::new();
        
        for token in tokens {
            if self.meets_criteria(&token, &criteria) {
                let price = self.pumpfun_client.calculate_token_price(&token);
                let estimated_liquidity = (token.virtual_sol_reserves as f64 / 1_000_000_000.0) * 2.0; // Rough estimate

                let opportunity = PotentialOpportunity {
                    token_address: token.mint.clone(),
                    symbol: token.symbol.clone(),
                    source: "pumpfun".to_string(),
                    opportunity_type: OpportunityType::NewToken,
                    market_cap: Some(token.market_cap),
                    volume_24h: None, // Would need to calculate from trades
                    liquidity_usd: Some(estimated_liquidity),
                    holder_count: None,
                    price,
                    price_change_24h: None,
                    confidence_score: self.calculate_pumpfun_confidence(&token),
                    discovered_at: Utc::now(),
                    metadata: HashMap::new(),
                };

                opportunities.push(opportunity);
            }
        }

        // Send opportunities
        for opportunity in &opportunities {
            if let Err(e) = self.opportunity_sender.send(opportunity.clone()).await {
                error!("Failed to send opportunity: {}", e);
            }
        }

        Ok(opportunities.len())
    }

    /// Scan Raydium for liquidity opportunities
    async fn scan_raydium(&self) -> TradingResult<usize> {
        // Implementation for Raydium scanning
        // This would check for new pools, liquidity spikes, etc.
        Ok(0) // Placeholder
    }

    /// Scan Meteora for DLMM opportunities
    async fn scan_meteora(&self) -> TradingResult<usize> {
        // Implementation for Meteora DLMM scanning
        // This would check for bin rebalancing opportunities
        Ok(0) // Placeholder
    }

    /// Get scanning criteria for PumpFun
    fn get_pumpfun_criteria(&self) -> ScanCriteria {
        ScanCriteria {
            min_market_cap: Some(10000.0),      // $10k minimum
            max_market_cap: Some(1000000.0),    // $1M maximum
            min_volume_24h: Some(5000.0),       // $5k volume
            min_liquidity: Some(2000.0),        // $2k liquidity
            min_holder_count: None,
            max_age_hours: Some(24),            // Max 24 hours old
            min_confidence_score: 0.6,          // 60% confidence minimum
        }
    }

    /// Check if token meets scanning criteria
    fn meets_criteria(&self, token: &crate::data_fetcher::pumpfun_client::PumpFunToken, criteria: &ScanCriteria) -> bool {
        // Market cap check
        if let Some(min_mc) = criteria.min_market_cap {
            if token.market_cap < min_mc { return false; }
        }

        if let Some(max_mc) = criteria.max_market_cap {
            if token.market_cap > max_mc { return false; }
        }

        // Volume check - skip for now as we don't have 24h volume in the token struct
        // Would need to fetch trades separately

        // Liquidity check (estimated from SOL reserves)
        if let Some(min_liq) = criteria.min_liquidity {
            let estimated_liquidity = (token.virtual_sol_reserves as f64 / 1_000_000_000.0) * 2.0;
            if estimated_liquidity < min_liq { return false; }
        }

        // Age check
        if let Some(max_age_hours) = criteria.max_age_hours {
            let token_age_hours = (Utc::now().timestamp() - token.created_timestamp) / 3600;
            if token_age_hours > max_age_hours as i64 { return false; }
        }

        true
    }

    /// Calculate confidence score for PumpFun token
    fn calculate_pumpfun_confidence(&self, token: &crate::data_fetcher::pumpfun_client::PumpFunToken) -> f64 {
        let mut score: f64 = 0.0;

        // Liquidity factor (0.3 weight) - based on SOL reserves
        let sol_reserves = token.virtual_sol_reserves as f64 / 1_000_000_000.0;
        let liquidity_score = (sol_reserves / 50.0).min(1.0); // Normalize to 50 SOL
        score += liquidity_score * 0.3;

        // Market cap factor (0.3 weight)
        let mc_score = ((token.market_cap - 10000.0) / 90000.0).clamp(0.0, 1.0); // $10k-$100k range
        score += mc_score * 0.3;

        // Age factor (0.2 weight) - newer tokens get higher score
        let token_age_hours = (Utc::now().timestamp() - token.created_timestamp) / 3600;
        let age_score = (1.0 - (token_age_hours as f64 / 24.0)).clamp(0.0, 1.0); // 24h window
        score += age_score * 0.2;

        // Completion status (0.2 weight)
        if !token.complete {
            score += 0.2; // Bonus for tokens still in bonding curve
        }

        score.clamp(0.0, 1.0)
    }
}

impl Default for ScanCriteria {
    fn default() -> Self {
        Self {
            min_market_cap: Some(5000.0),
            max_market_cap: Some(500000.0),
            min_volume_24h: Some(1000.0),
            min_liquidity: Some(1000.0),
            min_holder_count: Some(10),
            max_age_hours: Some(48),
            min_confidence_score: 0.5,
        }
    }
}

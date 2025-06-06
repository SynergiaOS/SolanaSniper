/*!
🧠 Pipeline Controller - Orchestrates the complete trading intelligence flow

This is the central brain that coordinates:
1. Soul Meteor Scanner (quantitative opportunity detection)
2. Crawl4AI Validator (qualitative sentiment validation)
3. Decision Engine (strategy selection and execution planning)
*/

use crate::data_fetcher::soul_meteor_scanner::{SoulMeteorScanner, SoulMeteorScannerConfig};
use crate::data_fetcher::textual_data_fetcher::{TextualDataFetcher, TextualDataFetcherConfig};
use crate::pipeline::opportunity::{ValidatedOpportunity, SentimentReport};
use crate::pipeline::decision_engine::{DecisionEngine, TradingDecision, DecisionEngineConfig};
use crate::models::{TradingResult, TokenInfo};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use tokio::time::{Duration, sleep};

/// Configuration for the pipeline controller
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Soul Meteor Scanner configuration
    pub soul_meteor_config: SoulMeteorScannerConfig,
    /// Crawl4AI configuration
    pub crawl4ai_config: TextualDataFetcherConfig,
    /// Decision engine configuration
    pub decision_config: DecisionEngineConfig,
    /// Minimum sentiment score to proceed (0.0 to 1.0)
    pub min_sentiment_score: f64,
    /// Minimum sentiment confidence to proceed (0.0 to 1.0)
    pub min_sentiment_confidence: f64,
    /// Maximum candidates to validate per cycle
    pub max_validation_candidates: usize,
    /// Cycle interval in seconds
    pub cycle_interval_seconds: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            soul_meteor_config: SoulMeteorScannerConfig::default(),
            crawl4ai_config: TextualDataFetcherConfig::default(),
            decision_config: DecisionEngineConfig::default(),
            min_sentiment_score: 0.1,
            min_sentiment_confidence: 0.5,
            max_validation_candidates: 10,
            cycle_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Pipeline execution statistics
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub cycles_completed: u64,
    pub total_candidates_found: u64,
    pub total_candidates_validated: u64,
    pub total_opportunities_created: u64,
    pub total_decisions_made: u64,
    pub last_cycle_duration_ms: u64,
    pub average_cycle_duration_ms: u64,
}

/// Main pipeline controller
pub struct PipelineController {
    config: PipelineConfig,
    soul_meteor_scanner: SoulMeteorScanner,
    crawl4ai_fetcher: TextualDataFetcher,
    decision_engine: DecisionEngine,
    stats: PipelineStats,
    active_opportunities: HashMap<String, ValidatedOpportunity>,
}

impl PipelineController {
    /// Create new pipeline controller with default configuration
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
            soul_meteor_scanner: SoulMeteorScanner::new(),
            crawl4ai_fetcher: TextualDataFetcher::new(),
            decision_engine: DecisionEngine::new(),
            stats: PipelineStats::default(),
            active_opportunities: HashMap::new(),
        }
    }

    /// Create new pipeline controller with custom configuration
    pub fn with_config(config: PipelineConfig) -> Self {
        Self {
            soul_meteor_scanner: SoulMeteorScanner::with_config(config.soul_meteor_config.clone()),
            crawl4ai_fetcher: TextualDataFetcher::with_config(config.crawl4ai_config.clone()),
            decision_engine: DecisionEngine::with_config(config.decision_config.clone()),
            config,
            stats: PipelineStats::default(),
            active_opportunities: HashMap::new(),
        }
    }

    /// Run a single pipeline cycle
    pub async fn run_cycle(&mut self) -> TradingResult<Vec<TradingDecision>> {
        let cycle_start = std::time::Instant::now();
        
        info!("🚀 === STARTING PIPELINE CYCLE #{} ===", self.stats.cycles_completed + 1);

        // STEP 1: Discover hot candidates with Soul Meteor Scanner
        let candidates = match self.discover_candidates().await {
            Ok(candidates) => candidates,
            Err(e) => {
                error!("❌ Failed to discover candidates: {}", e);
                return Err(e);
            }
        };

        if candidates.is_empty() {
            info!("📊 No candidates found in this cycle");
            self.update_cycle_stats(cycle_start, 0, 0, 0);
            return Ok(vec![]);
        }

        // STEP 2: Validate candidates with Crawl4AI
        let validated_opportunities = self.validate_candidates(candidates).await?;

        // STEP 3: Generate trading decisions
        let decisions = self.generate_decisions(&validated_opportunities).await?;

        // Update statistics
        let validated_count = validated_opportunities.len() as u64;
        let decisions_count = decisions.len() as u64;
        self.update_cycle_stats(cycle_start, self.stats.total_candidates_found, validated_count, decisions_count);

        info!("✅ === CYCLE COMPLETED: {} decisions generated ===", decisions.len());
        
        Ok(decisions)
    }

    /// Discover hot candidates using Soul Meteor Scanner
    async fn discover_candidates(&mut self) -> TradingResult<Vec<crate::data_fetcher::soul_meteor_scanner::HotCandidate>> {
        info!("🕸️ [STEP 1/3] Discovering hot candidates with Soul Meteor Scanner...");

        let candidates = self.soul_meteor_scanner.scan_for_opportunities().await?;
        
        self.stats.total_candidates_found += candidates.len() as u64;
        
        info!("✅ Found {} hot candidates", candidates.len());
        
        // Log top candidates
        for (i, candidate) in candidates.iter().take(5).enumerate() {
            debug!(
                "  {}. {} | Score: {:.2} | Liquidity: ${:.0} | Volume: ${:.0}",
                i + 1,
                candidate.name,
                candidate.opportunity_score,
                candidate.liquidity_usd,
                candidate.volume_24h
            );
        }

        Ok(candidates)
    }

    /// Validate candidates using Crawl4AI sentiment analysis
    async fn validate_candidates(
        &mut self,
        candidates: Vec<crate::data_fetcher::soul_meteor_scanner::HotCandidate>,
    ) -> TradingResult<Vec<ValidatedOpportunity>> {
        info!("🔬 [STEP 2/3] Validating {} candidates with Crawl4AI...", candidates.len());

        let mut validated_opportunities = Vec::new();
        let max_candidates = self.config.max_validation_candidates.min(candidates.len());

        for (i, candidate) in candidates.into_iter().take(max_candidates).enumerate() {
            info!("  Validating {}/{}: {}", i + 1, max_candidates, candidate.name);

            match self.validate_single_candidate(&candidate).await {
                Ok(Some(opportunity)) => {
                    info!("    ✅ VALIDATED: Score {:.2}, Sentiment {:.2}", 
                          opportunity.combined_score, opportunity.sentiment.aggregated_score);
                    
                    // Store in active opportunities
                    self.active_opportunities.insert(opportunity.id.clone(), opportunity.clone());
                    validated_opportunities.push(opportunity);
                }
                Ok(None) => {
                    info!("    ⛔ REJECTED: Failed sentiment validation");
                }
                Err(e) => {
                    warn!("    ⚠️ ERROR: Failed to validate {}: {}", candidate.name, e);
                }
            }

            // Small delay to avoid overwhelming the Crawl4AI service
            sleep(Duration::from_millis(500)).await;
        }

        self.stats.total_candidates_validated += validated_opportunities.len() as u64;
        
        info!("✅ Validated {}/{} candidates successfully", 
              validated_opportunities.len(), max_candidates);

        Ok(validated_opportunities)
    }

    /// Validate a single candidate with Crawl4AI
    async fn validate_single_candidate(
        &self,
        candidate: &crate::data_fetcher::soul_meteor_scanner::HotCandidate,
    ) -> TradingResult<Option<ValidatedOpportunity>> {
        // Create TokenInfo for Crawl4AI
        let token_info = crate::models::TokenInfo {
            symbol: candidate.name.clone(),
            address: candidate.address.clone(),
            name: Some(candidate.name.clone()),
            price: candidate.current_price,
            market_cap: Some(candidate.liquidity_usd),
            volume_24h: Some(candidate.volume_24h),
            liquidity: Some(candidate.liquidity_usd),
            age_hours: None, // Would need to calculate from creation time
        };

        // Fetch textual data
        match self.crawl4ai_fetcher.fetch_textual_data(&token_info).await {
            Ok(textual_data) => {
                // Convert to sentiment report
                let sentiment = self.convert_to_sentiment_report(&textual_data);
                
                // Check if sentiment meets minimum criteria
                if sentiment.aggregated_score >= self.config.min_sentiment_score &&
                   sentiment.confidence >= self.config.min_sentiment_confidence {
                    
                    let opportunity = ValidatedOpportunity::new(candidate.clone(), sentiment);
                    Ok(Some(opportunity))
                } else {
                    debug!("Sentiment below threshold: score={:.2}, confidence={:.2}", 
                           sentiment.aggregated_score, sentiment.confidence);
                    Ok(None)
                }
            }
            Err(e) => {
                debug!("Crawl4AI validation failed: {}", e);
                Ok(None) // Don't fail the entire cycle for one candidate
            }
        }
    }

    /// Convert TextualData to SentimentReport
    fn convert_to_sentiment_report(&self, textual_data: &crate::models::TextualData) -> SentimentReport {
        // Use aggregated sentiment from TextualData if available
        let aggregated_score = textual_data.aggregated_sentiment.overall_score;

        let confidence = if textual_data.sources.is_empty() {
            0.0
        } else {
            // Calculate confidence based on number of sources and sentiment consistency
            let source_factor = (textual_data.sources.len() as f64 / 10.0).min(1.0);
            source_factor * 0.8 // Base confidence
        };

        let patterns = textual_data.sources.iter()
            .map(|s| s.source_type.clone())
            .collect();

        SentimentReport {
            aggregated_score,
            confidence,
            patterns,
            sources_count: textual_data.sources.len() as u32,
            textual_data: Some(textual_data.clone()),
            analyzed_at: chrono::Utc::now(),
        }
    }

    /// Generate trading decisions for validated opportunities
    async fn generate_decisions(&mut self, opportunities: &[ValidatedOpportunity]) -> TradingResult<Vec<TradingDecision>> {
        info!("🤖 [STEP 3/3] Generating trading decisions for {} opportunities...", opportunities.len());

        let mut decisions = Vec::new();

        for opportunity in opportunities {
            match self.decision_engine.analyze_opportunity(opportunity).await {
                Ok(decision) => {
                    info!("  ✅ Decision for {}: {:?}", 
                          opportunity.candidate.name, decision.decision_type);
                    decisions.push(decision);
                }
                Err(e) => {
                    warn!("  ⚠️ Failed to generate decision for {}: {}", 
                          opportunity.candidate.name, e);
                }
            }
        }

        self.stats.total_decisions_made += decisions.len() as u64;
        
        info!("✅ Generated {} trading decisions", decisions.len());

        Ok(decisions)
    }

    /// Update cycle statistics
    fn update_cycle_stats(&mut self, cycle_start: std::time::Instant, candidates: u64, validated: u64, decisions: u64) {
        let duration = cycle_start.elapsed();
        let duration_ms = duration.as_millis() as u64;
        
        self.stats.cycles_completed += 1;
        self.stats.last_cycle_duration_ms = duration_ms;
        
        // Update average duration
        if self.stats.cycles_completed == 1 {
            self.stats.average_cycle_duration_ms = duration_ms;
        } else {
            self.stats.average_cycle_duration_ms = 
                (self.stats.average_cycle_duration_ms * (self.stats.cycles_completed - 1) + duration_ms) / self.stats.cycles_completed;
        }

        info!("📊 Cycle Stats: Duration: {}ms, Candidates: {}, Validated: {}, Decisions: {}", 
              duration_ms, candidates, validated, decisions);
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Get active opportunities
    pub fn active_opportunities(&self) -> &HashMap<String, ValidatedOpportunity> {
        &self.active_opportunities
    }

    /// Check if pipeline components are available
    pub fn is_ready(&self) -> bool {
        self.soul_meteor_scanner.is_available() && self.crawl4ai_fetcher.is_available()
    }

    /// Get configuration
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }
}

impl Default for PipelineController {
    fn default() -> Self {
        Self::new()
    }
}

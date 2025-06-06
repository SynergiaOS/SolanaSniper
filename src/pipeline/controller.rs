/*!
🧠 Pipeline Controller - Autonomous Trading Intelligence Hub

This is the central brain that orchestrates the Hub-and-Spoke architecture:
1. Reads opportunities from DragonflyDB (produced by Soul Meteor Scanner)
2. Validates candidates with Crawl4AI sentiment analysis
3. Generates trading decisions and stores them back to DragonflyDB
4. Operates autonomously without direct scanner invocation
*/

use crate::db_connector::DbClient;
use crate::models::persistent_state::{DbKeys, RawOpportunity};
use crate::models::python_compat::PythonRawOpportunity;
use crate::data_fetcher::textual_data_fetcher::{TextualDataFetcher, TextualDataFetcherConfig};
use crate::pipeline::opportunity::{ValidatedOpportunity, SentimentReport};
use crate::pipeline::decision_engine::{DecisionEngine, TradingDecision, DecisionEngineConfig};
use crate::models::TradingResult;
use crate::config::AppConfig;
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use tokio::time::{Duration, sleep, timeout};

// Note: PipelineConfig removed - now using AppConfig for all configuration

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

/// Autonomous pipeline controller with DragonflyDB integration
pub struct PipelineController {
    /// Database client for Hub-and-Spoke architecture
    db_client: DbClient,
    /// Crawl4AI fetcher for sentiment validation
    crawl4ai_fetcher: TextualDataFetcher,
    /// Decision engine for trading decisions
    decision_engine: DecisionEngine,
    /// Pipeline execution statistics
    stats: PipelineStats,
    /// Configuration reference
    config: &'static AppConfig,
}

impl PipelineController {
    /// Create new pipeline controller with DragonflyDB client
    pub fn new(db_client: DbClient, config: &'static AppConfig) -> Self {
        Self {
            db_client,
            crawl4ai_fetcher: TextualDataFetcher::new(),
            decision_engine: DecisionEngine::new(),
            stats: PipelineStats::default(),
            config,
        }
    }

    /// Create new pipeline controller with custom configuration
    pub fn with_config(db_client: DbClient, config: &'static AppConfig, crawl4ai_config: TextualDataFetcherConfig, decision_config: DecisionEngineConfig) -> Self {
        Self {
            db_client,
            crawl4ai_fetcher: TextualDataFetcher::with_config(crawl4ai_config),
            decision_engine: DecisionEngine::with_config(decision_config),
            stats: PipelineStats::default(),
            config,
        }
    }

    /// Process opportunities from DragonflyDB - Main autonomous operation method
    pub async fn process_opportunities_from_db(&mut self) -> TradingResult<usize> {
        let cycle_start = std::time::Instant::now();

        info!("🧠 === STARTING AUTONOMOUS PROCESSING CYCLE #{} ===", self.stats.cycles_completed + 1);

        // STEP 1: Read raw opportunities from DragonflyDB
        let raw_opportunities = match self.read_raw_opportunities_from_db().await {
            Ok(opportunities) => opportunities,
            Err(e) => {
                error!("❌ Failed to read opportunities from database: {}", e);
                return Err(e);
            }
        };

        if raw_opportunities.is_empty() {
            info!("📊 No new opportunities found in database");
            self.update_cycle_stats(cycle_start, 0, 0, 0);
            return Ok(0);
        }

        info!("📊 Found {} raw opportunities to process", raw_opportunities.len());

        // STEP 2: Validate opportunities with Crawl4AI
        let validated_opportunities = self.validate_opportunities_from_db(raw_opportunities).await?;

        // STEP 3: Generate and store trading decisions
        let decisions_count = self.generate_and_store_decisions(&validated_opportunities).await?;

        // Update statistics
        let validated_count = validated_opportunities.len() as u64;
        self.update_cycle_stats(cycle_start, self.stats.total_candidates_found, validated_count, decisions_count as u64);

        info!("✅ === AUTONOMOUS CYCLE COMPLETED: {} decisions generated ===", decisions_count);

        Ok(decisions_count)
    }

    /// Legacy method for backward compatibility
    pub async fn run_cycle(&mut self) -> TradingResult<Vec<TradingDecision>> {
        warn!("🚨 run_cycle() is deprecated. Use process_opportunities_from_db() for autonomous operation.");

        // For now, redirect to the new method and return empty decisions
        let _processed = self.process_opportunities_from_db().await?;
        Ok(vec![])
    }

    /// Read raw opportunities from DragonflyDB
    async fn read_raw_opportunities_from_db(&mut self) -> TradingResult<Vec<PythonRawOpportunity>> {
        info!("📊 [STEP 1/3] Reading raw opportunities from DragonflyDB...");

        // Get list of raw opportunity keys (raw strings, not JSON)
        let opportunity_keys: Vec<String> = self.db_client
            .list_range_raw(DbKeys::ALL_RAW_OPPORTUNITIES, 0, -1)
            .await
            .map_err(|e| format!("Failed to get opportunity keys: {}", e))?;

        if opportunity_keys.is_empty() {
            return Ok(vec![]);
        }

        let mut raw_opportunities = Vec::new();
        let max_to_process = self.config.main_loop.max_opportunities_per_cycle.min(opportunity_keys.len());

        for (i, key) in opportunity_keys.iter().take(max_to_process).enumerate() {
            match self.db_client.get::<PythonRawOpportunity>(key).await {
                Ok(Some(python_opp)) => {
                    // Check if opportunity is still valid (not expired)
                    if python_opp.is_valid() {
                        // Check if already processed
                        let is_processed = self.db_client
                            .set_contains(DbKeys::PROCESSED_TOKENS, &python_opp.candidate.address)
                            .await
                            .unwrap_or(false);

                        if !is_processed {
                            info!("  {}. {} (Age: {}min, Score: {:.2})",
                                  i + 1, python_opp.summary(),
                                  python_opp.age_minutes().unwrap_or(0),
                                  python_opp.candidate.opportunity_score);
                            raw_opportunities.push(python_opp);
                        } else {
                            debug!("  Skipping already processed: {}", python_opp.candidate.address);
                        }
                    } else {
                        debug!("  Skipping expired opportunity: {}", python_opp.candidate.address);
                    }
                }
                Ok(None) => {
                    warn!("  Key {} not found in database", key);
                }
                Err(e) => {
                    warn!("  Failed to read opportunity {}: {}", key, e);
                }
            }
        }

        self.stats.total_candidates_found += raw_opportunities.len() as u64;

        info!("✅ Found {} valid, unprocessed opportunities", raw_opportunities.len());

        Ok(raw_opportunities)
    }

    /// Validate opportunities from database using Crawl4AI sentiment analysis
    async fn validate_opportunities_from_db(
        &mut self,
        raw_opportunities: Vec<PythonRawOpportunity>,
    ) -> TradingResult<Vec<ValidatedOpportunity>> {
        let total_opportunities = raw_opportunities.len();
        info!("🔬 [STEP 2/3] Validating {} opportunities with Crawl4AI...", total_opportunities);

        let mut validated_opportunities = Vec::new();

        for (i, python_opp) in raw_opportunities.into_iter().enumerate() {
            info!("  Validating {}/{}: {}", i + 1, total_opportunities, python_opp.candidate.name);

            // Convert to Rust format for validation
            match python_opp.to_rust_format() {
                Ok(rust_opp) => {
                    match self.validate_single_opportunity(&rust_opp).await {
                        Ok(Some(opportunity)) => {
                            info!("    ✅ VALIDATED: Score {:.2}, Sentiment {:.2}",
                                  opportunity.combined_score, opportunity.sentiment.aggregated_score);

                            // Mark as processed in database
                            if let Err(e) = self.db_client
                                .set_add(DbKeys::PROCESSED_TOKENS, &python_opp.candidate.address)
                                .await {
                                warn!("Failed to mark {} as processed: {}", python_opp.candidate.address, e);
                            }

                            // Store validated opportunity in database
                            let validated_key = format!("validated_opportunity:{}", python_opp.candidate.address);
                            if let Err(e) = self.db_client.set(&validated_key, &opportunity, Some(3600)).await {
                                warn!("Failed to store validated opportunity: {}", e);
                            }

                            validated_opportunities.push(opportunity);
                        }
                        Ok(None) => {
                            info!("    ⛔ REJECTED: Failed sentiment validation");

                            // Mark as processed even if rejected to avoid reprocessing
                            if let Err(e) = self.db_client
                                .set_add(DbKeys::PROCESSED_TOKENS, &python_opp.candidate.address)
                                .await {
                                warn!("Failed to mark {} as processed: {}", python_opp.candidate.address, e);
                            }
                        }
                        Err(e) => {
                            warn!("    ⚠️ ERROR: Failed to validate {}: {}", python_opp.candidate.name, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("    ⚠️ ERROR: Failed to convert to Rust format: {}", e);
                }
            }

            // Small delay to avoid overwhelming the Crawl4AI service
            sleep(Duration::from_millis(500)).await;
        }

        self.stats.total_candidates_validated += validated_opportunities.len() as u64;

        info!("✅ Validated {}/{} opportunities successfully",
              validated_opportunities.len(), total_opportunities);

        Ok(validated_opportunities)
    }

    /// Validate a single opportunity with Crawl4AI
    async fn validate_single_opportunity(
        &self,
        raw_opportunity: &RawOpportunity,
    ) -> TradingResult<Option<ValidatedOpportunity>> {
        // Create TokenInfo for Crawl4AI
        let token_info = crate::models::TokenInfo {
            symbol: raw_opportunity.candidate.name.clone(),
            address: raw_opportunity.candidate.address.clone(),
            name: Some(raw_opportunity.candidate.name.clone()),
            price: raw_opportunity.candidate.current_price,
            market_cap: Some(raw_opportunity.candidate.liquidity_usd),
            volume_24h: Some(raw_opportunity.candidate.volume_24h),
            liquidity: Some(raw_opportunity.candidate.liquidity_usd),
            age_hours: None, // Would need to calculate from creation time
        };

        // Fetch textual data with timeout
        let crawl4ai_timeout = Duration::from_secs(self.config.main_loop.cycle_timeout_seconds / 4);

        match timeout(crawl4ai_timeout, self.crawl4ai_fetcher.fetch_textual_data(&token_info)).await {
            Ok(Ok(textual_data)) => {
                // Convert to sentiment report
                let sentiment = self.convert_to_sentiment_report(&textual_data);

                // Use configuration thresholds
                let min_sentiment_score = 0.1; // Could be moved to config
                let min_sentiment_confidence = 0.5; // Could be moved to config

                // Check if sentiment meets minimum criteria
                if sentiment.aggregated_score >= min_sentiment_score &&
                   sentiment.confidence >= min_sentiment_confidence {

                    let opportunity = ValidatedOpportunity::new(raw_opportunity.candidate.clone(), sentiment);
                    Ok(Some(opportunity))
                } else {
                    debug!("Sentiment below threshold: score={:.2}, confidence={:.2}",
                           sentiment.aggregated_score, sentiment.confidence);
                    Ok(None)
                }
            }
            Ok(Err(e)) => {
                debug!("Crawl4AI validation failed: {}", e);
                Ok(None) // Don't fail the entire cycle for one candidate
            }
            Err(_) => {
                warn!("Crawl4AI validation timed out for {}", raw_opportunity.candidate.name);
                Ok(None)
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

    /// Generate and store trading decisions for validated opportunities
    async fn generate_and_store_decisions(&mut self, opportunities: &[ValidatedOpportunity]) -> TradingResult<usize> {
        info!("🤖 [STEP 3/3] Generating trading decisions for {} opportunities...", opportunities.len());

        let mut decisions_count = 0;

        for opportunity in opportunities {
            match self.decision_engine.analyze_opportunity(opportunity).await {
                Ok(decision) => {
                    info!("  ✅ Decision for {}: {:?}",
                          opportunity.candidate.name, decision.decision_type);

                    // Store decision in DragonflyDB for TradingExecutor to consume
                    let decision_key = format!("trading_decision:{}", opportunity.candidate.address);
                    if let Err(e) = self.db_client.set(&decision_key, &decision, Some(1800)).await { // 30 min TTL
                        warn!("Failed to store trading decision: {}", e);
                    } else {
                        // Add to decisions queue
                        if let Err(e) = self.db_client
                            .list_push(DbKeys::TRADING_DECISIONS_QUEUE, &decision_key)
                            .await {
                            warn!("Failed to add decision to queue: {}", e);
                        } else {
                            decisions_count += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("  ⚠️ Failed to generate decision for {}: {}",
                          opportunity.candidate.name, e);
                }
            }
        }

        self.stats.total_decisions_made += decisions_count as u64;

        info!("✅ Generated and stored {} trading decisions", decisions_count);

        Ok(decisions_count)
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

    /// Get database client reference
    pub fn db_client(&self) -> &DbClient {
        &self.db_client
    }

    /// Check if pipeline components are available
    pub async fn is_ready(&self) -> bool {
        // Check database connection
        let db_health = self.db_client.health_check().await.unwrap_or(false);

        // Check Crawl4AI availability
        let crawl4ai_available = self.crawl4ai_fetcher.is_available();

        db_health && crawl4ai_available
    }

    /// Get configuration
    pub fn config(&self) -> &AppConfig {
        self.config
    }

    /// Perform health check on all components
    pub async fn health_check(&self) -> TradingResult<HashMap<String, bool>> {
        let mut health_status = HashMap::new();

        // Database health
        health_status.insert("database".to_string(),
                           self.db_client.health_check().await.unwrap_or(false));

        // Crawl4AI health
        health_status.insert("crawl4ai".to_string(),
                           self.crawl4ai_fetcher.is_available());

        // Decision engine health
        health_status.insert("decision_engine".to_string(), true); // Always available

        Ok(health_status)
    }
}

// Note: Default implementation removed as PipelineController now requires DbClient
// Use PipelineController::new(db_client) instead

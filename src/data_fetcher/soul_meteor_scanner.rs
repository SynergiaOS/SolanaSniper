/*!
🕸️ Soul Meteor Scanner - Rust Integration
Real-time DLMM pool scanner using Meteora API via Python service

This module integrates with the Soul Meteor Scanner Python script to provide
real-time hot liquidity pool opportunities for trading intelligence.
*/

use crate::models::{TradingError, TradingResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Hot candidate opportunity from Soul Meteor Scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotCandidate {
    pub name: String,
    pub address: String,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub fees_24h: f64,
    pub fee_tvl_ratio_24h: f64,
    pub apr: f64,
    pub apy: f64,
    pub opportunity_score: f64,
    pub mint_x: String,
    pub mint_y: String,
    pub current_price: f64,
    #[serde(default)]
    pub is_blacklisted: bool,
    #[serde(default)]
    pub hide: bool,
}

/// Errors specific to Soul Meteor scanning
#[derive(Debug, thiserror::Error)]
pub enum SoulMeteorError {
    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(#[from] std::io::Error),

    #[error("Failed to read from stdout: {0}")]
    ReadFromStdoutFailed(std::io::Error),

    #[error("JSON deserialization failed: {0}")]
    JsonDeserializationFailed(#[from] serde_json::Error),

    #[error("Command failed with status: {0}")]
    CommandFailed(String),

    #[error("Python script not found at path: {0}")]
    ScriptNotFound(String),

    #[error("No opportunities found")]
    NoOpportunities,
}

impl From<SoulMeteorError> for TradingError {
    fn from(err: SoulMeteorError) -> Self {
        TradingError::DataError(err.to_string())
    }
}

/// Configuration for the Soul Meteor Scanner
#[derive(Debug, Clone)]
pub struct SoulMeteorScannerConfig {
    /// Path to Python executable
    pub python_executable: String,
    /// Path to the Soul Meteor Scanner script
    pub script_path: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of opportunities to return
    pub max_opportunities: usize,
}

impl Default for SoulMeteorScannerConfig {
    fn default() -> Self {
        Self {
            python_executable: "python3".to_string(),
            script_path: "./pyinstaller_scripts/soul_meteor_scanner.py".to_string(),
            timeout_seconds: 60,
            max_opportunities: 20,
        }
    }
}

/// Soul Meteor Scanner - integrates with Python service
pub struct SoulMeteorScanner {
    config: SoulMeteorScannerConfig,
}

impl SoulMeteorScanner {
    /// Create a new SoulMeteorScanner with default configuration
    pub fn new() -> Self {
        Self {
            config: SoulMeteorScannerConfig::default(),
        }
    }

    /// Create a new SoulMeteorScanner with custom configuration
    pub fn with_config(config: SoulMeteorScannerConfig) -> Self {
        Self { config }
    }

    /// Scan for hot trading opportunities
    pub async fn scan_for_opportunities(&self) -> TradingResult<Vec<HotCandidate>> {
        info!("🕸️ Starting Soul Meteor scan for hot opportunities...");

        // Validate script exists
        if !Path::new(&self.config.script_path).exists() {
            return Err(SoulMeteorError::ScriptNotFound(self.config.script_path.clone()).into());
        }

        // Execute the Soul Meteor Scanner
        let candidates = self.execute_scanner().await?;

        if candidates.is_empty() {
            warn!("⚠️ No hot opportunities found in current scan");
            return Err(SoulMeteorError::NoOpportunities.into());
        }

        // Limit results
        let limited_candidates = candidates
            .into_iter()
            .take(self.config.max_opportunities)
            .collect::<Vec<_>>();

        info!(
            "✅ Found {} hot opportunities (limited to {})",
            limited_candidates.len(),
            self.config.max_opportunities
        );

        // Log top opportunities
        for (i, candidate) in limited_candidates.iter().take(5).enumerate() {
            info!(
                "🔥 #{} - {} | Score: {:.2} | Liquidity: ${:.0} | Volume: ${:.0} | Fee/TVL: {:.1}%",
                i + 1,
                candidate.name,
                candidate.opportunity_score,
                candidate.liquidity_usd,
                candidate.volume_24h,
                candidate.fee_tvl_ratio_24h
            );
        }

        Ok(limited_candidates)
    }

    /// Execute the Soul Meteor Scanner Python script
    async fn execute_scanner(&self) -> Result<Vec<HotCandidate>, SoulMeteorError> {
        debug!("🚀 Spawning Soul Meteor Scanner: {} {}", 
               self.config.python_executable, self.config.script_path);

        // Spawn the process
        let mut cmd = Command::new(&self.config.python_executable)
            .arg(&self.config.script_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(SoulMeteorError::ProcessSpawnFailed)?;

        // Get handles to stdout and stderr
        let mut stdout = cmd.stdout.take().unwrap();
        let mut stderr = cmd.stderr.take().unwrap();

        // Read outputs concurrently
        let (stdout_result, stderr_result) = tokio::join!(
            async {
                let mut buffer = String::new();
                stdout
                    .read_to_string(&mut buffer)
                    .await
                    .map_err(SoulMeteorError::ReadFromStdoutFailed)?;
                Ok::<String, SoulMeteorError>(buffer)
            },
            async {
                let mut buffer = String::new();
                stderr
                    .read_to_string(&mut buffer)
                    .await
                    .map_err(SoulMeteorError::ReadFromStdoutFailed)?;
                Ok::<String, SoulMeteorError>(buffer)
            }
        );

        let stdout_content = stdout_result?;
        let stderr_content = stderr_result?;

        // Log stderr output (Python service logs)
        if !stderr_content.is_empty() {
            debug!("📋 Soul Meteor Scanner Logs:\n{}", stderr_content);
        }

        // Wait for process to complete
        let status = cmd.wait().await.map_err(SoulMeteorError::ProcessSpawnFailed)?;

        if !status.success() {
            error!("❌ Soul Meteor Scanner failed with status: {}", status);
            error!("📋 Scanner stderr: {}", stderr_content);
            return Err(SoulMeteorError::CommandFailed(format!(
                "Process exited with status: {} | stderr: {}",
                status, stderr_content
            )));
        }

        // Parse JSON output
        if stdout_content.trim().is_empty() {
            warn!("⚠️ Soul Meteor Scanner returned empty output");
            return Ok(vec![]);
        }

        debug!("📥 Parsing JSON output from Soul Meteor Scanner");
        let candidates: Vec<HotCandidate> = serde_json::from_str(&stdout_content)
            .map_err(SoulMeteorError::JsonDeserializationFailed)?;

        debug!("✅ Successfully parsed {} candidates from Soul Meteor Scanner", candidates.len());
        Ok(candidates)
    }

    /// Get configuration
    pub fn config(&self) -> &SoulMeteorScannerConfig {
        &self.config
    }

    /// Update script path
    pub fn set_script_path(&mut self, path: String) {
        self.config.script_path = path;
    }

    /// Check if the Soul Meteor Scanner script is available
    pub fn is_available(&self) -> bool {
        Path::new(&self.config.script_path).exists()
    }

    /// Get top opportunities by score
    pub fn get_top_opportunities(candidates: &[HotCandidate], limit: usize) -> Vec<&HotCandidate> {
        let mut sorted_candidates = candidates.iter().collect::<Vec<_>>();
        sorted_candidates.sort_by(|a, b| b.opportunity_score.partial_cmp(&a.opportunity_score).unwrap());
        sorted_candidates.into_iter().take(limit).collect()
    }

    /// Filter candidates by minimum criteria
    pub fn filter_candidates(
        candidates: &[HotCandidate],
        min_liquidity: f64,
        min_score: f64,
    ) -> Vec<&HotCandidate> {
        candidates
            .iter()
            .filter(|c| c.liquidity_usd >= min_liquidity && c.opportunity_score >= min_score)
            .collect()
    }
}

impl Default for SoulMeteorScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SoulMeteorScannerConfig::default();
        assert_eq!(config.python_executable, "python3");
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_opportunities, 20);
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = SoulMeteorScanner::new();
        assert_eq!(scanner.config.python_executable, "python3");
    }

    #[test]
    fn test_filter_candidates() {
        let candidates = vec![
            HotCandidate {
                name: "Test1".to_string(),
                address: "addr1".to_string(),
                liquidity_usd: 50000.0,
                volume_24h: 100000.0,
                fees_24h: 1000.0,
                fee_tvl_ratio_24h: 2.0,
                apr: 2.0,
                apy: 100.0,
                opportunity_score: 3.5,
                mint_x: "mint1".to_string(),
                mint_y: "mint2".to_string(),
                current_price: 1.0,
                is_blacklisted: false,
                hide: false,
            },
            HotCandidate {
                name: "Test2".to_string(),
                address: "addr2".to_string(),
                liquidity_usd: 10000.0,
                volume_24h: 20000.0,
                fees_24h: 200.0,
                fee_tvl_ratio_24h: 2.0,
                apr: 2.0,
                apy: 100.0,
                opportunity_score: 2.0,
                mint_x: "mint1".to_string(),
                mint_y: "mint2".to_string(),
                current_price: 1.0,
                is_blacklisted: false,
                hide: false,
            },
        ];

        let filtered = SoulMeteorScanner::filter_candidates(&candidates, 30000.0, 3.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Test1");
    }
}

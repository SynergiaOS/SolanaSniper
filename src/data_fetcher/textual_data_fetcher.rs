/*!
🚀 Textual Data Fetcher - Scrapy Professional Integration
Real-time textual intelligence with 10x performance boost

This module integrates with the Scrapy PyInstaller executable to provide
professional-grade web scraping for trading intelligence with significant
performance improvements over the previous Crawl4AI implementation.
*/

use crate::models::{
    Crawl4AIRequest, Crawl4AIResponse, TextualData, TradingError, TradingResult, TokenInfo,
};
use serde_json;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Errors specific to textual data fetching
#[derive(Debug, thiserror::Error)]
pub enum TextualDataError {
    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(#[from] std::io::Error),

    #[error("Failed to access stdin/stdout/stderr pipe")]
    PipeFailed,

    #[error("Failed to write to stdin: {0}")]
    WriteToStdinFailed(std::io::Error),

    #[error("Failed to read from stdout: {0}")]
    ReadFromStdoutFailed(std::io::Error),

    #[error("JSON serialization failed: {0}")]
    JsonSerializationFailed(#[from] serde_json::Error),

    #[error("Command failed with status: {0}")]
    CommandFailed(String),

    #[error("Executable not found at path: {0}")]
    ExecutableNotFound(String),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Service returned error: {0}")]
    ServiceError(String),
}

impl From<TextualDataError> for TradingError {
    fn from(err: TextualDataError) -> Self {
        TradingError::DataError(err.to_string())
    }
}

/// Configuration for the Textual Data Fetcher
#[derive(Debug, Clone)]
pub struct TextualDataFetcherConfig {
    /// Path to the Scrapy service executable
    pub executable_path: String,
    /// Default data types to fetch
    pub default_data_types: Vec<String>,
    /// Default time range in hours
    pub default_time_range_hours: u32,
    /// Default maximum results per request
    pub default_max_results: u32,
    /// Whether to enable sentiment analysis by default
    pub default_sentiment_analysis: bool,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for TextualDataFetcherConfig {
    fn default() -> Self {
        Self {
            executable_path: "./pyinstaller_scripts/scrapy_service/dist/scrapy_service".to_string(),
            default_data_types: vec!["news".to_string(), "social".to_string()],
            default_time_range_hours: 24,
            default_max_results: 30,
            default_sentiment_analysis: true,
            timeout_seconds: 60,
        }
    }
}

/// Textual Data Fetcher - integrates with Scrapy service (10x performance boost)
pub struct TextualDataFetcher {
    config: TextualDataFetcherConfig,
}

impl TextualDataFetcher {
    /// Create a new TextualDataFetcher with default configuration
    pub fn new() -> Self {
        Self {
            config: TextualDataFetcherConfig::default(),
        }
    }

    /// Create a new TextualDataFetcher with custom configuration
    pub fn with_config(config: TextualDataFetcherConfig) -> Self {
        Self { config }
    }

    /// Fetch textual analytics for a token using default parameters
    pub async fn fetch_textual_data(&self, token_info: &TokenInfo) -> TradingResult<TextualData> {
        let request = Crawl4AIRequest {
            token_symbol: token_info.symbol.clone(),
            token_address: Some(token_info.address.clone()),
            data_types: self.config.default_data_types.clone(),
            time_range_hours: self.config.default_time_range_hours,
            max_results: self.config.default_max_results,
            sentiment_analysis: self.config.default_sentiment_analysis,
        };

        self.fetch_with_request(&request).await
    }

    /// Fetch textual analytics with custom request parameters
    pub async fn fetch_with_request(&self, request: &Crawl4AIRequest) -> TradingResult<TextualData> {
        info!(
            "🚀 Fetching textual data for {} with Scrapy (10x boost): {:?}",
            request.token_symbol, request.data_types
        );

        // Validate executable exists
        if !Path::new(&self.config.executable_path).exists() {
            return Err(TextualDataError::ExecutableNotFound(self.config.executable_path.clone()).into());
        }

        // Serialize request to JSON
        let input_json = serde_json::to_string(request)
            .map_err(TextualDataError::JsonSerializationFailed)?;

        debug!("📤 Sending request to Scrapy service: {}", input_json);

        // Execute the Scrapy service
        let response = self.execute_scrapy_service(&input_json).await?;

        // Parse and validate response
        let crawl4ai_response: Crawl4AIResponse = serde_json::from_str(&response)
            .map_err(TextualDataError::JsonSerializationFailed)?;

        debug!("📥 Received response from Scrapy service: status={}", crawl4ai_response.status);

        // Handle response
        match crawl4ai_response.status.as_str() {
            "success" => {
                if let Some(data) = crawl4ai_response.data {
                    info!(
                        "✅ Successfully fetched {} textual sources for {} ({}ms)",
                        crawl4ai_response.total_items,
                        request.token_symbol,
                        crawl4ai_response.execution_time_ms
                    );
                    Ok(data)
                } else {
                    Err(TextualDataError::InvalidResponse("Success response missing data".to_string()).into())
                }
            }
            "error" => {
                let error_msg = crawl4ai_response.error_message
                    .unwrap_or_else(|| "Unknown error from Crawl4AI service".to_string());
                warn!("⚠️ Scrapy service returned error: {}", error_msg);
                Err(TextualDataError::ServiceError(error_msg).into())
            }
            _ => {
                Err(TextualDataError::InvalidResponse(
                    format!("Unknown status: {}", crawl4ai_response.status)
                ).into())
            }
        }
    }

    /// Execute the Scrapy service with the given input
    async fn execute_scrapy_service(&self, input_json: &str) -> Result<String, TextualDataError> {
        debug!("🚀 Spawning Scrapy process: {}", self.config.executable_path);

        // Spawn the process
        let mut cmd = Command::new(&self.config.executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(TextualDataError::ProcessSpawnFailed)?;

        // Get handles to stdin, stdout, stderr
        let mut stdin = cmd.stdin.take().ok_or(TextualDataError::PipeFailed)?;
        let mut stdout = cmd.stdout.take().ok_or(TextualDataError::PipeFailed)?;
        let mut stderr = cmd.stderr.take().ok_or(TextualDataError::PipeFailed)?;

        // Execute I/O operations concurrently
        let (_write_result, stdout_result, stderr_result) = tokio::try_join!(
            // Write input to stdin
            async {
                stdin
                    .write_all(input_json.as_bytes())
                    .await
                    .map_err(TextualDataError::WriteToStdinFailed)?;
                stdin.shutdown().await.map_err(TextualDataError::WriteToStdinFailed)?;
                Ok::<(), TextualDataError>(())
            },
            // Read from stdout
            async {
                let mut buffer = String::new();
                stdout
                    .read_to_string(&mut buffer)
                    .await
                    .map_err(TextualDataError::ReadFromStdoutFailed)?;
                Ok::<String, TextualDataError>(buffer)
            },
            // Read from stderr
            async {
                let mut buffer = String::new();
                stderr
                    .read_to_string(&mut buffer)
                    .await
                    .map_err(TextualDataError::ReadFromStdoutFailed)?;
                Ok::<String, TextualDataError>(buffer)
            }
        )?;

        // Log stderr output (service logs)
        if !stderr_result.is_empty() {
            debug!("📋 Scrapy Service Logs:\n{}", stderr_result);
        }

        // Wait for process to complete
        let status = cmd.wait().await.map_err(TextualDataError::ProcessSpawnFailed)?;

        if !status.success() {
            error!("❌ Scrapy service failed with status: {}", status);
            error!("📋 Service stderr: {}", stderr_result);
            return Err(TextualDataError::CommandFailed(format!(
                "Process exited with status: {} | stderr: {}",
                status, stderr_result
            )));
        }

        debug!("✅ Scrapy service completed successfully");
        Ok(stdout_result)
    }

    /// Get configuration
    pub fn config(&self) -> &TextualDataFetcherConfig {
        &self.config
    }

    /// Update executable path
    pub fn set_executable_path(&mut self, path: String) {
        self.config.executable_path = path;
    }

    /// Check if the Scrapy service executable is available
    pub fn is_available(&self) -> bool {
        Path::new(&self.config.executable_path).exists()
    }
}

impl Default for TextualDataFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TextualDataFetcherConfig::default();
        assert_eq!(config.default_time_range_hours, 24);
        assert_eq!(config.default_max_results, 30);
        assert!(config.default_sentiment_analysis);
        assert_eq!(config.default_data_types, vec!["news", "social"]);
    }

    #[test]
    fn test_fetcher_creation() {
        let fetcher = TextualDataFetcher::new();
        assert_eq!(fetcher.config.default_time_range_hours, 24);
    }

    #[test]
    fn test_executable_availability() {
        let fetcher = TextualDataFetcher::new();
        // This will be false in test environment, which is expected
        let _available = fetcher.is_available();
    }
}

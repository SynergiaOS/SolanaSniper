use crate::models::{AnalyticsResult, TradingError, TradingResult};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, error, info, warn};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct AnalyticsAggregator {
    python_executables_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonExecutableInput {
    pub symbol: String,
    pub data: serde_json::Value,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonExecutableOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub confidence: f64,
}

impl AnalyticsAggregator {
    pub fn new(python_executables_path: String) -> Self {
        Self {
            python_executables_path,
        }
    }

    pub async fn run_talib_analysis(&self, symbol: &str, price_data: &[f64]) -> TradingResult<AnalyticsResult> {
        let input = PythonExecutableInput {
            symbol: symbol.to_string(),
            data: serde_json::json!({
                "prices": price_data,
                "indicators": ["sma", "rsi", "macd", "bollinger_bands"]
            }),
            parameters: serde_json::json!({
                "sma_period": 20,
                "rsi_period": 14,
                "macd_fast": 12,
                "macd_slow": 26,
                "macd_signal": 9,
                "bb_period": 20,
                "bb_std": 2.0
            }),
        };

        let output = self.execute_python_binary("talib_minimal", &input).await?;
        
        Ok(AnalyticsResult {
            source: "talib_minimal".to_string(),
            symbol: symbol.to_string(),
            result_type: "technical_analysis".to_string(),
            data: output.result,
            confidence: output.confidence,
            timestamp: Utc::now(),
        })
    }

    pub async fn run_social_scanner(&self, symbol: &str, keywords: &[String]) -> TradingResult<AnalyticsResult> {
        let input = PythonExecutableInput {
            symbol: symbol.to_string(),
            data: serde_json::json!({
                "keywords": keywords,
                "sources": ["twitter", "reddit", "discord"],
                "time_range": "1h"
            }),
            parameters: serde_json::json!({
                "max_posts": 100,
                "min_engagement": 10,
                "language": "en"
            }),
        };

        let output = self.execute_python_binary("social_scanner", &input).await?;
        
        Ok(AnalyticsResult {
            source: "social_scanner".to_string(),
            symbol: symbol.to_string(),
            result_type: "social_sentiment".to_string(),
            data: output.result,
            confidence: output.confidence,
            timestamp: Utc::now(),
        })
    }

    pub async fn run_sentiment_analyzer(&self, symbol: &str, text_data: &[String]) -> TradingResult<AnalyticsResult> {
        let input = PythonExecutableInput {
            symbol: symbol.to_string(),
            data: serde_json::json!({
                "texts": text_data,
                "analysis_types": ["sentiment", "emotion", "topics"]
            }),
            parameters: serde_json::json!({
                "model": "finbert",
                "batch_size": 32,
                "confidence_threshold": 0.7
            }),
        };

        let output = self.execute_python_binary("sentiment_analyzer", &input).await?;
        
        Ok(AnalyticsResult {
            source: "sentiment_analyzer".to_string(),
            symbol: symbol.to_string(),
            result_type: "sentiment_analysis".to_string(),
            data: output.result,
            confidence: output.confidence,
            timestamp: Utc::now(),
        })
    }

    async fn execute_python_binary(&self, binary_name: &str, input: &PythonExecutableInput) -> TradingResult<PythonExecutableOutput> {
        let binary_path = format!("{}/{}", self.python_executables_path, binary_name);
        
        // Check if binary exists
        if !std::path::Path::new(&binary_path).exists() {
            return Err(TradingError::DataError(format!(
                "Python executable not found: {}", binary_path
            )));
        }

        let input_json = serde_json::to_string(input)
            .map_err(|e| TradingError::DataError(format!("Failed to serialize input: {}", e)))?;

        debug!("Executing {} with input: {}", binary_name, input_json);

        // Execute the binary with JSON input via stdin
        let mut command = Command::new(&binary_path);
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut child = command.spawn()
            .map_err(|e| TradingError::DataError(format!("Failed to spawn process: {}", e)))?;

        // Write input to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(input_json.as_bytes())
                .map_err(|e| TradingError::DataError(format!("Failed to write to stdin: {}", e)))?;
        }

        // Wait for the process to complete
        let output = child.wait_with_output()
            .map_err(|e| TradingError::DataError(format!("Failed to wait for process: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Python executable {} failed: {}", binary_name, stderr);
            return Err(TradingError::DataError(format!(
                "Python executable failed: {}", stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Python executable {} output: {}", binary_name, stdout);

        let result: PythonExecutableOutput = serde_json::from_str(&stdout)
            .map_err(|e| TradingError::DataError(format!("Failed to parse output: {}", e)))?;

        if !result.success {
            return Err(TradingError::DataError(format!(
                "Python executable returned error: {}", 
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        info!("Successfully executed {} for symbol {}", binary_name, input.symbol);
        Ok(result)
    }

    pub fn check_executables(&self) -> Vec<String> {
        let executables = ["talib_minimal", "social_scanner", "sentiment_analyzer"];
        let mut missing = Vec::new();

        for executable in &executables {
            let path = format!("{}/{}", self.python_executables_path, executable);
            if !std::path::Path::new(&path).exists() {
                missing.push(executable.to_string());
            }
        }

        if missing.is_empty() {
            info!("All Python executables found");
        } else {
            warn!("Missing Python executables: {:?}", missing);
        }

        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_aggregator_creation() {
        let aggregator = AnalyticsAggregator::new("python_executables".to_string());
        assert_eq!(aggregator.python_executables_path, "python_executables");
    }

    #[test]
    fn test_check_executables() {
        let aggregator = AnalyticsAggregator::new("nonexistent_path".to_string());
        let missing = aggregator.check_executables();
        
        // Should report all as missing since the path doesn't exist
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&"talib_minimal".to_string()));
        assert!(missing.contains(&"social_scanner".to_string()));
        assert!(missing.contains(&"sentiment_analyzer".to_string()));
    }

    #[tokio::test]
    async fn test_python_executable_input_serialization() {
        let input = PythonExecutableInput {
            symbol: "BTCUSDT".to_string(),
            data: serde_json::json!({"test": "data"}),
            parameters: serde_json::json!({"param": "value"}),
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: PythonExecutableInput = serde_json::from_str(&json).unwrap();
        
        assert_eq!(input.symbol, deserialized.symbol);
    }
}

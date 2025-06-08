use crate::models::{StrategySignal, SignalType, TradingResult, MarketEvent};
use crate::strategy::{EnhancedStrategy, StrategyContext, StrategyType};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpikeTrigger {
    pub symbol: String,
    pub current_volume: f64,
    pub average_volume: f64,
    pub spike_ratio: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeData {
    pub volume: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct VolumeSpikeStrategy {
    pub name: String,
    pub enabled: bool,
    
    // Configuration
    pub spike_threshold: f64,        // Minimum spike ratio (e.g., 3.0 = 300% of average)
    pub volume_window: i64,          // Minutes to calculate average volume
    pub min_volume: f64,             // Minimum volume to consider
    pub cooldown_period: i64,        // Minutes between signals for same token
    
    // State
    volume_history: HashMap<String, Vec<VolumeData>>,
    last_signals: HashMap<String, DateTime<Utc>>,
}

impl VolumeSpikeStrategy {
    pub fn new() -> Self {
        Self {
            name: "Volume Spike".to_string(),
            enabled: true,
            spike_threshold: 3.0,
            volume_window: 30,
            min_volume: 1000.0,
            cooldown_period: 15,
            volume_history: HashMap::new(),
            last_signals: HashMap::new(),
        }
    }

    pub fn with_config(
        spike_threshold: f64,
        volume_window: i64,
        min_volume: f64,
        cooldown_period: i64,
    ) -> Self {
        Self {
            name: "Volume Spike".to_string(),
            enabled: true,
            spike_threshold,
            volume_window,
            min_volume,
            cooldown_period,
            volume_history: HashMap::new(),
            last_signals: HashMap::new(),
        }
    }

    fn update_volume_history(&mut self, symbol: &str, volume: f64, price: f64, timestamp: DateTime<Utc>) {
        let history = self.volume_history.entry(symbol.to_string()).or_insert_with(Vec::new);
        
        // Add new data point
        history.push(VolumeData {
            volume,
            price,
            timestamp,
        });

        // Remove old data points (older than volume_window)
        let cutoff_time = timestamp - Duration::minutes(self.volume_window);
        history.retain(|data| data.timestamp > cutoff_time);

        // Keep only last 100 data points to prevent memory bloat
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
    }

    fn calculate_average_volume(&self, symbol: &str) -> Option<f64> {
        let history = self.volume_history.get(symbol)?;
        
        if history.len() < 3 {
            return None; // Need at least 3 data points
        }

        let total_volume: f64 = history.iter().map(|data| data.volume).sum();
        Some(total_volume / history.len() as f64)
    }

    fn is_in_cooldown(&self, symbol: &str, current_time: DateTime<Utc>) -> bool {
        if let Some(last_signal_time) = self.last_signals.get(symbol) {
            let cooldown_end = *last_signal_time + Duration::minutes(self.cooldown_period);
            current_time < cooldown_end
        } else {
            false
        }
    }

    fn detect_volume_spike(&mut self, symbol: &str, current_volume: f64, price: f64, timestamp: DateTime<Utc>) -> Option<VolumeSpikeTrigger> {
        // Check if we have enough historical data
        let average_volume = self.calculate_average_volume(symbol)?;
        
        // Check minimum volume threshold
        if current_volume < self.min_volume {
            return None;
        }

        // Calculate spike ratio
        let spike_ratio = current_volume / average_volume;
        
        // Check if it's a significant spike
        if spike_ratio < self.spike_threshold {
            return None;
        }

        // Check cooldown period
        if self.is_in_cooldown(symbol, timestamp) {
            debug!("Volume spike detected for {} but in cooldown period", symbol);
            return None;
        }

        info!(
            "🔥 Volume spike detected: {} - Current: {:.2}, Average: {:.2}, Ratio: {:.2}x",
            symbol, current_volume, average_volume, spike_ratio
        );

        Some(VolumeSpikeTrigger {
            symbol: symbol.to_string(),
            current_volume,
            average_volume,
            spike_ratio,
            price,
            timestamp,
        })
    }

    fn generate_signal(&mut self, trigger: VolumeSpikeTrigger) -> StrategySignal {
        // Update last signal time
        self.last_signals.insert(trigger.symbol.clone(), trigger.timestamp);

        // Determine signal strength based on spike ratio
        let strength = ((trigger.spike_ratio - self.spike_threshold) / self.spike_threshold)
            .min(1.0)
            .max(0.1);

        // Volume spikes usually indicate buying pressure
        let signal_type = SignalType::Buy;

        StrategySignal {
            strategy: self.name.clone(),
            signal_type,
            symbol: trigger.symbol.clone(),
            strength,
            price: trigger.price,
            size: 0.05, // 🚀 LIVE TRADING: Small amounts for testing (0.05 SOL)
            timestamp: trigger.timestamp,
            metadata: serde_json::json!({
                "spike_ratio": trigger.spike_ratio,
                "current_volume": trigger.current_volume,
                "average_volume": trigger.average_volume,
                "volume_window_minutes": self.volume_window,
                "reason": format!(
                    "Volume spike: {:.1}x average volume ({:.0} vs {:.0})",
                    trigger.spike_ratio,
                    trigger.current_volume,
                    trigger.average_volume
                )
            }),
        }
    }
}

#[async_trait]
impl EnhancedStrategy for VolumeSpikeStrategy {
    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }

        let market_data = &context.aggregated_data.primary_data;

        // Create a mutable copy to update volume history
        let mut strategy = self.clone();

        // Update volume history with current market data
        strategy.update_volume_history(
            &market_data.symbol,
            market_data.volume,
            market_data.price,
            market_data.timestamp,
        );

        // Check for volume spike
        if let Some(trigger) = strategy.detect_volume_spike(
            &market_data.symbol,
            market_data.volume,
            market_data.price,
            market_data.timestamp,
        ) {
            let signal = strategy.generate_signal(trigger);
            debug!("Generated volume spike signal: {:?}", signal);
            return Ok(Some(signal));
        }

        Ok(None)
    }

    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()> {
        if let Some(spike_threshold) = parameters.get("spike_threshold") {
            if let Some(value) = spike_threshold.as_f64() {
                self.spike_threshold = value;
                info!("Updated spike_threshold to {}", value);
            }
        }

        if let Some(volume_window) = parameters.get("volume_window") {
            if let Some(value) = volume_window.as_i64() {
                self.volume_window = value;
                info!("Updated volume_window to {} minutes", value);
            }
        }

        if let Some(min_volume) = parameters.get("min_volume") {
            if let Some(value) = min_volume.as_f64() {
                self.min_volume = value;
                info!("Updated min_volume to {}", value);
            }
        }

        if let Some(cooldown_period) = parameters.get("cooldown_period") {
            if let Some(value) = cooldown_period.as_i64() {
                self.cooldown_period = value;
                info!("Updated cooldown_period to {} minutes", value);
            }
        }

        if let Some(enabled) = parameters.get("enabled") {
            if let Some(value) = enabled.as_bool() {
                self.enabled = value;
                info!("Volume Spike Strategy {}", if value { "enabled" } else { "disabled" });
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_confidence(&self) -> f64 {
        0.8 // High confidence for volume spike detection
    }

    fn required_data_sources(&self) -> Vec<String> {
        vec!["volume".to_string(), "price".to_string()]
    }

    fn can_operate(&self, context: &StrategyContext) -> bool {
        // Can operate if we have volume data and price data
        context.aggregated_data.primary_data.volume > 0.0
            && context.aggregated_data.primary_data.price > 0.0
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::VolumeSpike
    }

    fn min_confidence_threshold(&self) -> f64 {
        0.6
    }

    async fn on_market_event(&self, event: &MarketEvent, _context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }

        // Create a mutable copy to update volume history
        let mut strategy = self.clone();

        match event {
            MarketEvent::PriceUpdate { symbol, price, volume_24h, timestamp, .. } => {
                if let Some(volume) = volume_24h {
                    let timestamp = DateTime::from_timestamp(*timestamp as i64, 0)
                        .unwrap_or_else(|| Utc::now());

                    strategy.update_volume_history(symbol, *volume, *price, timestamp);

                    if let Some(trigger) = strategy.detect_volume_spike(symbol, *volume, *price, timestamp) {
                        let signal = strategy.generate_signal(trigger);
                        return Ok(Some(signal));
                    }
                }
            }

            MarketEvent::NewTransaction { token_address, amount, price, timestamp, .. } => {
                if let Some(price_val) = price {
                    let timestamp = DateTime::from_timestamp(*timestamp as i64, 0)
                        .unwrap_or_else(|| Utc::now());

                    strategy.update_volume_history(token_address, *amount, *price_val, timestamp);

                    if let Some(trigger) = strategy.detect_volume_spike(token_address, *amount, *price_val, timestamp) {
                        let signal = strategy.generate_signal(trigger);
                        return Ok(Some(signal));
                    }
                }
            }

            _ => {
                // Ignore other event types
            }
        }

        Ok(None)
    }

    fn is_interested_in_event(&self, event: &MarketEvent) -> bool {
        matches!(event,
            MarketEvent::PriceUpdate { .. } |
            MarketEvent::NewTransaction { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_volume_spike_detection() {
        let mut strategy = VolumeSpikeStrategy::new();
        let symbol = "SOL/USDC";
        let base_time = Utc::now();

        // Add some baseline volume data
        for i in 0..5 {
            strategy.update_volume_history(
                symbol,
                1000.0, // Normal volume
                100.0,  // Price
                base_time + Duration::minutes(i),
            );
        }

        // Test volume spike detection
        let spike_trigger = strategy.detect_volume_spike(
            symbol,
            4000.0, // 4x normal volume
            105.0,  // Slightly higher price
            base_time + Duration::minutes(10),
        );

        assert!(spike_trigger.is_some());
        let trigger = spike_trigger.unwrap();
        assert_eq!(trigger.symbol, symbol);
        assert!(trigger.spike_ratio >= 3.0);
    }

    #[tokio::test]
    async fn test_cooldown_period() {
        let mut strategy = VolumeSpikeStrategy::new();
        let symbol = "SOL/USDC";
        let base_time = Utc::now();

        // Add baseline data
        for i in 0..5 {
            strategy.update_volume_history(symbol, 1000.0, 100.0, base_time + Duration::minutes(i));
        }

        // First spike should be detected
        let first_spike = strategy.detect_volume_spike(symbol, 4000.0, 105.0, base_time + Duration::minutes(10));
        assert!(first_spike.is_some());

        // Generate signal to update last_signals
        if let Some(trigger) = first_spike {
            strategy.generate_signal(trigger);
        }

        // Second spike within cooldown should be ignored
        let second_spike = strategy.detect_volume_spike(symbol, 5000.0, 110.0, base_time + Duration::minutes(15));
        assert!(second_spike.is_none());

        // Spike after cooldown should be detected
        let third_spike = strategy.detect_volume_spike(symbol, 4500.0, 108.0, base_time + Duration::minutes(30));
        assert!(third_spike.is_some());
    }
}

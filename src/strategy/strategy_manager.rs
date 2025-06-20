use crate::models::{MarketEvent, StrategySignal, TradingResult};
use crate::strategy::enhanced_strategy::{EnhancedStrategy, StrategyContext};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn, error, instrument};

/// Manages multiple trading strategies and coordinates their execution
pub struct StrategyManager {
    strategies: Arc<RwLock<HashMap<String, Arc<dyn EnhancedStrategy + Send + Sync>>>>,
    signal_sender: mpsc::Sender<StrategySignal>,
    active_strategies: Arc<RwLock<Vec<String>>>,
    strategy_performance: Arc<RwLock<HashMap<String, StrategyPerformance>>>,
    portfolio_aware_activation: bool,
    last_balance_check: Arc<RwLock<f64>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StrategyPerformance {
    pub signals_generated: u64,
    pub successful_signals: u64,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub avg_signal_strength: f64,
    pub last_signal_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for StrategyPerformance {
    fn default() -> Self {
        Self {
            signals_generated: 0,
            successful_signals: 0,
            total_pnl: 0.0,
            win_rate: 0.0,
            avg_signal_strength: 0.0,
            last_signal_time: None,
        }
    }
}

impl StrategyManager {
    /// Create new strategy manager
    pub fn new(signal_sender: mpsc::Sender<StrategySignal>) -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            signal_sender,
            active_strategies: Arc::new(RwLock::new(Vec::new())),
            strategy_performance: Arc::new(RwLock::new(HashMap::new())),
            portfolio_aware_activation: true,
            last_balance_check: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Add a strategy to the manager
    #[instrument(skip(self, strategy))]
    pub async fn add_strategy(&self, strategy: Box<dyn EnhancedStrategy + Send + Sync>) -> TradingResult<()> {
        let strategy_name = strategy.get_name().to_string();

        info!("📊 Adding strategy: {}", strategy_name);

        // Add to strategies map
        {
            let mut strategies = self.strategies.write().await;
            strategies.insert(strategy_name.clone(), Arc::from(strategy));
        }
        
        // Add to active strategies if enabled
        if self.is_strategy_enabled(&strategy_name).await {
            let mut active = self.active_strategies.write().await;
            if !active.contains(&strategy_name) {
                active.push(strategy_name.clone());
            }
        }
        
        // Initialize performance tracking
        {
            let mut performance = self.strategy_performance.write().await;
            performance.insert(strategy_name.clone(), StrategyPerformance::default());
        }
        
        info!("✅ Strategy {} added successfully", strategy_name);
        Ok(())
    }

    /// Remove a strategy from the manager
    #[instrument(skip(self))]
    pub async fn remove_strategy(&self, strategy_name: &str) -> TradingResult<()> {
        info!("🗑️ Removing strategy: {}", strategy_name);
        
        // Remove from strategies
        {
            let mut strategies = self.strategies.write().await;
            strategies.remove(strategy_name);
        }
        
        // Remove from active strategies
        {
            let mut active = self.active_strategies.write().await;
            active.retain(|name| name != strategy_name);
        }
        
        // Remove performance data
        {
            let mut performance = self.strategy_performance.write().await;
            performance.remove(strategy_name);
        }
        
        info!("✅ Strategy {} removed successfully", strategy_name);
        Ok(())
    }

    /// Process market event and distribute to interested strategies
    #[instrument(skip(self, context), fields(event_type = ?std::mem::discriminant(event)))]
    pub async fn process_market_event(&self, event: &MarketEvent, context: &StrategyContext) -> TradingResult<Vec<StrategySignal>> {
        let mut signals = Vec::new();
        let active_strategies = self.active_strategies.read().await.clone();
        
        debug!("📡 Processing market event for {} active strategies", active_strategies.len());
        
        for strategy_name in active_strategies {
            if let Some(strategy) = self.get_strategy(&strategy_name).await {
                // Check if strategy is interested in this event type
                if strategy.is_interested_in_event(event) {
                    debug!("🎯 Strategy {} is interested in event", strategy_name);
                    
                    match strategy.on_market_event(event, context).await {
                        Ok(Some(signal)) => {
                            info!("📈 Signal generated by {}: {:?}", strategy_name, signal.signal_type);

                            // TODO: Consult AI Decision Engine if available
                            // if let Some(final_signal) = self.consult_ai_for_signal(event, context, &signal).await {
                            //     signal = final_signal;
                            //     info!("🤖 AI enhanced signal: {:?}", signal.signal_type);
                            // }

                            // Update performance metrics
                            self.update_strategy_performance(&strategy_name, &signal).await;

                            // Send signal to execution engine
                            if let Err(e) = self.signal_sender.send(signal.clone()).await {
                                error!("Failed to send signal from {}: {}", strategy_name, e);
                            } else {
                                signals.push(signal);
                            }
                        }
                        Ok(None) => {
                            debug!("Strategy {} did not generate signal", strategy_name);
                        }
                        Err(e) => {
                            warn!("Strategy {} error processing event: {}", strategy_name, e);
                        }
                    }
                } else {
                    debug!("Strategy {} not interested in this event type", strategy_name);
                }
            }
        }
        
        if !signals.is_empty() {
            info!("🚀 Generated {} signals from market event", signals.len());
        }
        
        Ok(signals)
    }

    /// Run periodic analysis for all active strategies
    #[instrument(skip(self, context))]
    pub async fn run_periodic_analysis(&self, context: &StrategyContext) -> TradingResult<Vec<StrategySignal>> {
        let mut signals = Vec::new();
        let active_strategies = self.active_strategies.read().await.clone();
        
        info!("🔄 Running periodic analysis for {} strategies", active_strategies.len());
        
        for strategy_name in active_strategies {
            if let Some(strategy) = self.get_strategy(&strategy_name).await {
                match strategy.analyze(context).await {
                    Ok(Some(signal)) => {
                        info!("📊 Periodic signal generated by {}: {:?}", strategy_name, signal.signal_type);
                        
                        // Update performance metrics
                        self.update_strategy_performance(&strategy_name, &signal).await;
                        
                        // Send signal to execution engine
                        if let Err(e) = self.signal_sender.send(signal.clone()).await {
                            error!("Failed to send periodic signal from {}: {}", strategy_name, e);
                        } else {
                            signals.push(signal);
                        }
                    }
                    Ok(None) => {
                        debug!("Strategy {} did not generate periodic signal", strategy_name);
                    }
                    Err(e) => {
                        warn!("Strategy {} error in periodic analysis: {}", strategy_name, e);
                    }
                }
            }
        }
        
        if !signals.is_empty() {
            info!("📈 Generated {} signals from periodic analysis", signals.len());
        }
        
        Ok(signals)
    }

    /// Enable a strategy
    pub async fn enable_strategy(&self, strategy_name: &str) -> TradingResult<()> {
        let mut active = self.active_strategies.write().await;
        if !active.contains(&strategy_name.to_string()) {
            active.push(strategy_name.to_string());
            info!("✅ Strategy {} enabled", strategy_name);
        }
        Ok(())
    }

    /// Disable a strategy
    pub async fn disable_strategy(&self, strategy_name: &str) -> TradingResult<()> {
        let mut active = self.active_strategies.write().await;
        active.retain(|name| name != strategy_name);
        info!("⏸️ Strategy {} disabled", strategy_name);
        Ok(())
    }

    /// Get strategy performance metrics
    pub async fn get_strategy_performance(&self, strategy_name: &str) -> Option<StrategyPerformance> {
        let performance = self.strategy_performance.read().await;
        performance.get(strategy_name).cloned()
    }

    /// Get all strategy performance metrics
    pub async fn get_all_performance(&self) -> HashMap<String, StrategyPerformance> {
        self.strategy_performance.read().await.clone()
    }

    /// Get list of active strategies
    pub async fn get_active_strategies(&self) -> Vec<String> {
        self.active_strategies.read().await.clone()
    }

    /// Get list of all strategies
    pub async fn get_all_strategies(&self) -> Vec<String> {
        let strategies = self.strategies.read().await;
        strategies.keys().cloned().collect()
    }

    // Private helper methods
    
    async fn get_strategy(&self, strategy_name: &str) -> Option<Arc<dyn EnhancedStrategy + Send + Sync>> {
        let strategies = self.strategies.read().await;
        strategies.get(strategy_name).cloned()
    }

    async fn is_strategy_enabled(&self, strategy_name: &str) -> bool {
        if let Some(strategy) = self.get_strategy(strategy_name).await {
            strategy.is_enabled()
        } else {
            false
        }
    }

    async fn update_strategy_performance(&self, strategy_name: &str, signal: &StrategySignal) {
        let mut performance = self.strategy_performance.write().await;
        if let Some(perf) = performance.get_mut(strategy_name) {
            perf.signals_generated += 1;
            perf.last_signal_time = Some(signal.timestamp);
            
            // Update average signal strength
            let total_strength = perf.avg_signal_strength * (perf.signals_generated - 1) as f64 + signal.strength;
            perf.avg_signal_strength = total_strength / perf.signals_generated as f64;
            
            debug!("📊 Updated performance for {}: {} signals, avg strength: {:.3}",
                strategy_name, perf.signals_generated, perf.avg_signal_strength);
        }
    }

    /// Get performance statistics for all strategies
    pub async fn get_performance_stats(&self) -> Vec<(String, StrategyPerformance)> {
        let performance = self.strategy_performance.read().await;
        performance.iter()
            .map(|(name, perf)| (name.clone(), perf.clone()))
            .collect()
    }

    /// Update active strategies based on current portfolio balance
    pub async fn update_strategies_for_balance(&self, sol_balance: f64) -> TradingResult<()> {
        if !self.portfolio_aware_activation {
            return Ok(());
        }

        let mut last_balance = self.last_balance_check.write().await;

        // Only update if balance changed significantly (>5%)
        if (*last_balance - sol_balance).abs() / *last_balance < 0.05 && *last_balance > 0.0 {
            return Ok(());
        }

        *last_balance = sol_balance;
        drop(last_balance);

        info!("🔄 Updating strategies for balance: {:.3} SOL", sol_balance);

        let mut active_strategies = self.active_strategies.write().await;
        active_strategies.clear();

        // Strategy activation based on balance
        // 🧪 TESTING MODE: Lowered thresholds for local testing
        match sol_balance {
            // < 0.001 SOL: Only PumpFun sniping (low fees, high risk/reward)
            balance if balance < 0.001 => {
                active_strategies.push("pumpfun_sniping".to_string());
                info!("💰 Low balance mode: Only PumpFun sniping active");
            }

            // 0.001 - 0.01 SOL: PumpFun + Liquidity sniping (TESTING THRESHOLD)
            balance if balance < 0.01 => {
                active_strategies.push("pumpfun_sniping".to_string());
                active_strategies.push("liquidity_sniping".to_string());
                info!("💰 Medium balance mode: PumpFun + Liquidity sniping active");
            }

            // > 0.01 SOL: All strategies including arbitrage (TESTING THRESHOLD)
            _ => {
                active_strategies.push("pumpfun_sniping".to_string());
                active_strategies.push("liquidity_sniping".to_string());
                active_strategies.push("helius_arbitrage".to_string());
                active_strategies.push("meteora_dlmm".to_string());
                info!("💰 High balance mode: All strategies active");
            }
        }

        info!("✅ Active strategies updated: {:?}", *active_strategies);
        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::strategy::pumpfun_sniping::PumpFunSnipingStrategy;

    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_strategy_manager_creation() {
        let (signal_sender, _signal_receiver) = mpsc::channel(100);
        let manager = StrategyManager::new(signal_sender);
        
        assert_eq!(manager.get_active_strategies().await.len(), 0);
        assert_eq!(manager.get_all_strategies().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_strategy() {
        let (signal_sender, _signal_receiver) = mpsc::channel(100);
        let manager = StrategyManager::new(signal_sender);
        
        let strategy = Box::new(PumpFunSnipingStrategy::new("test_strategy".to_string()));
        
        // Add strategy
        manager.add_strategy(strategy).await.unwrap();
        assert_eq!(manager.get_all_strategies().await.len(), 1);
        
        // Remove strategy
        manager.remove_strategy("test_strategy").await.unwrap();
        assert_eq!(manager.get_all_strategies().await.len(), 0);
    }

    #[tokio::test]
    async fn test_enable_disable_strategy() {
        let (signal_sender, _signal_receiver) = mpsc::channel(100);
        let manager = StrategyManager::new(signal_sender);
        
        // Enable non-existent strategy should work
        manager.enable_strategy("test").await.unwrap();
        assert_eq!(manager.get_active_strategies().await.len(), 1);
        
        // Disable strategy
        manager.disable_strategy("test").await.unwrap();
        assert_eq!(manager.get_active_strategies().await.len(), 0);
    }
}

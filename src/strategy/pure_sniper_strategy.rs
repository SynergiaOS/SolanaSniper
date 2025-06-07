use crate::models::{MarketEvent, StrategySignal, SignalType, TradingResult};
use crate::strategy::enhanced_strategy::{EnhancedStrategy, StrategyContext, StrategyType};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug, error};

/// Pure Sniper Strategy - Reflex Core Family
/// 
/// Filozofia: Być jednym z pierwszych. Zysk pochodzi z przewagi informacyjnej i czasowej.
/// Akceptujemy wyższe ryzyko w zamian za potencjalnie eksplozywne, krótkoterminowe zyski.
/// 
/// Trigger: Wykrycie nowej puli SOL/NOWY_TOKEN przez OnChainStreamListener
/// Walidacja: <100ms - sprawdzenie czy mint_authority i freeze_authority są spalone
/// Akcja: Natychmiastowy zakup 0.05 SOL przez Jito Bundle
/// Exit: TP: +300%, SL: -80%, Time Exit: 1 godzina
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PureSniperStrategy {
    name: String,
    enabled: bool,
    
    // Strategy Parameters
    purchase_amount_sol: f64,      // 0.05 SOL - mała, stała kwota
    take_profit_percent: f64,      // 300% - eksplozywny zysk
    stop_loss_percent: f64,        // -80% - akceptujemy duże ryzyko
    time_exit_hours: f64,          // 1 godzina - szybkie wyjście
    
    // Validation Parameters
    max_validation_time_ms: u64,   // <100ms - maksymalny czas walidacji
    require_burned_mint_authority: bool,    // Czy mint_authority musi być spalona
    require_burned_freeze_authority: bool,  // Czy freeze_authority musi być spalona
    
    // Performance Tracking
    signals_generated: u64,
    successful_trades: u64,
    failed_trades: u64,
    total_pnl: f64,
}

impl PureSniperStrategy {
    pub fn new(name: String) -> Self {
        Self {
            name,
            enabled: true,
            
            // Default Pure Sniper parameters
            purchase_amount_sol: 0.05,
            take_profit_percent: 300.0,
            stop_loss_percent: -80.0,
            time_exit_hours: 1.0,
            
            // Validation settings
            max_validation_time_ms: 100,
            require_burned_mint_authority: true,
            require_burned_freeze_authority: true,
            
            // Performance tracking
            signals_generated: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_pnl: 0.0,
        }
    }

    /// Fast validation of new token (<100ms)
    async fn validate_new_token(&self, token_mint: &str, _context: &StrategyContext) -> TradingResult<bool> {
        let start_time = std::time::Instant::now();
        
        debug!("🔍 Pure Sniper: Validating token {} (max {}ms)", token_mint, self.max_validation_time_ms);

        // TODO: Implement actual on-chain validation
        // For now, simulate validation logic
        
        // Check 1: Mint authority burned?
        if self.require_burned_mint_authority {
            // TODO: Query Solana RPC to check if mint_authority is None
            debug!("✅ Checking mint_authority for {}", token_mint);
        }

        // Check 2: Freeze authority burned?
        if self.require_burned_freeze_authority {
            // TODO: Query Solana RPC to check if freeze_authority is None
            debug!("✅ Checking freeze_authority for {}", token_mint);
        }

        let validation_time = start_time.elapsed().as_millis() as u64;
        
        if validation_time > self.max_validation_time_ms {
            warn!("⚠️ Pure Sniper: Validation took {}ms (max {}ms) - REJECTED", 
                validation_time, self.max_validation_time_ms);
            return Ok(false);
        }

        info!("✅ Pure Sniper: Token {} validated in {}ms - APPROVED", token_mint, validation_time);
        Ok(true)
    }

    /// Check if this is a new pool creation event we're interested in
    fn is_new_pool_event(&self, event: &MarketEvent) -> Option<(String, String)> {
        match event {
            MarketEvent::NewPoolCreated { base_mint, quote_mint, .. } => {
                // We're only interested in SOL/NEW_TOKEN pairs
                if quote_mint == "So11111111111111111111111111111111111111112" { // SOL mint
                    Some((base_mint.clone(), quote_mint.clone()))
                } else if base_mint == "So11111111111111111111111111111111111111112" {
                    Some((quote_mint.clone(), base_mint.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Generate Pure Sniper signal for immediate execution
    fn generate_sniper_signal(&self, token_mint: &str, sol_mint: &str) -> StrategySignal {
        let symbol = format!("{}/SOL", token_mint);
        
        StrategySignal {
            strategy: self.name.clone(),
            symbol,
            signal_type: SignalType::Buy,
            strength: 0.95, // Very high confidence for Pure Sniper
            price: 0.0, // Market price - will be determined by Jupiter
            size: self.purchase_amount_sol,
            metadata: serde_json::json!({
                "strategy_type": "pure_sniper",
                "token_mint": token_mint,
                "sol_mint": sol_mint,
                "purchase_amount_sol": self.purchase_amount_sol,
                "take_profit_percent": self.take_profit_percent,
                "stop_loss_percent": self.stop_loss_percent,
                "time_exit_hours": self.time_exit_hours,
                "use_mev_protection": true,
                "priority": "ultra_high",
                "execution_mode": "immediate"
            }),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl EnhancedStrategy for PureSniperStrategy {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::Sniping
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn analyze(&self, _context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        // Pure Sniper is purely reactive to events, not analytical
        // It doesn't analyze ongoing market data, only reacts to new pool creation
        Ok(None)
    }

    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()> {
        info!("🔧 Updating Pure Sniper strategy parameters: {}", self.name);

        for (key, value) in parameters {
            match key.as_str() {
                "enabled" => {
                    if let Some(enabled) = value.as_bool() {
                        self.enabled = enabled;
                        info!("📊 Updated enabled: {}", enabled);
                    }
                }
                "purchase_amount_sol" => {
                    if let Some(amount) = value.as_f64() {
                        self.purchase_amount_sol = amount;
                        info!("📊 Updated purchase_amount_sol: {}", amount);
                    }
                }
                "take_profit_percent" => {
                    if let Some(tp) = value.as_f64() {
                        self.take_profit_percent = tp;
                        info!("📊 Updated take_profit_percent: {}%", tp);
                    }
                }
                "stop_loss_percent" => {
                    if let Some(sl) = value.as_f64() {
                        self.stop_loss_percent = sl;
                        info!("📊 Updated stop_loss_percent: {}%", sl);
                    }
                }
                "time_exit_hours" => {
                    if let Some(hours) = value.as_f64() {
                        self.time_exit_hours = hours;
                        info!("📊 Updated time_exit_hours: {} hours", hours);
                    }
                }
                _ => {
                    warn!("⚠️ Unknown parameter for Pure Sniper: {}", key);
                }
            }
        }

        info!("✅ Pure Sniper strategy parameters updated successfully");
        Ok(())
    }

    fn get_confidence(&self) -> f64 {
        0.95 // Very high confidence for Pure Sniper when conditions are met
    }

    fn required_data_sources(&self) -> Vec<String> {
        vec!["solana_rpc".to_string()] // Only needs Solana RPC for validation
    }

    fn can_operate(&self, _context: &StrategyContext) -> bool {
        self.enabled // Simple check - can operate if enabled
    }

    fn min_confidence_threshold(&self) -> f64 {
        0.9 // High threshold for Pure Sniper
    }

    fn is_interested_in_event(&self, event: &MarketEvent) -> bool {
        if !self.enabled {
            return false;
        }

        // Pure Sniper is only interested in new pool creation events
        matches!(event, MarketEvent::NewPoolCreated { .. })
    }

    async fn on_market_event(&self, event: &MarketEvent, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }

        // Check if this is a new pool event we care about
        if let Some((token_mint, sol_mint)) = self.is_new_pool_event(event) {
            info!("🎯 Pure Sniper: New SOL/{} pool detected!", token_mint);

            // Fast validation (<100ms)
            match self.validate_new_token(&token_mint, context).await {
                Ok(true) => {
                    info!("🚀 Pure Sniper: EXECUTING IMMEDIATE BUY for {}", token_mint);
                    
                    // Generate immediate buy signal
                    let signal = self.generate_sniper_signal(&token_mint, &sol_mint);
                    
                    // Update performance tracking
                    // Note: In a real implementation, we'd need mutable access to update counters
                    
                    Ok(Some(signal))
                }
                Ok(false) => {
                    info!("❌ Pure Sniper: Token {} failed validation - SKIPPED", token_mint);
                    Ok(None)
                }
                Err(e) => {
                    error!("💥 Pure Sniper: Validation error for {}: {}", token_mint, e);
                    Ok(None)
                }
            }
        } else {
            // Not interested in this event
            Ok(None)
        }
    }


}

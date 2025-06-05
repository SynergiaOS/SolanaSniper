use async_trait::async_trait;
use crate::models::{DataSource, MarketEvent, StrategySignal, SignalType, TradingResult};
use crate::strategy::{EnhancedStrategy, StrategyContext, StrategyType};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, debug};

/// Represents a triangular arbitrage path (e.g., SOL -> USDC -> BONK -> SOL)
#[derive(Debug, Clone)]
pub struct TriangularPath {
    pub tokens: Vec<String>,           // Token symbols in order (e.g., ["SOL", "USDC", "BONK"])
    pub dex_sources: Vec<String>,      // DEX for each hop (e.g., ["jupiter", "raydium", "pump_fun"])
    pub estimated_fees: Vec<f64>,      // Fee percentage for each hop
    pub min_liquidity_per_hop: Vec<f64>, // Minimum liquidity required for each hop
}

/// Enhanced DEX source information
#[derive(Debug, Clone)]
pub struct DexSource {
    pub name: String,
    pub base_fee_percentage: f64,
    pub supports_bundles: bool,
    pub average_slippage: f64,
    pub reliability_score: f64,        // 0.0 - 1.0
}

/// ArbitrageStrategy detects and exploits price discrepancies across different exchanges
/// Supports CEX-DEX, DEX-DEX, and triangular arbitrage opportunities
pub struct ArbitrageStrategy {
    name: String,
    config: ArbitrageConfig,
    last_opportunity_time: Option<Instant>,
    recent_opportunities: HashMap<String, Instant>, // Track recent opportunities per pair
    enabled: bool,
    dex_sources: HashMap<String, DexSource>,       // Available DEX sources
}

#[derive(Debug, Clone)]
pub struct ArbitrageConfig {
    pub min_profit_percentage: f64,    // Minimum profit after fees (e.g., 0.5%)
    pub max_slippage_percentage: f64,  // Maximum acceptable slippage per leg
    pub min_volume_24h: f64,           // Minimum 24h volume for consideration
    pub max_position_size: f64,        // Maximum position size in SOL
    pub cooldown_seconds: u64,         // Cooldown between opportunities
    pub preferred_pairs: Vec<String>,  // Preferred trading pairs
    pub min_liquidity: f64,            // Minimum liquidity required
    pub max_price_impact: f64,         // Maximum price impact allowed

    // Enhanced arbitrage features
    pub enable_triangular: bool,       // Enable triangular arbitrage
    pub enable_dex_dex: bool,          // Enable DEX-DEX arbitrage
    pub enable_jito_bundles: bool,     // Enable Jito bundle execution
    pub max_triangular_hops: u8,       // Maximum hops for triangular arbitrage (default: 3)
    pub dex_sources: Vec<String>,      // Supported DEX sources
    pub triangular_pairs: Vec<TriangularPath>, // Predefined triangular paths
    pub bundle_tip_lamports: u64,      // Jito bundle tip amount
}

impl Default for ArbitrageConfig {
    fn default() -> Self {
        Self {
            min_profit_percentage: 0.005,  // 0.5% minimum profit
            max_slippage_percentage: 0.003, // 0.3% max slippage
            min_volume_24h: 50000.0,       // $50k minimum volume
            max_position_size: 10.0,       // 10 SOL max position
            cooldown_seconds: 300,         // 5 minute cooldown
            preferred_pairs: vec![
                "SOL/USDC".to_string(),
                "SOL/USDT".to_string(),
                "BONK/SOL".to_string(),
                "WIF/SOL".to_string(),
            ],
            min_liquidity: 10000.0,        // $10k minimum liquidity
            max_price_impact: 0.02,        // 2% max price impact

            // Enhanced arbitrage features
            enable_triangular: true,
            enable_dex_dex: true,
            enable_jito_bundles: true,
            max_triangular_hops: 3,
            dex_sources: vec![
                "jupiter".to_string(),
                "raydium".to_string(),
                "pump_fun".to_string(),
                "meteora".to_string(),
            ],
            triangular_pairs: vec![
                TriangularPath {
                    tokens: vec!["SOL".to_string(), "USDC".to_string(), "BONK".to_string()],
                    dex_sources: vec!["jupiter".to_string(), "raydium".to_string(), "pump_fun".to_string()],
                    estimated_fees: vec![0.003, 0.0025, 0.003], // 0.3%, 0.25%, 0.3%
                    min_liquidity_per_hop: vec![5000.0, 3000.0, 2000.0],
                },
                TriangularPath {
                    tokens: vec!["SOL".to_string(), "USDT".to_string(), "WIF".to_string()],
                    dex_sources: vec!["jupiter".to_string(), "raydium".to_string(), "jupiter".to_string()],
                    estimated_fees: vec![0.003, 0.0025, 0.003],
                    min_liquidity_per_hop: vec![5000.0, 3000.0, 1500.0],
                },
            ],
            bundle_tip_lamports: 10000, // 0.00001 SOL tip
        }
    }
}

impl ArbitrageStrategy {
    pub fn new(name: String) -> Self {
        let mut strategy = Self {
            name,
            config: ArbitrageConfig::default(),
            last_opportunity_time: None,
            recent_opportunities: HashMap::new(),
            enabled: true,
            dex_sources: HashMap::new(),
        };
        strategy.initialize_dex_sources();
        strategy
    }

    pub fn with_config(name: String, config: ArbitrageConfig) -> Self {
        let mut strategy = Self {
            name,
            config,
            last_opportunity_time: None,
            recent_opportunities: HashMap::new(),
            enabled: true,
            dex_sources: HashMap::new(),
        };
        strategy.initialize_dex_sources();
        strategy
    }

    /// Initialize DEX sources with their characteristics
    fn initialize_dex_sources(&mut self) {
        self.dex_sources.insert("jupiter".to_string(), DexSource {
            name: "Jupiter".to_string(),
            base_fee_percentage: 0.003,
            supports_bundles: true,
            average_slippage: 0.002,
            reliability_score: 0.95,
        });

        self.dex_sources.insert("raydium".to_string(), DexSource {
            name: "Raydium".to_string(),
            base_fee_percentage: 0.0025,
            supports_bundles: true,
            average_slippage: 0.0015,
            reliability_score: 0.92,
        });

        self.dex_sources.insert("pump_fun".to_string(), DexSource {
            name: "Pump.fun".to_string(),
            base_fee_percentage: 0.003,
            supports_bundles: false,
            average_slippage: 0.005,
            reliability_score: 0.85,
        });

        self.dex_sources.insert("meteora".to_string(), DexSource {
            name: "Meteora".to_string(),
            base_fee_percentage: 0.002,
            supports_bundles: true,
            average_slippage: 0.0018,
            reliability_score: 0.90,
        });
    }

    /// Calculate potential arbitrage profit between two price sources
    fn calculate_arbitrage_profit(
        &self,
        buy_price: f64,
        sell_price: f64,
        buy_liquidity: f64,
        sell_liquidity: f64,
        volume: f64,
    ) -> Option<ArbitrageOpportunity> {
        if buy_price <= 0.0 || sell_price <= 0.0 || buy_price >= sell_price {
            return None;
        }

        // Calculate gross profit percentage
        let gross_profit_pct = (sell_price - buy_price) / buy_price;

        // Estimate fees and slippage
        let estimated_fees = 0.002; // 0.2% total fees (buy + sell)
        let price_impact = (volume / buy_liquidity.min(sell_liquidity)).min(self.config.max_price_impact);
        let estimated_slippage = price_impact * 2.0; // Both legs

        // Calculate net profit
        let net_profit_pct = gross_profit_pct - estimated_fees - estimated_slippage;

        if net_profit_pct > self.config.min_profit_percentage {
            Some(ArbitrageOpportunity {
                buy_price,
                sell_price,
                gross_profit_pct,
                net_profit_pct,
                estimated_fees,
                estimated_slippage,
                max_volume: volume,
            })
        } else {
            None
        }
    }

    /// Detect arbitrage opportunities from market context
    fn detect_arbitrage_opportunities(&self, context: &StrategyContext) -> Vec<(String, ArbitrageOpportunity)> {
        let mut opportunities = Vec::new();
        
        // Get multi-source price data from aggregated data
        let primary_price = context.aggregated_data.primary_data.price;
        let symbol = &context.aggregated_data.primary_data.symbol;

        // Check if this pair is in our preferred list
        if !self.config.preferred_pairs.iter().any(|pair| symbol.contains(&pair.split('/').next().unwrap_or(""))) {
            return opportunities;
        }

        // Analyze secondary data sources for price discrepancies
        for secondary_data in &context.aggregated_data.secondary_data {
            if let Some(opportunity) = self.calculate_arbitrage_profit(
                primary_price.min(secondary_data.price),
                primary_price.max(secondary_data.price),
                context.market_conditions.liquidity_depth,
                context.market_conditions.liquidity_depth * 0.8, // Assume secondary has less liquidity
                self.config.max_position_size * primary_price,
            ) {
                debug!(
                    "Arbitrage opportunity detected for {}: {:.3}% net profit",
                    symbol, opportunity.net_profit_pct * 100.0
                );
                opportunities.push((symbol.clone(), opportunity));
            }
        }

        opportunities
    }

    /// Check if we're in cooldown for a specific pair
    fn is_in_cooldown(&self, pair: &str) -> bool {
        if let Some(last_time) = self.recent_opportunities.get(pair) {
            last_time.elapsed().as_secs() < self.config.cooldown_seconds
        } else {
            false
        }
    }

    /// Generate trading signal for arbitrage opportunity
    fn generate_arbitrage_signal(
        &self,
        pair: &str,
        opportunity: &ArbitrageOpportunity,
        context: &StrategyContext,
    ) -> StrategySignal {
        let signal_strength = (opportunity.net_profit_pct / self.config.min_profit_percentage)
            .min(1.0)
            .max(0.0);

        let confidence = if context.aggregated_data.sources_count >= 3 {
            0.9 // High confidence with multiple sources
        } else {
            0.7 // Lower confidence with fewer sources
        };

        StrategySignal {
            strategy: self.name.clone(),
            signal_type: SignalType::Buy, // Arbitrage starts with a buy
            symbol: pair.to_string(),
            price: opportunity.buy_price,
            strength: signal_strength,
            size: self.config.max_position_size,
            metadata: serde_json::json!({
                "arbitrage_type": "traditional",
                "buy_price": opportunity.buy_price,
                "sell_price": opportunity.sell_price,
                "gross_profit_pct": format!("{:.4}", opportunity.gross_profit_pct * 100.0),
                "net_profit_pct": format!("{:.4}", opportunity.net_profit_pct * 100.0),
                "estimated_fees": format!("{:.4}", opportunity.estimated_fees * 100.0),
                "estimated_slippage": format!("{:.4}", opportunity.estimated_slippage * 100.0),
                "max_volume": opportunity.max_volume,
                "confidence": confidence,
            }),
            timestamp: Utc::now(),
        }
    }

    /// Detect triangular arbitrage opportunities
    fn detect_triangular_opportunities(&self, context: &StrategyContext) -> Vec<TriangularOpportunity> {
        if !self.config.enable_triangular {
            return Vec::new();
        }

        let mut opportunities = Vec::new();

        for path in &self.config.triangular_pairs {
            if let Some(opportunity) = self.analyze_triangular_path(path, context) {
                opportunities.push(opportunity);
            }
        }

        opportunities
    }

    /// Analyze a specific triangular arbitrage path
    fn analyze_triangular_path(&self, path: &TriangularPath, _context: &StrategyContext) -> Option<TriangularOpportunity> {
        if path.tokens.len() < 3 || path.dex_sources.len() != path.tokens.len() {
            return None;
        }

        // Simulate prices for triangular path (in real implementation, fetch from multiple DEXs)
        let mut total_fees = 0.0;
        let mut total_slippage = 0.0;
        let mut min_liquidity = f64::MAX;
        let mut estimated_profit = 1.0; // Start with 1.0 (100%)

        // Calculate profit through the triangular path
        for (i, hop_dex) in path.dex_sources.iter().enumerate() {
            if let Some(dex_info) = self.dex_sources.get(hop_dex) {
                // Add fees for this hop
                total_fees += dex_info.base_fee_percentage;
                total_slippage += dex_info.average_slippage;

                // Track minimum liquidity
                if i < path.min_liquidity_per_hop.len() {
                    min_liquidity = min_liquidity.min(path.min_liquidity_per_hop[i]);
                }

                // Simulate price impact (in real implementation, get actual prices)
                let price_impact = 1.0 - (dex_info.average_slippage * 0.5); // Simplified
                estimated_profit *= price_impact;
            }
        }

        // Calculate net profit after fees and slippage
        let net_profit = estimated_profit - 1.0 - total_fees - total_slippage;

        if net_profit > self.config.min_profit_percentage && min_liquidity > self.config.min_liquidity {
            Some(TriangularOpportunity {
                path: path.clone(),
                estimated_profit: net_profit,
                total_fees,
                total_slippage,
                min_liquidity,
                execution_complexity: path.tokens.len() as f64,
                bundle_required: path.dex_sources.iter().any(|dex| {
                    self.dex_sources.get(dex).map_or(false, |info| info.supports_bundles)
                }),
            })
        } else {
            None
        }
    }

    /// Detect DEX-DEX arbitrage opportunities
    fn detect_dex_dex_opportunities(&self, context: &StrategyContext) -> Vec<DexDexOpportunity> {
        if !self.config.enable_dex_dex {
            return Vec::new();
        }

        let mut opportunities = Vec::new();
        let symbol = &context.aggregated_data.primary_data.symbol;

        // Compare prices across different DEX sources
        for (i, source1) in self.config.dex_sources.iter().enumerate() {
            for source2 in self.config.dex_sources.iter().skip(i + 1) {
                if let Some(opportunity) = self.analyze_dex_dex_pair(source1, source2, symbol, context) {
                    opportunities.push(opportunity);
                }
            }
        }

        opportunities
    }

    /// Analyze arbitrage opportunity between two DEX sources
    fn analyze_dex_dex_pair(
        &self,
        source1: &str,
        source2: &str,
        symbol: &str,
        context: &StrategyContext,
    ) -> Option<DexDexOpportunity> {
        let dex1_info = self.dex_sources.get(source1)?;
        let dex2_info = self.dex_sources.get(source2)?;

        // Simulate different prices (in real implementation, fetch actual prices)
        let base_price = context.aggregated_data.primary_data.price;
        let price1 = base_price * (1.0 + (dex1_info.reliability_score - 0.9) * 0.01); // Slight variation
        let price2 = base_price * (1.0 + (dex2_info.reliability_score - 0.9) * 0.01);

        if (price1 - price2).abs() / price1.min(price2) < self.config.min_profit_percentage {
            return None;
        }

        let buy_price = price1.min(price2);
        let sell_price = price1.max(price2);
        let buy_dex = if price1 < price2 { source1 } else { source2 };
        let sell_dex = if price1 < price2 { source2 } else { source1 };

        let gross_profit = (sell_price - buy_price) / buy_price;
        let total_fees = dex1_info.base_fee_percentage + dex2_info.base_fee_percentage;
        let total_slippage = dex1_info.average_slippage + dex2_info.average_slippage;
        let net_profit = gross_profit - total_fees - total_slippage;

        if net_profit > self.config.min_profit_percentage {
            Some(DexDexOpportunity {
                symbol: symbol.to_string(),
                buy_dex: buy_dex.to_string(),
                sell_dex: sell_dex.to_string(),
                buy_price,
                sell_price,
                gross_profit,
                net_profit,
                total_fees,
                total_slippage,
                bundle_compatible: dex1_info.supports_bundles && dex2_info.supports_bundles,
            })
        } else {
            None
        }
    }

    /// Convert DataSource enum to string
    fn data_source_to_string(&self, source: &DataSource) -> String {
        match source {
            DataSource::Binance => "binance".to_string(),
            DataSource::Coinbase => "coinbase".to_string(),
            DataSource::Kraken => "kraken".to_string(),
            DataSource::Solana => "helius".to_string(), // Map Solana to Helius
            DataSource::Ethereum => "ethereum".to_string(),
            DataSource::Polygon => "polygon".to_string(),
        }
    }

    /// Check if DataSource is Helius/Solana
    fn is_helius_source(&self, source: &DataSource) -> bool {
        matches!(source, DataSource::Solana)
    }

    /// Detect real-time arbitrage opportunities using Helius WebSocket data
    fn detect_helius_arbitrage_opportunities(&self, context: &StrategyContext) -> Vec<HeliusArbitrageOpportunity> {
        let mut opportunities = Vec::new();
        let symbol = &context.aggregated_data.primary_data.symbol;

        // Check if we have Helius data in the aggregated sources
        let helius_price = self.extract_helius_price(context);
        if helius_price.is_none() {
            return opportunities;
        }

        let helius_price = helius_price.unwrap();

        // Compare Helius real-time price with other DEX prices
        for secondary_data in &context.aggregated_data.secondary_data {
            let source_name = self.data_source_to_string(&secondary_data.source);
            if source_name == "helius" {
                continue; // Skip comparing Helius with itself
            }

            let price_diff_pct = (helius_price - secondary_data.price).abs() / helius_price.min(secondary_data.price);

            if price_diff_pct > self.config.min_profit_percentage {
                let buy_price = helius_price.min(secondary_data.price);
                let sell_price = helius_price.max(secondary_data.price);
                let buy_source = if helius_price < secondary_data.price { "helius" } else { &source_name };
                let sell_source = if helius_price < secondary_data.price { &source_name } else { "helius" };

                // Calculate net profit after fees and slippage
                let gross_profit = (sell_price - buy_price) / buy_price;
                let estimated_fees = 0.003; // 0.3% total fees
                let estimated_slippage = 0.002; // 0.2% estimated slippage
                let net_profit = gross_profit - estimated_fees - estimated_slippage;

                if net_profit > self.config.min_profit_percentage {
                    opportunities.push(HeliusArbitrageOpportunity {
                        symbol: symbol.clone(),
                        helius_price,
                        dex_price: secondary_data.price,
                        dex_source: source_name.clone(),
                        buy_price,
                        sell_price,
                        buy_source: buy_source.to_string(),
                        sell_source: sell_source.to_string(),
                        gross_profit,
                        net_profit,
                        estimated_fees,
                        estimated_slippage,
                        timestamp: context.aggregated_data.primary_data.timestamp,
                        liquidity_score: self.calculate_liquidity_score(context),
                    });
                }
            }
        }

        opportunities
    }

    /// Extract Helius price from aggregated data
    fn extract_helius_price(&self, context: &StrategyContext) -> Option<f64> {
        // Check if primary data is from Helius
        if self.is_helius_source(&context.aggregated_data.primary_data.source) {
            return Some(context.aggregated_data.primary_data.price);
        }

        // Check secondary data sources
        for data in &context.aggregated_data.secondary_data {
            if self.is_helius_source(&data.source) {
                return Some(data.price);
            }
        }

        None
    }

    /// Calculate liquidity score based on market conditions
    fn calculate_liquidity_score(&self, context: &StrategyContext) -> f64 {
        let base_score = if context.market_conditions.liquidity_depth > self.config.min_liquidity {
            0.8
        } else {
            0.4
        };

        // Adjust based on volume
        let volume_multiplier: f64 = if context.aggregated_data.primary_data.volume > self.config.min_volume_24h * 2.0 {
            1.2
        } else if context.aggregated_data.primary_data.volume > self.config.min_volume_24h {
            1.0
        } else {
            0.7
        };

        (base_score * volume_multiplier).min(1.0)
    }

    /// Generate trading signal for Helius arbitrage opportunity
    fn generate_helius_arbitrage_signal(
        &self,
        opportunity: &HeliusArbitrageOpportunity,
        _context: &StrategyContext,
    ) -> StrategySignal {
        let signal_strength = (opportunity.net_profit / self.config.min_profit_percentage)
            .min(1.0)
            .max(0.0);

        let confidence = 0.95 * opportunity.liquidity_score; // High confidence for Helius data

        StrategySignal {
            strategy: self.name.clone(),
            signal_type: SignalType::Buy,
            symbol: opportunity.symbol.clone(),
            price: opportunity.buy_price,
            strength: signal_strength,
            size: self.config.max_position_size,
            metadata: serde_json::json!({
                "arbitrage_type": "helius_dex",
                "helius_price": opportunity.helius_price,
                "dex_price": opportunity.dex_price,
                "dex_source": opportunity.dex_source,
                "buy_source": opportunity.buy_source,
                "sell_source": opportunity.sell_source,
                "gross_profit": format!("{:.4}", opportunity.gross_profit * 100.0),
                "net_profit": format!("{:.4}", opportunity.net_profit * 100.0),
                "estimated_fees": format!("{:.4}", opportunity.estimated_fees * 100.0),
                "estimated_slippage": format!("{:.4}", opportunity.estimated_slippage * 100.0),
                "liquidity_score": opportunity.liquidity_score,
                "confidence": confidence,
                "data_freshness": "real_time",
            }),
            timestamp: Utc::now(),
        }
    }

    /// Generate trading signal for triangular arbitrage opportunity
    fn generate_triangular_signal(
        &self,
        opportunity: &TriangularOpportunity,
        context: &StrategyContext,
    ) -> StrategySignal {
        let signal_strength = (opportunity.estimated_profit / self.config.min_profit_percentage)
            .min(1.0)
            .max(0.0);

        let confidence = if opportunity.bundle_required { 0.95 } else { 0.8 };

        StrategySignal {
            strategy: self.name.clone(),
            signal_type: SignalType::Buy,
            symbol: opportunity.path.tokens[0].clone(), // Start token
            price: context.aggregated_data.primary_data.price,
            strength: signal_strength,
            size: self.config.max_position_size,
            metadata: serde_json::json!({
                "arbitrage_type": "triangular",
                "path_tokens": opportunity.path.tokens.join(" -> "),
                "path_dexes": opportunity.path.dex_sources.join(" -> "),
                "estimated_profit": format!("{:.4}", opportunity.estimated_profit * 100.0),
                "total_fees": format!("{:.4}", opportunity.total_fees * 100.0),
                "total_slippage": format!("{:.4}", opportunity.total_slippage * 100.0),
                "execution_complexity": opportunity.execution_complexity,
                "bundle_required": opportunity.bundle_required,
                "min_liquidity": opportunity.min_liquidity,
                "confidence": confidence,
            }),
            timestamp: Utc::now(),
        }
    }

    /// Generate trading signal for DEX-DEX arbitrage opportunity
    fn generate_dex_dex_signal(
        &self,
        opportunity: &DexDexOpportunity,
        _context: &StrategyContext,
    ) -> StrategySignal {
        let signal_strength = (opportunity.net_profit / self.config.min_profit_percentage)
            .min(1.0)
            .max(0.0);

        let confidence = if opportunity.bundle_compatible { 0.9 } else { 0.75 };

        StrategySignal {
            strategy: self.name.clone(),
            signal_type: SignalType::Buy,
            symbol: opportunity.symbol.clone(),
            price: opportunity.buy_price,
            strength: signal_strength,
            size: self.config.max_position_size,
            metadata: serde_json::json!({
                "arbitrage_type": "dex_dex",
                "buy_dex": opportunity.buy_dex,
                "sell_dex": opportunity.sell_dex,
                "buy_price": opportunity.buy_price,
                "sell_price": opportunity.sell_price,
                "gross_profit": format!("{:.4}", opportunity.gross_profit * 100.0),
                "net_profit": format!("{:.4}", opportunity.net_profit * 100.0),
                "total_fees": format!("{:.4}", opportunity.total_fees * 100.0),
                "total_slippage": format!("{:.4}", opportunity.total_slippage * 100.0),
                "bundle_compatible": opportunity.bundle_compatible,
                "confidence": confidence,
            }),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    buy_price: f64,
    sell_price: f64,
    gross_profit_pct: f64,
    net_profit_pct: f64,
    estimated_fees: f64,
    estimated_slippage: f64,
    max_volume: f64,
}

#[derive(Debug, Clone)]
struct TriangularOpportunity {
    path: TriangularPath,
    estimated_profit: f64,
    total_fees: f64,
    total_slippage: f64,
    min_liquidity: f64,
    execution_complexity: f64,
    bundle_required: bool,
}

#[derive(Debug, Clone)]
struct DexDexOpportunity {
    symbol: String,
    buy_dex: String,
    sell_dex: String,
    buy_price: f64,
    sell_price: f64,
    gross_profit: f64,
    net_profit: f64,
    total_fees: f64,
    total_slippage: f64,
    bundle_compatible: bool,
}

#[derive(Debug, Clone)]
struct HeliusArbitrageOpportunity {
    symbol: String,
    helius_price: f64,
    dex_price: f64,
    dex_source: String,
    buy_price: f64,
    sell_price: f64,
    buy_source: String,
    sell_source: String,
    gross_profit: f64,
    net_profit: f64,
    estimated_fees: f64,
    estimated_slippage: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
    liquidity_score: f64,
}

#[async_trait]
impl EnhancedStrategy for ArbitrageStrategy {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> TradingResult<()> {
        if let Some(min_profit) = parameters.get("min_profit_percentage") {
            if let Some(value) = min_profit.as_f64() {
                self.config.min_profit_percentage = value;
            }
        }

        if let Some(max_slippage) = parameters.get("max_slippage_percentage") {
            if let Some(value) = max_slippage.as_f64() {
                self.config.max_slippage_percentage = value;
            }
        }

        if let Some(enabled) = parameters.get("enabled") {
            if let Some(value) = enabled.as_bool() {
                self.enabled = value;
            }
        }

        Ok(())
    }

    fn get_confidence(&self) -> f64 {
        0.85 // High confidence for arbitrage when opportunities are detected
    }

    fn required_data_sources(&self) -> Vec<String> {
        vec![
            "helius".to_string(),      // Primary Solana data source
            "jupiter".to_string(),     // DEX aggregator
            "raydium".to_string(),     // AMM DEX
            "meteora".to_string(),     // DLMM DEX
            "pump_fun".to_string(),    // Meme token DEX
        ]
    }

    fn can_operate(&self, context: &StrategyContext) -> bool {
        self.enabled
            && context.aggregated_data.sources_count >= 2 // Need at least 2 sources for arbitrage
            && context.aggregated_data.primary_data.volume > self.config.min_volume_24h
    }

    fn get_strategy_type(&self) -> StrategyType {
        StrategyType::Arbitrage
    }

    fn min_confidence_threshold(&self) -> f64 {
        0.7
    }

    async fn analyze(&self, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }

        // Check volume requirements
        if context.aggregated_data.primary_data.volume < self.config.min_volume_24h {
            debug!("ArbitrageStrategy: Volume too low for {}", context.aggregated_data.primary_data.symbol);
            return Ok(None);
        }

        // Check liquidity requirements
        if context.market_conditions.liquidity_depth < self.config.min_liquidity {
            debug!("ArbitrageStrategy: Liquidity too low for {}", context.aggregated_data.primary_data.symbol);
            return Ok(None);
        }

        // 1. Detect traditional arbitrage opportunities
        let traditional_opportunities = self.detect_arbitrage_opportunities(context);

        // 2. Detect triangular arbitrage opportunities
        let triangular_opportunities = self.detect_triangular_opportunities(context);

        // 3. Detect DEX-DEX arbitrage opportunities
        let dex_dex_opportunities = self.detect_dex_dex_opportunities(context);

        // 4. Detect Helius real-time arbitrage opportunities
        let helius_opportunities = self.detect_helius_arbitrage_opportunities(context);

        // Find the best opportunity across all types
        let mut best_signal: Option<StrategySignal> = None;
        let mut best_profit = 0.0;

        // Check traditional arbitrage
        if !traditional_opportunities.is_empty() {
            let (best_pair, best_opportunity) = traditional_opportunities
                .into_iter()
                .max_by(|(_, a), (_, b)| a.net_profit_pct.partial_cmp(&b.net_profit_pct).unwrap())
                .unwrap();

            if !self.is_in_cooldown(&best_pair) && best_opportunity.net_profit_pct > best_profit {
                best_profit = best_opportunity.net_profit_pct;
                best_signal = Some(self.generate_arbitrage_signal(&best_pair, &best_opportunity, context));

                info!(
                    "🔄 Traditional arbitrage opportunity found for {}: {:.3}% net profit",
                    best_pair, best_opportunity.net_profit_pct * 100.0
                );
            }
        }

        // Check triangular arbitrage
        for triangular_opp in triangular_opportunities {
            if triangular_opp.estimated_profit > best_profit {
                best_profit = triangular_opp.estimated_profit;
                best_signal = Some(self.generate_triangular_signal(&triangular_opp, context));

                info!(
                    "🔺 Triangular arbitrage opportunity found: {:.3}% profit via {:?}",
                    triangular_opp.estimated_profit * 100.0, triangular_opp.path.tokens
                );
            }
        }

        // Check DEX-DEX arbitrage
        for dex_dex_opp in dex_dex_opportunities {
            let pair_key = format!("{}_{}_to_{}", dex_dex_opp.symbol, dex_dex_opp.buy_dex, dex_dex_opp.sell_dex);

            if !self.is_in_cooldown(&pair_key) && dex_dex_opp.net_profit > best_profit {
                best_profit = dex_dex_opp.net_profit;
                best_signal = Some(self.generate_dex_dex_signal(&dex_dex_opp, context));

                info!(
                    "🔀 DEX-DEX arbitrage opportunity found for {}: {:.3}% profit ({} -> {})",
                    dex_dex_opp.symbol, dex_dex_opp.buy_dex, dex_dex_opp.sell_dex,
                    dex_dex_opp.net_profit * 100.0
                );
            }
        }

        // Check Helius real-time arbitrage (highest priority due to data freshness)
        for helius_opp in helius_opportunities {
            let pair_key = format!("helius_{}_{}_to_{}", helius_opp.symbol, helius_opp.buy_source, helius_opp.sell_source);

            if !self.is_in_cooldown(&pair_key) && helius_opp.net_profit > best_profit {
                best_profit = helius_opp.net_profit;
                best_signal = Some(self.generate_helius_arbitrage_signal(&helius_opp, context));

                info!(
                    "⚡ Helius real-time arbitrage opportunity found for {}: {:.3}% profit ({} -> {}) [REAL-TIME]",
                    helius_opp.symbol, helius_opp.buy_source, helius_opp.sell_source,
                    helius_opp.net_profit * 100.0
                );
            }
        }

        Ok(best_signal)
    }

    async fn on_market_event(&self, event: &MarketEvent, context: &StrategyContext) -> TradingResult<Option<StrategySignal>> {
        match event {
            MarketEvent::PriceUpdate { .. } | 
            MarketEvent::LiquidityUpdate { .. } => {
                // Price or liquidity updates might create arbitrage opportunities
                self.analyze(context).await
            }
            _ => Ok(None),
        }
    }

    fn is_interested_in_event(&self, event: &MarketEvent) -> bool {
        matches!(event, 
            MarketEvent::PriceUpdate { .. } | 
            MarketEvent::LiquidityUpdate { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrage_strategy_creation() {
        let strategy = ArbitrageStrategy::new("test_arbitrage".to_string());
        assert_eq!(strategy.get_name(), "test_arbitrage");
        assert_eq!(strategy.get_strategy_type(), StrategyType::Arbitrage);
        assert!(strategy.is_enabled());
    }

    #[test]
    fn test_arbitrage_profit_calculation() {
        let strategy = ArbitrageStrategy::new("test".to_string());
        
        // Test profitable opportunity
        let opportunity = strategy.calculate_arbitrage_profit(
            100.0, // buy price
            101.0, // sell price (1% difference)
            50000.0, // buy liquidity
            50000.0, // sell liquidity
            1000.0, // volume
        );
        
        assert!(opportunity.is_some());
        let opp = opportunity.unwrap();
        assert!(opp.net_profit_pct > 0.0);
        
        // Test unprofitable opportunity (too small spread)
        let no_opportunity = strategy.calculate_arbitrage_profit(
            100.0, // buy price
            100.1, // sell price (0.1% difference - too small)
            50000.0, // buy liquidity
            50000.0, // sell liquidity
            1000.0, // volume
        );
        
        assert!(no_opportunity.is_none());
    }
}

# 🕸️ Crawl4AI Integration Architecture - SniperBot 2.0

## 🎯 **OVERVIEW**

Crawl4AI integration będzie **rewolucją danych** dla SniperBot 2.0, dostarczając real-time textual intelligence dla AI Decision Engine.

## 📊 **A. ROLA CRAWL4AI W SYSTEMIE**

### **🎯 Cel:**
- **Real-time News Scraping** - Crypto news, market analysis, regulatory updates
- **Social Media Sentiment** - Twitter/X mentions, Reddit discussions, Telegram channels
- **Market Intelligence** - Blog posts, analyst reports, community sentiment
- **Risk Intelligence** - FUD detection, scam alerts, regulatory warnings

### **📋 Format Danych (JSON Output):**
```json
{
  "token_symbol": "SOL",
  "timestamp": 1749160330000,
  "data_type": "news|social|analysis|alert",
  "sources": [
    {
      "url": "https://example.com/article",
      "title": "Solana Network Upgrade Announced",
      "content": "Extracted clean text content...",
      "sentiment_score": 0.75,
      "keywords": ["solana", "upgrade", "performance"],
      "source_type": "news|twitter|reddit|telegram",
      "credibility_score": 0.85,
      "timestamp": 1749160330000
    }
  ],
  "aggregated_sentiment": {
    "overall_score": 0.68,
    "positive_mentions": 15,
    "negative_mentions": 3,
    "neutral_mentions": 8,
    "trending_keywords": ["upgrade", "bullish", "adoption"]
  }
}
```

### **🎯 Typ Danych do Pozyskiwania:**
1. **Crypto News Sites** - CoinDesk, CoinTelegraph, Decrypt, The Block
2. **Social Media** - Twitter/X mentions, Reddit r/solana, Telegram channels
3. **Technical Analysis** - Trading blogs, analyst reports
4. **Regulatory News** - SEC announcements, regulatory updates
5. **Community Sentiment** - Discord discussions, forum posts

## 🐍 **B. CRAWL4AI PYINSTALLER SERVICE ARCHITECTURE**

### **📁 Struktura Projektu:**
```
pyinstaller_scripts/crawl4ai_service/
├── main.py                 # Entry point for PyInstaller executable
├── crawl4ai_scraper.py     # Core scraping logic
├── sentiment_analyzer.py   # Sentiment analysis
├── data_processor.py       # Data cleaning and processing
├── config.py              # Configuration and targets
├── requirements.txt       # Python dependencies
├── build_executable.py    # PyInstaller build script
└── README.md              # Service documentation
```

### **🔧 Kluczowe Biblioteki:**
```python
# Core scraping
crawl4ai>=0.3.0
beautifulsoup4>=4.12.0
lxml>=4.9.0
requests>=2.31.0
httpx>=0.25.0  # For async requests

# Sentiment analysis
textblob>=0.17.0
vaderSentiment>=3.3.0

# Data processing
pandas>=2.0.0
numpy>=1.24.0

# Utilities
python-dateutil>=2.8.0
pytz>=2023.3
```

### **⚡ Interfejs PyInstaller Executable:**

**Input (JSON via stdin):**
```json
{
  "token_symbol": "SOL",
  "token_address": "So11111111111111111111111111111111111111112",
  "data_types": ["news", "social", "analysis"],
  "time_range_hours": 24,
  "max_results": 50,
  "sentiment_analysis": true
}
```

**Output (JSON via stdout):**
```json
{
  "status": "success|error",
  "data": { /* TextualData structure */ },
  "error_message": null,
  "execution_time_ms": 2500,
  "sources_scraped": 15,
  "total_items": 42
}
```

### **🛡️ Mechanizmy Odporności:**
- **Rate Limiting** - Intelligent delays between requests
- **Proxy Rotation** - Multiple proxy sources for resilience
- **User-Agent Rotation** - Avoid detection
- **Retry Logic** - Exponential backoff for failed requests
- **Fallback Sources** - Alternative sources if primary fails
- **Error Handling** - Graceful degradation on failures

## 🦀 **C. INTEGRACJA Z RUST (ANALYSIS AGGREGATOR)**

### **📁 Lokalizacja:**
`crates/sniperbot_core/src/analysis_aggregator/textual_data_fetcher.rs`

### **🔧 Nowa Funkcjonalność:**
```rust
// New struct for textual data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextualData {
    pub token_symbol: String,
    pub timestamp: i64,
    pub data_type: String,
    pub sources: Vec<TextualSource>,
    pub aggregated_sentiment: SentimentSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextualSource {
    pub url: String,
    pub title: String,
    pub content: String,
    pub sentiment_score: f64,
    pub keywords: Vec<String>,
    pub source_type: String,
    pub credibility_score: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSummary {
    pub overall_score: f64,
    pub positive_mentions: u32,
    pub negative_mentions: u32,
    pub neutral_mentions: u32,
    pub trending_keywords: Vec<String>,
}

// New method in AnalysisAggregator
impl AnalysisAggregator {
    pub async fn fetch_textual_data(&self, token_info: &TokenInfo) -> Result<TextualData, TradingError> {
        // Call crawl4ai_service executable
        // Process JSON response
        // Return structured data
    }
}
```

### **🔄 Aktualizacja AggregatedAnalytics:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedAnalytics {
    // Existing fields...
    pub market_data: MarketData,
    pub technical_indicators: TechnicalIndicators,
    
    // NEW: Textual intelligence
    pub textual_data: Option<TextualData>,
    pub sentiment_score: Option<f64>,
    pub news_impact_score: Option<f64>,
}
```

## 🧠 **D. INTEGRACJA Z AI DECISION ENGINE**

### **🎨 Enhanced Prompt Engineering:**
```rust
fn create_enhanced_prompt_with_context(
    token_info: &TokenInfo,
    analytics: &AggregatedAnalytics,
    signal: &StrategySignal
) -> String {
    let mut prompt = format!(
        "Analyze trading opportunity for {} ({})\n\n",
        token_info.symbol, token_info.name
    );
    
    // Market data context
    prompt.push_str(&format!(
        "MARKET DATA:\n- Price: ${:.6}\n- Volume 24h: ${:.2}\n- Market Cap: ${:.2}\n\n",
        token_info.price, analytics.market_data.volume_24h, analytics.market_data.market_cap
    ));
    
    // NEW: Textual intelligence context
    if let Some(textual_data) = &analytics.textual_data {
        prompt.push_str(&format!(
            "NEWS & SENTIMENT ANALYSIS:\n- Overall Sentiment: {:.2}\n- Positive Mentions: {}\n- Negative Mentions: {}\n",
            textual_data.aggregated_sentiment.overall_score,
            textual_data.aggregated_sentiment.positive_mentions,
            textual_data.aggregated_sentiment.negative_mentions
        ));
        
        prompt.push_str("- Trending Keywords: ");
        prompt.push_str(&textual_data.aggregated_sentiment.trending_keywords.join(", "));
        prompt.push_str("\n\n");
        
        // Add recent news headlines
        prompt.push_str("RECENT NEWS HEADLINES:\n");
        for source in textual_data.sources.iter().take(3) {
            prompt.push_str(&format!("- {} (Sentiment: {:.2})\n", source.title, source.sentiment_score));
        }
        prompt.push_str("\n");
    }
    
    // Strategy signal context
    prompt.push_str(&format!(
        "STRATEGY SIGNAL:\n- Strategy: {}\n- Action: {}\n- Strength: {:.2}\n- Rationale: {}\n\n",
        signal.strategy_name, signal.signal_type, signal.strength, signal.rationale
    ));
    
    // Enhanced AI instructions
    prompt.push_str(
        "ANALYSIS INSTRUCTIONS:\n\
        1. Consider market data, news sentiment, and strategy signal\n\
        2. Assess risk based on sentiment analysis and news impact\n\
        3. Factor in trending keywords and community sentiment\n\
        4. Provide risk_score (0.0-1.0) considering textual intelligence\n\
        5. If negative news or FUD detected, increase risk_score\n\
        6. If positive developments confirmed, consider lower risk_score\n\n"
    );
    
    prompt.push_str(
        "Respond with JSON only:\n\
        {\n\
          \"action\": \"BUY|SELL|HOLD|REJECT\",\n\
          \"confidence\": 0.85,\n\
          \"rationale\": \"Clear explanation including sentiment analysis\",\n\
          \"risk_score\": 0.65,\n\
          \"target_price\": 0.001234,\n\
          \"stop_loss_price\": 0.001000,\n\
          \"strategy_parameters\": {\"position_size\": \"0.1\"}\n\
        }"
    );
    
    prompt
}
```

## 🚀 **E. KOLEJNOŚĆ WDROŻENIA**

### **Phase 2.1: Python Service Foundation**
1. ✅ Zaprojektuj crawl4ai_service structure
2. 🔄 Implement basic scraping logic
3. 🔄 Add sentiment analysis
4. 🔄 Build PyInstaller executable
5. 🔄 Test standalone functionality

### **Phase 2.2: Rust Integration**
1. 🔄 Create TextualDataFetcher module
2. 🔄 Update AggregatedAnalytics structure
3. 🔄 Integrate with AnalysisAggregator
4. 🔄 Test Rust ↔ Python communication

### **Phase 2.3: AI Enhancement**
1. 🔄 Enhanced prompt engineering
2. 🔄 Context-aware AI decisions
3. 🔄 Sentiment-based risk scoring
4. 🔄 End-to-end testing

### **Phase 2.4: Production Optimization**
1. 🔄 Performance optimization
2. 🔄 Error handling refinement
3. 🔄 Monitoring and logging
4. 🔄 Documentation completion

## 🎯 **SUCCESS METRICS**

- **Data Quality**: >80% successful scraping rate
- **Sentiment Accuracy**: >75% correlation with market movements
- **Performance**: <5s total data collection time
- **AI Enhancement**: >10% improvement in trading signal accuracy
- **Reliability**: <1% system failures due to textual data issues

---

**NEXT STEP: Implement Phase 2.1 - Python Service Foundation** 🚀

# 🕸️ Crawl4AI Service - SniperBot 2.0

Real-time textual intelligence service for AI-enhanced trading decisions.

## 🎯 **Overview**

Crawl4AI Service is a standalone PyInstaller executable that provides real-time web scraping and sentiment analysis for cryptocurrency trading intelligence. It's designed to integrate seamlessly with the SniperBot 2.0 Rust core.

## 🚀 **Features**

- **🌐 Multi-Source Scraping**: News sites, social media, market analysis
- **🧠 Advanced Sentiment Analysis**: VADER + TextBlob + Crypto-specific patterns
- **⚡ High Performance**: Async scraping with rate limiting
- **🛡️ Robust Error Handling**: Retries, fallbacks, graceful degradation
- **📦 Standalone Executable**: No Python runtime required in production
- **🔧 LLM-Friendly Output**: Clean, structured JSON for AI consumption

## 📁 **Project Structure**

```
pyinstaller_scripts/crawl4ai_service/
├── main.py                 # Entry point for PyInstaller executable
├── crawl4ai_scraper.py     # Core scraping logic
├── sentiment_analyzer.py   # Sentiment analysis engine
├── data_processor.py       # Data cleaning and processing
├── config.py              # Configuration and targets
├── requirements.txt       # Python dependencies
├── build_executable.py    # PyInstaller build script
└── README.md              # This file
```

## 🔧 **Installation & Build**

### Prerequisites
- Python 3.8+
- pip package manager

### Build Process

1. **Install Dependencies**:
   ```bash
   pip install -r requirements.txt
   ```

2. **Build Executable**:
   ```bash
   python build_executable.py
   ```

3. **Test Executable**:
   ```bash
   ./dist/crawl4ai_service
   ```

## 📊 **Usage**

### Input Format (JSON via stdin)
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

### Output Format (JSON via stdout)
```json
{
  "status": "success",
  "data": {
    "token_symbol": "SOL",
    "timestamp": 1749160330000,
    "data_type": "news|social",
    "sources": [
      {
        "url": "https://example.com/article",
        "title": "Solana Network Upgrade Announced",
        "content": "Clean extracted content...",
        "sentiment_score": 0.75,
        "keywords": ["solana", "upgrade", "performance"],
        "source_type": "news",
        "source_name": "CoinDesk",
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
  },
  "error_message": null,
  "execution_time_ms": 2500,
  "sources_scraped": 15,
  "total_items": 42
}
```

## 🎯 **Data Sources**

### 📰 News Sources
- CoinDesk
- CoinTelegraph  
- Decrypt
- The Block
- CryptoSlate

### 🐦 Social Media
- Reddit (r/cryptocurrency, r/solana)
- Twitter/X (via API when available)
- Telegram channels (when accessible)

### 📈 Market Analysis
- CryptoCompare
- Messari
- CoinGecko Blog

## 🧠 **Sentiment Analysis**

### Engines Used
1. **VADER Sentiment**: Enhanced with crypto-specific lexicon
2. **TextBlob**: Polarity and subjectivity analysis
3. **Crypto Patterns**: Regex-based crypto-specific sentiment

### Crypto-Specific Terms
- **Positive**: moon, bullish, pump, rally, adoption, partnership
- **Negative**: rug pull, scam, dump, crash, FUD, hack

### Sentiment Score
- **0.0 - 0.4**: Negative sentiment
- **0.4 - 0.6**: Neutral sentiment  
- **0.6 - 1.0**: Positive sentiment

## ⚙️ **Configuration**

### Rate Limiting
- Global rate limit: 1.0 seconds between requests
- Per-domain rate limits: 1.5-4.0 seconds
- Max concurrent requests: 5
- Exponential backoff on failures

### Quality Filters
- Minimum content length: 50 characters
- Title length: 10-200 characters
- Deduplication by URL and content hash
- Credibility scoring by source type

## 🔗 **Integration with Rust**

### Rust Integration Example
```rust
use tokio::process::Command;
use serde_json;

async fn fetch_textual_data(token_symbol: &str) -> Result<TextualData, TradingError> {
    let input = serde_json::json!({
        "token_symbol": token_symbol,
        "data_types": ["news", "social"],
        "time_range_hours": 24,
        "max_results": 30,
        "sentiment_analysis": true
    });
    
    let output = Command::new("./crawl4ai_service")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .stdin.as_mut().unwrap()
        .write_all(input.to_string().as_bytes())?;
    
    let result = output.wait_with_output().await?;
    let response: CrawlResponse = serde_json::from_slice(&result.stdout)?;
    
    Ok(response.data)
}
```

## 🛡️ **Error Handling**

### Resilience Features
- **Retry Logic**: Exponential backoff for failed requests
- **Fallback Sources**: Alternative sources if primary fails
- **Graceful Degradation**: Partial results on some failures
- **Rate Limit Respect**: Intelligent delays to avoid blocking
- **Timeout Handling**: Configurable timeouts per source

### Error Response
```json
{
  "status": "error",
  "data": null,
  "error_message": "Detailed error description",
  "execution_time_ms": 1500,
  "sources_scraped": 0,
  "total_items": 0
}
```

## 📈 **Performance**

### Benchmarks
- **Typical execution time**: 2-5 seconds
- **Max concurrent requests**: 5
- **Memory usage**: ~50-100 MB
- **Executable size**: ~15-25 MB

### Optimization
- Async HTTP requests
- Connection pooling
- Response streaming
- Minimal dependencies
- Optimized PyInstaller build

## 🧪 **Testing**

### Manual Testing
```bash
echo '{"token_symbol":"SOL","data_types":["news"],"time_range_hours":1,"max_results":5,"sentiment_analysis":true}' | ./dist/crawl4ai_service
```

### Integration Testing
```bash
python -c "
import json, subprocess
input_data = {'token_symbol': 'SOL', 'data_types': ['news'], 'time_range_hours': 1, 'max_results': 5, 'sentiment_analysis': True}
result = subprocess.run(['./dist/crawl4ai_service'], input=json.dumps(input_data), capture_output=True, text=True)
print(result.stdout)
"
```

## 🚀 **Deployment**

### Production Deployment
1. Build executable on target platform
2. Copy executable to production server
3. Set up monitoring and logging
4. Configure rate limits and proxies
5. Test with real trading scenarios

### Docker Deployment
```dockerfile
FROM python:3.11-slim
COPY dist/crawl4ai_service /usr/local/bin/
RUN chmod +x /usr/local/bin/crawl4ai_service
ENTRYPOINT ["/usr/local/bin/crawl4ai_service"]
```

## 📝 **Logging**

Logs are written to stderr with structured format:
```
2024-01-15 10:30:45 - crawl4ai_scraper - INFO - 📰 Scraping crypto news for SOL
2024-01-15 10:30:47 - sentiment_analyzer - INFO - 🧠 Analyzing sentiment for 15 sources
2024-01-15 10:30:48 - data_processor - INFO - ✅ Processed 12 clean sources
```

## 🔮 **Future Enhancements**

- **Browser Automation**: Playwright/Selenium for JS-heavy sites
- **Proxy Rotation**: Enhanced anti-detection measures
- **ML Sentiment**: Custom trained models for crypto sentiment
- **Real-time Streaming**: WebSocket connections for live data
- **Advanced NLP**: Named entity recognition, topic modeling
- **Multi-language**: Support for non-English sources

---

**Built with ❤️ for SniperBot 2.0 - The Future of AI-Enhanced Trading**

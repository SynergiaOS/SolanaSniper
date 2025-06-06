#!/usr/bin/env python3
"""
🕸️ Crawl4AI Service - SniperBot 2.0
Real-time textual intelligence for AI-enhanced trading decisions

Entry point for PyInstaller executable
"""

import sys
import json
import asyncio
import logging
import time
from typing import Dict, Any, Optional
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stderr)  # Log to stderr, data to stdout
    ]
)

logger = logging.getLogger(__name__)

try:
    from crawl4ai_scraper import Crawl4AIScraper
    from sentiment_analyzer import SentimentAnalyzer
    from data_processor import DataProcessor
    from config import ScrapingConfig
except ImportError as e:
    logger.error(f"Failed to import required modules: {e}")
    sys.exit(1)


class Crawl4AIService:
    """Main service class for textual data collection"""
    
    def __init__(self):
        self.scraper = Crawl4AIScraper()
        self.sentiment_analyzer = SentimentAnalyzer()
        self.data_processor = DataProcessor()
        self.config = ScrapingConfig()
        
    async def process_request(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Process a single scraping request"""
        start_time = time.time()
        
        try:
            # Validate input
            token_symbol = request_data.get('token_symbol', '').upper()
            if not token_symbol:
                raise ValueError("token_symbol is required")
            
            token_address = request_data.get('token_address', '')
            data_types = request_data.get('data_types', ['news', 'social'])
            time_range_hours = request_data.get('time_range_hours', 24)
            max_results = request_data.get('max_results', 50)
            sentiment_analysis = request_data.get('sentiment_analysis', True)
            
            logger.info(f"🕸️ Processing request for {token_symbol}")
            logger.info(f"📊 Data types: {data_types}, Time range: {time_range_hours}h")
            
            # Collect raw data
            raw_sources = []
            
            if 'news' in data_types:
                logger.info("📰 Scraping crypto news...")
                news_data = await self.scraper.scrape_crypto_news(
                    token_symbol, time_range_hours, max_results // len(data_types)
                )
                raw_sources.extend(news_data)
            
            if 'social' in data_types:
                logger.info("🐦 Scraping social media...")
                social_data = await self.scraper.scrape_social_media(
                    token_symbol, time_range_hours, max_results // len(data_types)
                )
                raw_sources.extend(social_data)
            
            if 'analysis' in data_types:
                logger.info("📈 Scraping market analysis...")
                analysis_data = await self.scraper.scrape_market_analysis(
                    token_symbol, time_range_hours, max_results // len(data_types)
                )
                raw_sources.extend(analysis_data)
            
            logger.info(f"📊 Collected {len(raw_sources)} raw sources")
            
            # Process and clean data
            processed_sources = self.data_processor.process_sources(raw_sources)
            logger.info(f"✅ Processed {len(processed_sources)} sources")
            
            # Perform sentiment analysis
            if sentiment_analysis and processed_sources:
                logger.info("🧠 Analyzing sentiment...")
                for source in processed_sources:
                    source['sentiment_score'] = self.sentiment_analyzer.analyze_text(
                        f"{source['title']} {source['content']}"
                    )
                
                # Calculate aggregated sentiment
                aggregated_sentiment = self.sentiment_analyzer.aggregate_sentiment(
                    processed_sources
                )
            else:
                aggregated_sentiment = {
                    "overall_score": 0.5,
                    "positive_mentions": 0,
                    "negative_mentions": 0,
                    "neutral_mentions": len(processed_sources),
                    "trending_keywords": []
                }
            
            # Build response
            execution_time = int((time.time() - start_time) * 1000)
            
            response = {
                "status": "success",
                "data": {
                    "token_symbol": token_symbol,
                    "timestamp": int(time.time() * 1000),
                    "data_type": "|".join(data_types),
                    "sources": processed_sources,
                    "aggregated_sentiment": aggregated_sentiment
                },
                "error_message": None,
                "execution_time_ms": execution_time,
                "sources_scraped": len(raw_sources),
                "total_items": len(processed_sources)
            }
            
            logger.info(f"✅ Request completed in {execution_time}ms")
            return response
            
        except Exception as e:
            execution_time = int((time.time() - start_time) * 1000)
            logger.error(f"❌ Error processing request: {e}")
            
            return {
                "status": "error",
                "data": None,
                "error_message": str(e),
                "execution_time_ms": execution_time,
                "sources_scraped": 0,
                "total_items": 0
            }


async def main():
    """Main entry point"""
    logger.info("🚀 Crawl4AI Service starting...")
    
    try:
        # Read input from stdin
        input_data = sys.stdin.read().strip()
        if not input_data:
            raise ValueError("No input data provided")
        
        # Parse JSON input
        try:
            request_data = json.loads(input_data)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON input: {e}")
        
        # Initialize service
        service = Crawl4AIService()
        
        # Process request
        response = await service.process_request(request_data)
        
        # Output JSON response to stdout
        print(json.dumps(response, ensure_ascii=False, indent=None))
        
        # Exit with appropriate code
        sys.exit(0 if response["status"] == "success" else 1)
        
    except Exception as e:
        logger.error(f"💥 Fatal error: {e}")
        
        # Output error response
        error_response = {
            "status": "error",
            "data": None,
            "error_message": str(e),
            "execution_time_ms": 0,
            "sources_scraped": 0,
            "total_items": 0
        }
        
        print(json.dumps(error_response, ensure_ascii=False, indent=None))
        sys.exit(1)


if __name__ == "__main__":
    # Run async main
    asyncio.run(main())

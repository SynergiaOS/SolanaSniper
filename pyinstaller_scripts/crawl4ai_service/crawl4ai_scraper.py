"""
🕸️ Crawl4AI Scraper - Core scraping logic
LLM-friendly web scraping for crypto intelligence
"""

import asyncio
import logging
import time
import random
from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta

import aiohttp
import requests
from bs4 import BeautifulSoup
from urllib.parse import urljoin, urlparse
import re

from config import ScrapingConfig, ScrapingTarget

logger = logging.getLogger(__name__)


class Crawl4AIScraper:
    """Main scraper class using Crawl4AI principles"""
    
    def __init__(self):
        self.config = ScrapingConfig()
        self.session = None
        self.last_request_time = {}  # Per-domain rate limiting
        
    async def __aenter__(self):
        """Async context manager entry"""
        self.session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=self.config.request_timeout),
            connector=aiohttp.TCPConnector(limit=self.config.max_concurrent_requests)
        )
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self.session:
            await self.session.close()
    
    async def scrape_crypto_news(self, token_symbol: str, time_range_hours: int, max_results: int) -> List[Dict[str, Any]]:
        """Scrape crypto news for a specific token"""
        logger.info(f"📰 Scraping crypto news for {token_symbol}")
        
        all_sources = []
        targets = self.config.get_targets_for_type('news')
        
        for target in targets[:3]:  # Limit to top 3 news sources for performance
            try:
                sources = await self._scrape_target(target, token_symbol, max_results // len(targets))
                all_sources.extend(sources)
                logger.info(f"✅ {target.name}: {len(sources)} articles")
            except Exception as e:
                logger.warning(f"⚠️ Failed to scrape {target.name}: {e}")
                continue
        
        # Filter by time range
        cutoff_time = datetime.now() - timedelta(hours=time_range_hours)
        filtered_sources = [
            source for source in all_sources 
            if source.get('timestamp', 0) > cutoff_time.timestamp() * 1000
        ]
        
        logger.info(f"📰 Total news articles: {len(filtered_sources)}")
        return filtered_sources[:max_results]
    
    async def scrape_social_media(self, token_symbol: str, time_range_hours: int, max_results: int) -> List[Dict[str, Any]]:
        """Scrape social media mentions for a specific token"""
        logger.info(f"🐦 Scraping social media for {token_symbol}")
        
        all_sources = []
        targets = self.config.get_targets_for_type('social')
        
        for target in targets:
            try:
                sources = await self._scrape_target(target, token_symbol, max_results // len(targets))
                all_sources.extend(sources)
                logger.info(f"✅ {target.name}: {len(sources)} posts")
            except Exception as e:
                logger.warning(f"⚠️ Failed to scrape {target.name}: {e}")
                continue
        
        # Filter by time range
        cutoff_time = datetime.now() - timedelta(hours=time_range_hours)
        filtered_sources = [
            source for source in all_sources 
            if source.get('timestamp', 0) > cutoff_time.timestamp() * 1000
        ]
        
        logger.info(f"🐦 Total social posts: {len(filtered_sources)}")
        return filtered_sources[:max_results]
    
    async def scrape_market_analysis(self, token_symbol: str, time_range_hours: int, max_results: int) -> List[Dict[str, Any]]:
        """Scrape market analysis for a specific token"""
        logger.info(f"📈 Scraping market analysis for {token_symbol}")
        
        all_sources = []
        targets = self.config.get_targets_for_type('analysis')
        
        for target in targets:
            try:
                sources = await self._scrape_target(target, token_symbol, max_results // len(targets))
                all_sources.extend(sources)
                logger.info(f"✅ {target.name}: {len(sources)} analyses")
            except Exception as e:
                logger.warning(f"⚠️ Failed to scrape {target.name}: {e}")
                continue
        
        # Filter by time range
        cutoff_time = datetime.now() - timedelta(hours=time_range_hours)
        filtered_sources = [
            source for source in all_sources 
            if source.get('timestamp', 0) > cutoff_time.timestamp() * 1000
        ]
        
        logger.info(f"📈 Total analysis pieces: {len(filtered_sources)}")
        return filtered_sources[:max_results]
    
    async def _scrape_target(self, target: ScrapingTarget, token_symbol: str, max_results: int) -> List[Dict[str, Any]]:
        """Scrape a specific target for token-related content"""
        
        # Rate limiting per domain
        domain = urlparse(target.base_url).netloc
        await self._enforce_rate_limit(domain, target.rate_limit_seconds)
        
        # Build search URL
        search_url = self.config.get_search_url(target, token_symbol)
        headers = self.config.get_headers(target)
        
        logger.debug(f"🔍 Scraping {target.name}: {search_url}")
        
        # Fetch page content
        content = await self._fetch_page(search_url, headers, target)
        if not content:
            return []
        
        # Parse content based on target type
        if target.name in ['CoinDesk', 'CoinTelegraph', 'Decrypt', 'The Block', 'CryptoSlate']:
            return self._parse_news_site(content, target, token_symbol)
        elif 'Reddit' in target.name:
            return self._parse_reddit(content, target, token_symbol)
        elif target.name in ['CryptoCompare', 'Messari', 'CoinGecko Blog']:
            return self._parse_analysis_site(content, target, token_symbol)
        else:
            return self._parse_generic(content, target, token_symbol)
    
    async def _fetch_page(self, url: str, headers: Dict[str, str], target: ScrapingTarget) -> Optional[str]:
        """Fetch page content with retries and error handling"""
        
        for attempt in range(target.max_retries):
            try:
                if self.session:
                    # Use aiohttp session
                    async with self.session.get(url, headers=headers, timeout=target.timeout_seconds) as response:
                        if response.status == 200:
                            return await response.text()
                        else:
                            logger.warning(f"HTTP {response.status} for {url}")
                else:
                    # Fallback to requests
                    response = requests.get(url, headers=headers, timeout=target.timeout_seconds)
                    if response.status_code == 200:
                        return response.text
                    else:
                        logger.warning(f"HTTP {response.status_code} for {url}")
                        
            except Exception as e:
                logger.warning(f"Attempt {attempt + 1} failed for {url}: {e}")
                if attempt < target.max_retries - 1:
                    await asyncio.sleep(self.config.backoff_factor ** attempt)
        
        return None
    
    def _parse_news_site(self, content: str, target: ScrapingTarget, token_symbol: str) -> List[Dict[str, Any]]:
        """Parse news site content"""
        soup = BeautifulSoup(content, 'html.parser')
        articles = []
        
        # Generic article selectors (would be customized per site in production)
        article_selectors = [
            'article',
            '.article',
            '.post',
            '.news-item',
            '.story',
            '[data-module="ArticleCard"]'
        ]
        
        for selector in article_selectors:
            elements = soup.select(selector)
            if elements:
                break
        else:
            elements = []
        
        for element in elements[:10]:  # Limit per page
            try:
                # Extract title
                title_elem = element.find(['h1', 'h2', 'h3', 'h4']) or element.find(class_=re.compile(r'title|headline'))
                title = title_elem.get_text(strip=True) if title_elem else ""
                
                # Extract link
                link_elem = element.find('a') or element
                url = link_elem.get('href', '') if link_elem else ""
                if url and not url.startswith('http'):
                    url = urljoin(target.base_url, url)
                
                # Extract content preview
                content_elem = element.find(class_=re.compile(r'summary|excerpt|description|content'))
                content_text = content_elem.get_text(strip=True) if content_elem else ""
                
                # Extract timestamp (simplified)
                time_elem = element.find('time') or element.find(class_=re.compile(r'date|time'))
                timestamp = int(time.time() * 1000)  # Default to now
                
                if title and token_symbol.lower() in title.lower():
                    articles.append({
                        'url': url,
                        'title': title,
                        'content': content_text,
                        'source_type': 'news',
                        'source_name': target.name,
                        'timestamp': timestamp,
                        'credibility_score': 0.8,  # Default high for news sites
                        'keywords': self._extract_keywords(f"{title} {content_text}")
                    })
                    
            except Exception as e:
                logger.debug(f"Error parsing article element: {e}")
                continue
        
        return articles
    
    def _parse_reddit(self, content: str, target: ScrapingTarget, token_symbol: str) -> List[Dict[str, Any]]:
        """Parse Reddit content"""
        soup = BeautifulSoup(content, 'html.parser')
        posts = []
        
        # Reddit post selectors
        post_elements = soup.select('[data-testid="post-container"]') or soup.select('.Post')
        
        for element in post_elements[:5]:  # Limit Reddit posts
            try:
                # Extract title
                title_elem = element.find('h3') or element.find(class_=re.compile(r'title'))
                title = title_elem.get_text(strip=True) if title_elem else ""
                
                # Extract content
                content_elem = element.find(class_=re.compile(r'usertext-body|md'))
                content_text = content_elem.get_text(strip=True) if content_elem else ""
                
                # Extract link
                link_elem = element.find('a')
                url = link_elem.get('href', '') if link_elem else ""
                if url and not url.startswith('http'):
                    url = urljoin(target.base_url, url)
                
                if title and token_symbol.lower() in f"{title} {content_text}".lower():
                    posts.append({
                        'url': url,
                        'title': title,
                        'content': content_text,
                        'source_type': 'social',
                        'source_name': target.name,
                        'timestamp': int(time.time() * 1000),
                        'credibility_score': 0.6,  # Lower for social media
                        'keywords': self._extract_keywords(f"{title} {content_text}")
                    })
                    
            except Exception as e:
                logger.debug(f"Error parsing Reddit post: {e}")
                continue
        
        return posts
    
    def _parse_analysis_site(self, content: str, target: ScrapingTarget, token_symbol: str) -> List[Dict[str, Any]]:
        """Parse market analysis site content"""
        # Similar to news parsing but with analysis-specific selectors
        return self._parse_news_site(content, target, token_symbol)
    
    def _parse_generic(self, content: str, target: ScrapingTarget, token_symbol: str) -> List[Dict[str, Any]]:
        """Generic content parser"""
        soup = BeautifulSoup(content, 'html.parser')
        
        # Remove script and style elements
        for script in soup(["script", "style"]):
            script.decompose()
        
        # Get text content
        text = soup.get_text()
        
        # Simple check if token is mentioned
        if token_symbol.lower() in text.lower():
            return [{
                'url': target.base_url,
                'title': f"{target.name} content mentioning {token_symbol}",
                'content': text[:500],  # First 500 chars
                'source_type': 'generic',
                'source_name': target.name,
                'timestamp': int(time.time() * 1000),
                'credibility_score': 0.5,
                'keywords': self._extract_keywords(text[:500])
            }]
        
        return []
    
    def _extract_keywords(self, text: str) -> List[str]:
        """Extract relevant keywords from text"""
        # Simple keyword extraction (would use NLP in production)
        crypto_keywords = self.config.sentiment_config['crypto_keywords']
        
        text_lower = text.lower()
        found_keywords = [kw for kw in crypto_keywords if kw in text_lower]
        
        return found_keywords[:5]  # Top 5 keywords
    
    async def _enforce_rate_limit(self, domain: str, rate_limit_seconds: float):
        """Enforce rate limiting per domain"""
        now = time.time()
        last_request = self.last_request_time.get(domain, 0)
        
        time_since_last = now - last_request
        if time_since_last < rate_limit_seconds:
            sleep_time = rate_limit_seconds - time_since_last
            logger.debug(f"⏱️ Rate limiting {domain}: sleeping {sleep_time:.2f}s")
            await asyncio.sleep(sleep_time)
        
        self.last_request_time[domain] = time.time()

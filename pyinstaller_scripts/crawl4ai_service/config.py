"""
🔧 Crawl4AI Service Configuration
Scraping targets, rate limits, and service settings
"""

import os
from typing import Dict, List, Any
from dataclasses import dataclass


@dataclass
class ScrapingTarget:
    """Configuration for a single scraping target"""
    name: str
    base_url: str
    search_pattern: str
    rate_limit_seconds: float
    max_retries: int
    timeout_seconds: int
    requires_js: bool = False
    headers: Dict[str, str] = None
    
    def __post_init__(self):
        if self.headers is None:
            self.headers = {
                'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
            }


class ScrapingConfig:
    """Main configuration class for Crawl4AI service"""
    
    def __init__(self):
        self.setup_targets()
        self.setup_rate_limits()
        self.setup_sentiment_config()
    
    def setup_targets(self):
        """Configure scraping targets for different data types"""
        
        # 📰 Crypto News Sources
        self.news_targets = [
            ScrapingTarget(
                name="CoinDesk",
                base_url="https://www.coindesk.com",
                search_pattern="/search?s={token}",
                rate_limit_seconds=2.0,
                max_retries=3,
                timeout_seconds=10
            ),
            ScrapingTarget(
                name="CoinTelegraph",
                base_url="https://cointelegraph.com",
                search_pattern="/search?query={token}",
                rate_limit_seconds=2.5,
                max_retries=3,
                timeout_seconds=10
            ),
            ScrapingTarget(
                name="Decrypt",
                base_url="https://decrypt.co",
                search_pattern="/search?q={token}",
                rate_limit_seconds=1.5,
                max_retries=3,
                timeout_seconds=10
            ),
            ScrapingTarget(
                name="The Block",
                base_url="https://www.theblock.co",
                search_pattern="/search?query={token}",
                rate_limit_seconds=3.0,
                max_retries=3,
                timeout_seconds=15
            ),
            ScrapingTarget(
                name="CryptoSlate",
                base_url="https://cryptoslate.com",
                search_pattern="/search/?q={token}",
                rate_limit_seconds=2.0,
                max_retries=3,
                timeout_seconds=10
            )
        ]
        
        # 🐦 Social Media Sources
        self.social_targets = [
            ScrapingTarget(
                name="Reddit Crypto",
                base_url="https://www.reddit.com",
                search_pattern="/r/cryptocurrency/search/?q={token}&restrict_sr=1&sort=new",
                rate_limit_seconds=3.0,
                max_retries=2,
                timeout_seconds=15,
                requires_js=True
            ),
            ScrapingTarget(
                name="Reddit Solana",
                base_url="https://www.reddit.com",
                search_pattern="/r/solana/search/?q={token}&restrict_sr=1&sort=new",
                rate_limit_seconds=3.0,
                max_retries=2,
                timeout_seconds=15,
                requires_js=True
            ),
            # Note: Twitter/X requires API access, not direct scraping
            # Telegram channels would need specific channel access
        ]
        
        # 📈 Market Analysis Sources
        self.analysis_targets = [
            ScrapingTarget(
                name="CryptoCompare",
                base_url="https://www.cryptocompare.com",
                search_pattern="/coins/{token}/analysis",
                rate_limit_seconds=2.0,
                max_retries=3,
                timeout_seconds=12
            ),
            ScrapingTarget(
                name="Messari",
                base_url="https://messari.io",
                search_pattern="/asset/{token}/research",
                rate_limit_seconds=4.0,
                max_retries=2,
                timeout_seconds=20
            ),
            ScrapingTarget(
                name="CoinGecko Blog",
                base_url="https://blog.coingecko.com",
                search_pattern="/search?q={token}",
                rate_limit_seconds=2.5,
                max_retries=3,
                timeout_seconds=15
            )
        ]
    
    def setup_rate_limits(self):
        """Configure global rate limiting"""
        self.global_rate_limit = 1.0  # Minimum seconds between any requests
        self.max_concurrent_requests = 5
        self.request_timeout = 30
        self.max_retries_global = 3
        
        # Backoff configuration
        self.backoff_factor = 2.0
        self.max_backoff_seconds = 60.0
    
    def setup_sentiment_config(self):
        """Configure sentiment analysis"""
        self.sentiment_config = {
            'use_vader': True,
            'use_textblob': True,
            'crypto_keywords': [
                # Positive
                'bullish', 'moon', 'pump', 'rally', 'breakout', 'adoption',
                'partnership', 'upgrade', 'launch', 'integration', 'growth',
                'institutional', 'mainstream', 'breakthrough', 'innovation',
                
                # Negative  
                'bearish', 'dump', 'crash', 'scam', 'rug', 'hack', 'exploit',
                'regulation', 'ban', 'fud', 'sell-off', 'decline', 'risk',
                'warning', 'investigation', 'lawsuit', 'fraud'
            ],
            'weight_title': 2.0,  # Title sentiment has 2x weight
            'weight_content': 1.0,
            'min_content_length': 50,  # Minimum chars for sentiment analysis
        }
    
    def get_targets_for_type(self, data_type: str) -> List[ScrapingTarget]:
        """Get scraping targets for a specific data type"""
        if data_type == 'news':
            return self.news_targets
        elif data_type == 'social':
            return self.social_targets
        elif data_type == 'analysis':
            return self.analysis_targets
        else:
            return []
    
    def get_search_url(self, target: ScrapingTarget, token_symbol: str) -> str:
        """Build search URL for a target and token"""
        search_path = target.search_pattern.format(token=token_symbol.lower())
        return f"{target.base_url}{search_path}"
    
    @property
    def user_agents(self) -> List[str]:
        """Rotating user agents for scraping"""
        return [
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0',
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0'
        ]
    
    @property
    def proxy_list(self) -> List[str]:
        """Proxy servers for rotation (if needed)"""
        # In production, these would be real proxy servers
        # For now, return empty list (direct connections)
        return []
    
    def get_headers(self, target: ScrapingTarget) -> Dict[str, str]:
        """Get headers for a specific target"""
        import random
        
        headers = target.headers.copy()
        headers['User-Agent'] = random.choice(self.user_agents)
        headers.update({
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Accept-Encoding': 'gzip, deflate, br',
            'DNT': '1',
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1',
        })
        
        return headers


# Global configuration instance
config = ScrapingConfig()

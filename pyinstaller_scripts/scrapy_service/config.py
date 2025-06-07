"""
🔧 Scrapy Service Configuration
Professional web scraping configuration for 10x performance boost
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
                'User-Agent': 'SniperBot-Scrapy/2.0 (+https://github.com/SynergiaOS/SolanaSniper)'
            }


class ScrapingConfig:
    """Professional Scrapy service configuration"""
    
    def __init__(self):
        # 🚀 Performance settings
        self.max_concurrent_requests = 16
        self.concurrent_requests_per_domain = 4
        self.download_delay = 1.0
        self.randomize_download_delay = 0.5
        self.request_timeout = 30
        
        # 🎯 AutoThrottle settings
        self.autothrottle_enabled = True
        self.autothrottle_start_delay = 1
        self.autothrottle_max_delay = 10
        self.autothrottle_target_concurrency = 2.0
        
        # 🔄 Retry settings
        self.retry_enabled = True
        self.retry_times = 3
        self.retry_http_codes = [500, 502, 503, 504, 408, 429]
        
        # 🧠 Sentiment analysis settings
        self.sentiment_config = {
            'min_content_length': 10,
            'enable_vader': True,
            'enable_textblob': False,  # Disabled for PyInstaller
            'crypto_patterns_enabled': True
        }
        
        # 📰 News sources (optimized for Scrapy)
        self.news_targets = [
            ScrapingTarget(
                name="CoinDesk",
                base_url="https://www.coindesk.com",
                search_pattern="/search?s={token}",
                rate_limit_seconds=1.0,
                max_retries=3,
                timeout_seconds=15
            ),
            ScrapingTarget(
                name="CoinTelegraph", 
                base_url="https://cointelegraph.com",
                search_pattern="/search?query={token}",
                rate_limit_seconds=1.0,
                max_retries=3,
                timeout_seconds=15
            ),
            ScrapingTarget(
                name="Decrypt",
                base_url="https://decrypt.co",
                search_pattern="/search?q={token}",
                rate_limit_seconds=1.5,
                max_retries=3,
                timeout_seconds=15
            ),
            ScrapingTarget(
                name="The Block",
                base_url="https://www.theblock.co",
                search_pattern="/search?query={token}",
                rate_limit_seconds=1.5,
                max_retries=3,
                timeout_seconds=15
            ),
            ScrapingTarget(
                name="CryptoSlate",
                base_url="https://cryptoslate.com",
                search_pattern="/search/?q={token}",
                rate_limit_seconds=2.0,
                max_retries=2,
                timeout_seconds=20
            )
        ]
        
        # 📱 Social media sources
        self.social_targets = [
            ScrapingTarget(
                name="Reddit Crypto",
                base_url="https://www.reddit.com",
                search_pattern="/r/cryptocurrency/search/?q={token}",
                rate_limit_seconds=3.0,
                max_retries=2,
                timeout_seconds=20
            ),
            ScrapingTarget(
                name="Reddit CryptoMoonShots",
                base_url="https://www.reddit.com",
                search_pattern="/r/CryptoMoonShots/search/?q={token}",
                rate_limit_seconds=3.0,
                max_retries=2,
                timeout_seconds=20
            )
        ]
        
        # 📈 Analysis sources
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
            )
        ]
    
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
        search_path = target.search_pattern.format(token=token_symbol)
        return f"{target.base_url}{search_path}"
    
    def get_headers(self, target: ScrapingTarget) -> Dict[str, str]:
        """Get headers for a target"""
        return target.headers.copy()
    
    def get_scrapy_settings(self) -> Dict[str, Any]:
        """Get Scrapy-specific settings"""
        return {
            'CONCURRENT_REQUESTS': self.max_concurrent_requests,
            'CONCURRENT_REQUESTS_PER_DOMAIN': self.concurrent_requests_per_domain,
            'DOWNLOAD_DELAY': self.download_delay,
            'RANDOMIZE_DOWNLOAD_DELAY': self.randomize_download_delay,
            'DOWNLOAD_TIMEOUT': self.request_timeout,
            
            # AutoThrottle
            'AUTOTHROTTLE_ENABLED': self.autothrottle_enabled,
            'AUTOTHROTTLE_START_DELAY': self.autothrottle_start_delay,
            'AUTOTHROTTLE_MAX_DELAY': self.autothrottle_max_delay,
            'AUTOTHROTTLE_TARGET_CONCURRENCY': self.autothrottle_target_concurrency,
            'AUTOTHROTTLE_DEBUG': False,
            
            # Retry
            'RETRY_ENABLED': self.retry_enabled,
            'RETRY_TIMES': self.retry_times,
            'RETRY_HTTP_CODES': self.retry_http_codes,
            
            # Other settings
            'USER_AGENT': 'SniperBot-Scrapy/2.0 (+https://github.com/SynergiaOS/SolanaSniper)',
            'ROBOTSTXT_OBEY': True,
            'COOKIES_ENABLED': False,
            'TELNETCONSOLE_ENABLED': False,
            'LOG_LEVEL': 'WARNING'
        }

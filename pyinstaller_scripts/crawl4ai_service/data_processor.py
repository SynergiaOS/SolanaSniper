"""
🔧 Data Processor - Clean and structure scraped data
LLM-friendly data processing and cleaning
"""

import logging
import re
import html
from typing import List, Dict, Any, Optional
from urllib.parse import urlparse
import hashlib

logger = logging.getLogger(__name__)


class DataProcessor:
    """Process and clean scraped data for AI consumption"""
    
    def __init__(self):
        self.url_cache = set()  # Deduplicate URLs
        self.content_hashes = set()  # Deduplicate content
        
    def process_sources(self, raw_sources: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Process and clean list of raw sources"""
        if not raw_sources:
            return []
        
        logger.info(f"🔧 Processing {len(raw_sources)} raw sources")
        
        processed = []
        for source in raw_sources:
            try:
                cleaned_source = self._process_single_source(source)
                if cleaned_source and self._is_valid_source(cleaned_source):
                    processed.append(cleaned_source)
            except Exception as e:
                logger.debug(f"Error processing source: {e}")
                continue
        
        # Deduplicate
        deduplicated = self._deduplicate_sources(processed)
        
        # Sort by timestamp (newest first)
        sorted_sources = sorted(
            deduplicated, 
            key=lambda x: x.get('timestamp', 0), 
            reverse=True
        )
        
        logger.info(f"✅ Processed {len(sorted_sources)} clean sources")
        return sorted_sources
    
    def _process_single_source(self, source: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Process a single source"""
        
        # Clean and validate URL
        url = self._clean_url(source.get('url', ''))
        if not url:
            return None
        
        # Clean title
        title = self._clean_text(source.get('title', ''))
        if not title:
            return None
        
        # Clean content
        content = self._clean_text(source.get('content', ''))
        
        # Extract and clean keywords
        keywords = self._clean_keywords(source.get('keywords', []))
        
        # Validate and normalize other fields
        source_type = source.get('source_type', 'unknown')
        source_name = source.get('source_name', 'Unknown')
        timestamp = int(source.get('timestamp', 0))
        credibility_score = float(source.get('credibility_score', 0.5))
        
        # Ensure credibility score is in valid range
        credibility_score = max(0.0, min(1.0, credibility_score))
        
        return {
            'url': url,
            'title': title,
            'content': content,
            'source_type': source_type,
            'source_name': source_name,
            'timestamp': timestamp,
            'credibility_score': credibility_score,
            'keywords': keywords,
            'content_length': len(content),
            'content_hash': self._generate_content_hash(f"{title} {content}")
        }
    
    def _clean_url(self, url: str) -> str:
        """Clean and validate URL"""
        if not url:
            return ""
        
        # Remove whitespace
        url = url.strip()
        
        # Basic URL validation
        try:
            parsed = urlparse(url)
            if not parsed.scheme or not parsed.netloc:
                return ""
        except Exception:
            return ""
        
        # Remove tracking parameters
        tracking_params = [
            'utm_source', 'utm_medium', 'utm_campaign', 'utm_content', 'utm_term',
            'fbclid', 'gclid', 'ref', 'source', 'campaign'
        ]
        
        # Simple parameter removal (would use urllib.parse.parse_qs in production)
        for param in tracking_params:
            if f'{param}=' in url:
                url = re.sub(f'[?&]{param}=[^&]*', '', url)
        
        # Clean up URL
        url = re.sub(r'[?&]$', '', url)  # Remove trailing ? or &
        
        return url
    
    def _clean_text(self, text: str) -> str:
        """Clean and normalize text content"""
        if not text:
            return ""
        
        # Decode HTML entities
        text = html.unescape(text)
        
        # Remove extra whitespace
        text = re.sub(r'\s+', ' ', text)
        text = text.strip()
        
        # Remove common unwanted patterns
        unwanted_patterns = [
            r'Read more\.\.\.?',
            r'Continue reading\.\.\.?',
            r'Click here\.\.\.?',
            r'Subscribe to.*',
            r'Follow us on.*',
            r'Share this.*',
            r'Advertisement',
            r'Sponsored content',
            r'\[.*?\]',  # Remove [brackets] content
            r'Cookie policy.*',
            r'Privacy policy.*',
        ]
        
        for pattern in unwanted_patterns:
            text = re.sub(pattern, '', text, flags=re.IGNORECASE)
        
        # Remove excessive punctuation
        text = re.sub(r'[.]{3,}', '...', text)
        text = re.sub(r'[!]{2,}', '!', text)
        text = re.sub(r'[?]{2,}', '?', text)
        
        # Remove URLs from content (keep them in url field)
        text = re.sub(r'https?://[^\s]+', '', text)
        
        # Final cleanup
        text = re.sub(r'\s+', ' ', text).strip()
        
        return text
    
    def _clean_keywords(self, keywords: List[str]) -> List[str]:
        """Clean and normalize keywords"""
        if not keywords:
            return []
        
        cleaned = []
        for keyword in keywords:
            if isinstance(keyword, str):
                # Clean keyword
                clean_kw = keyword.strip().lower()
                clean_kw = re.sub(r'[^\w\s-]', '', clean_kw)
                clean_kw = re.sub(r'\s+', ' ', clean_kw).strip()
                
                # Filter out very short or long keywords
                if 2 <= len(clean_kw) <= 30 and clean_kw not in cleaned:
                    cleaned.append(clean_kw)
        
        return cleaned[:10]  # Limit to 10 keywords
    
    def _is_valid_source(self, source: Dict[str, Any]) -> bool:
        """Validate if source meets quality criteria"""
        
        # Must have title
        if not source.get('title'):
            return False
        
        # Title must be reasonable length
        title_len = len(source['title'])
        if title_len < 10 or title_len > 200:
            return False
        
        # Must have valid URL
        if not source.get('url'):
            return False
        
        # Content should be reasonable length (if present)
        content_len = len(source.get('content', ''))
        if content_len > 0 and content_len < 20:
            return False
        
        # Must have valid timestamp
        if source.get('timestamp', 0) <= 0:
            return False
        
        return True
    
    def _deduplicate_sources(self, sources: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Remove duplicate sources based on URL and content"""
        seen_urls = set()
        seen_hashes = set()
        deduplicated = []
        
        for source in sources:
            url = source.get('url', '')
            content_hash = source.get('content_hash', '')
            
            # Skip if URL already seen
            if url in seen_urls:
                continue
            
            # Skip if content hash already seen (duplicate content)
            if content_hash in seen_hashes:
                continue
            
            seen_urls.add(url)
            seen_hashes.add(content_hash)
            deduplicated.append(source)
        
        return deduplicated
    
    def _generate_content_hash(self, content: str) -> str:
        """Generate hash for content deduplication"""
        if not content:
            return ""
        
        # Normalize content for hashing
        normalized = re.sub(r'\s+', ' ', content.lower().strip())
        
        # Generate SHA-256 hash
        return hashlib.sha256(normalized.encode('utf-8')).hexdigest()[:16]
    
    def get_processing_stats(self, sources: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Get statistics about processed sources"""
        if not sources:
            return {
                'total_sources': 0,
                'avg_content_length': 0,
                'source_types': {},
                'credibility_distribution': {}
            }
        
        # Calculate stats
        total_sources = len(sources)
        total_content_length = sum(s.get('content_length', 0) for s in sources)
        avg_content_length = total_content_length / total_sources if total_sources > 0 else 0
        
        # Source type distribution
        source_types = {}
        for source in sources:
            source_type = source.get('source_type', 'unknown')
            source_types[source_type] = source_types.get(source_type, 0) + 1
        
        # Credibility distribution
        credibility_ranges = {
            'high (0.8-1.0)': 0,
            'medium (0.5-0.8)': 0,
            'low (0.0-0.5)': 0
        }
        
        for source in sources:
            credibility = source.get('credibility_score', 0.5)
            if credibility >= 0.8:
                credibility_ranges['high (0.8-1.0)'] += 1
            elif credibility >= 0.5:
                credibility_ranges['medium (0.5-0.8)'] += 1
            else:
                credibility_ranges['low (0.0-0.5)'] += 1
        
        return {
            'total_sources': total_sources,
            'avg_content_length': round(avg_content_length, 1),
            'source_types': source_types,
            'credibility_distribution': credibility_ranges
        }

"""
🧠 Sentiment Analyzer - Crypto-aware sentiment analysis
Advanced sentiment analysis for trading intelligence
"""

import logging
import re
from typing import Dict, List, Any, Tuple
from collections import Counter

try:
    from vaderSentiment.vaderSentiment import SentimentIntensityAnalyzer
    VADER_AVAILABLE = True
except ImportError:
    VADER_AVAILABLE = False
    logging.warning("VADER sentiment analyzer not available")

# Disable TextBlob for PyInstaller compatibility
TEXTBLOB_AVAILABLE = False

from config import ScrapingConfig

logger = logging.getLogger(__name__)


class SentimentAnalyzer:
    """Advanced sentiment analyzer for crypto content"""
    
    def __init__(self):
        self.config = ScrapingConfig()
        
        # Initialize VADER analyzer
        if VADER_AVAILABLE:
            self.vader = SentimentIntensityAnalyzer()
            # Add crypto-specific terms to VADER lexicon
            self._enhance_vader_lexicon()
        else:
            self.vader = None
        
        # Crypto-specific sentiment patterns
        self.crypto_patterns = self._build_crypto_patterns()
        
        logger.info(f"🧠 Sentiment analyzer initialized (VADER: {VADER_AVAILABLE}, TextBlob: {TEXTBLOB_AVAILABLE})")
    
    def analyze_text(self, text: str) -> float:
        """
        Analyze sentiment of text and return score between 0.0 and 1.0
        0.0 = very negative, 0.5 = neutral, 1.0 = very positive
        """
        if not text or len(text.strip()) < self.config.sentiment_config['min_content_length']:
            return 0.5  # Neutral for short/empty text
        
        scores = []
        
        # VADER sentiment
        if self.vader:
            vader_score = self._analyze_with_vader(text)
            scores.append(vader_score)
        
        # TextBlob sentiment
        if TEXTBLOB_AVAILABLE:
            textblob_score = self._analyze_with_textblob(text)
            scores.append(textblob_score)
        
        # Crypto-specific pattern analysis
        crypto_score = self._analyze_crypto_patterns(text)
        scores.append(crypto_score)
        
        # Weighted average
        if scores:
            final_score = sum(scores) / len(scores)
        else:
            final_score = 0.5  # Neutral fallback
        
        # Ensure score is in valid range
        return max(0.0, min(1.0, final_score))
    
    def _analyze_with_vader(self, text: str) -> float:
        """Analyze sentiment using VADER"""
        try:
            scores = self.vader.polarity_scores(text)
            # Convert compound score (-1 to 1) to 0-1 scale
            compound = scores['compound']
            return (compound + 1.0) / 2.0
        except Exception as e:
            logger.debug(f"VADER analysis failed: {e}")
            return 0.5
    
    def _analyze_with_textblob(self, text: str) -> float:
        """Analyze sentiment using TextBlob (disabled for PyInstaller)"""
        # TextBlob disabled for PyInstaller compatibility
        return 0.5
    
    def _analyze_crypto_patterns(self, text: str) -> float:
        """Analyze sentiment using crypto-specific patterns"""
        text_lower = text.lower()
        
        positive_score = 0
        negative_score = 0
        
        # Check positive patterns
        for pattern, weight in self.crypto_patterns['positive'].items():
            matches = len(re.findall(pattern, text_lower))
            positive_score += matches * weight
        
        # Check negative patterns
        for pattern, weight in self.crypto_patterns['negative'].items():
            matches = len(re.findall(pattern, text_lower))
            negative_score += matches * weight
        
        # Calculate final score
        total_score = positive_score - negative_score
        
        # Normalize to 0-1 scale
        if total_score > 0:
            return min(1.0, 0.5 + (total_score * 0.1))
        elif total_score < 0:
            return max(0.0, 0.5 + (total_score * 0.1))
        else:
            return 0.5
    
    def aggregate_sentiment(self, sources: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Aggregate sentiment across multiple sources"""
        if not sources:
            return {
                "overall_score": 0.5,
                "positive_mentions": 0,
                "negative_mentions": 0,
                "neutral_mentions": 0,
                "trending_keywords": []
            }
        
        sentiment_scores = []
        all_keywords = []
        positive_count = 0
        negative_count = 0
        neutral_count = 0
        
        for source in sources:
            score = source.get('sentiment_score', 0.5)
            sentiment_scores.append(score)
            
            # Count sentiment categories
            if score > 0.6:
                positive_count += 1
            elif score < 0.4:
                negative_count += 1
            else:
                neutral_count += 1
            
            # Collect keywords
            keywords = source.get('keywords', [])
            all_keywords.extend(keywords)
        
        # Calculate overall sentiment
        if sentiment_scores:
            # Weight by source credibility if available
            weighted_scores = []
            for i, source in enumerate(sources):
                score = sentiment_scores[i]
                credibility = source.get('credibility_score', 1.0)
                weighted_scores.append(score * credibility)
            
            overall_score = sum(weighted_scores) / len(weighted_scores)
        else:
            overall_score = 0.5
        
        # Find trending keywords
        keyword_counts = Counter(all_keywords)
        trending_keywords = [kw for kw, count in keyword_counts.most_common(10)]
        
        return {
            "overall_score": round(overall_score, 3),
            "positive_mentions": positive_count,
            "negative_mentions": negative_count,
            "neutral_mentions": neutral_count,
            "trending_keywords": trending_keywords
        }
    
    def _enhance_vader_lexicon(self):
        """Add crypto-specific terms to VADER lexicon"""
        if not self.vader:
            return
        
        crypto_lexicon = {
            # Very positive
            'moon': 3.0,
            'lambo': 2.5,
            'hodl': 2.0,
            'diamond hands': 3.0,
            'to the moon': 3.5,
            'bullish': 2.5,
            'pump': 2.0,
            'rally': 2.0,
            'breakout': 2.5,
            'adoption': 2.0,
            'institutional': 1.5,
            'partnership': 2.0,
            'upgrade': 1.5,
            'innovation': 1.5,
            
            # Very negative
            'rug pull': -3.5,
            'scam': -3.0,
            'dump': -2.5,
            'crash': -2.5,
            'bearish': -2.0,
            'fud': -2.0,
            'paper hands': -2.0,
            'rekt': -2.5,
            'liquidated': -2.5,
            'hack': -3.0,
            'exploit': -3.0,
            'ponzi': -3.5,
            'regulation': -1.5,
            'ban': -2.5,
            'investigation': -2.0,
            'lawsuit': -2.0,
        }
        
        # Update VADER lexicon
        self.vader.lexicon.update(crypto_lexicon)
    
    def _build_crypto_patterns(self) -> Dict[str, Dict[str, float]]:
        """Build regex patterns for crypto sentiment analysis"""
        return {
            'positive': {
                r'\b(moon|mooning|moonshot)\b': 2.0,
                r'\b(bull|bullish|bull market)\b': 1.5,
                r'\b(pump|pumping|pumped)\b': 1.5,
                r'\b(rally|rallying)\b': 1.5,
                r'\b(breakout|breakthrough)\b': 1.5,
                r'\b(adoption|mainstream)\b': 1.0,
                r'\b(partnership|collaboration)\b': 1.0,
                r'\b(upgrade|improvement)\b': 1.0,
                r'\b(launch|release)\b': 1.0,
                r'\b(integration|support)\b': 1.0,
                r'\b(institutional|enterprise)\b': 1.0,
                r'\b(growth|expansion)\b': 1.0,
                r'\b(innovation|revolutionary)\b': 1.0,
                r'\b(hodl|hold|diamond hands)\b': 1.5,
                r'🚀|📈|💎|🌙': 1.0,  # Emoji patterns
            },
            'negative': {
                r'\b(rug pull|rug|rugged)\b': 3.0,
                r'\b(scam|fraud|ponzi)\b': 2.5,
                r'\b(dump|dumping|dumped)\b': 2.0,
                r'\b(crash|crashed|crashing)\b': 2.0,
                r'\b(bear|bearish|bear market)\b': 1.5,
                r'\b(fud|fear)\b': 1.5,
                r'\b(hack|hacked|exploit)\b': 2.5,
                r'\b(liquidated|rekt|liquidation)\b': 2.0,
                r'\b(regulation|regulatory|ban)\b': 1.5,
                r'\b(investigation|lawsuit|legal)\b': 1.5,
                r'\b(warning|risk|danger)\b': 1.0,
                r'\b(decline|fall|drop)\b': 1.0,
                r'\b(sell-off|selling)\b': 1.0,
                r'\b(paper hands|weak hands)\b': 1.5,
                r'📉|💀|⚠️|🔴': 1.0,  # Emoji patterns
            }
        }

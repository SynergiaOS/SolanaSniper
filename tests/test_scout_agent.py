# SolanaSniper 3.0 - Scout Agent Tests
# OPERACJA "STRESS TEST" - Unit Tests dla Scout Agent

import pytest
import asyncio
import json
from unittest.mock import Mock, AsyncMock, patch
from datetime import datetime
import sys
import os

# Dodaj ścieżkę do agentów
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))

from scout_agent import ScoutAgent
from test_helpers import (
    create_mock_session,
    create_mock_error_session,
    create_sample_news_article,
    assert_mock_called_with_url,
    print_test_summary
)

class TestScoutAgent:
    """
    Kompletny test suite dla Scout Agent
    
    Testuje wszystkie funkcjonalności:
    - Inicjalizację agenta
    - Analizę artykułów
    - Identyfikację okazji
    - Publikację wyników
    - Error handling
    - Edge cases
    """
    
    @pytest.fixture
    def scout_agent(self):
        """Fixture tworzący instancję Scout Agent do testów"""
        return ScoutAgent(livestore_url="http://test:8000")
    
    @pytest.fixture
    def sample_article(self):
        """Fixture z przykładowym artykułem"""
        return {
            'title': 'Solana price surges 15% as new DeFi protocol launches',
            'description': 'Solana (SOL) sees massive pump after Jupiter integration announcement',
            'url': 'https://example.com/solana-news',
            'source': 'TestNews',
            'published_date': '2025-06-09T10:00:00Z'
        }
    
    @pytest.fixture
    def sample_message(self, sample_article):
        """Fixture z przykładową wiadomością z LiveStore"""
        return json.dumps({
            'data': {
                'type': 'news_article',
                'content': sample_article
            }
        })
    
    # === TESTY INICJALIZACJI ===
    
    def test_scout_agent_initialization(self, scout_agent):
        """Test: Czy agent inicjalizuje się poprawnie"""
        assert scout_agent.livestore_url == "http://test:8000"
        assert scout_agent.websocket_url == "ws://test:8000/ws/raw_data"
        assert scout_agent.running == False
        assert scout_agent.stats['messages_received'] == 0
        assert scout_agent.stats['opportunities_found'] == 0
        assert len(scout_agent.solana_keywords) > 0
        assert len(scout_agent.opportunity_keywords) > 0
        assert len(scout_agent.risk_keywords) > 0
    
    def test_scout_agent_custom_url(self):
        """Test: Czy agent akceptuje custom URL"""
        custom_agent = ScoutAgent(livestore_url="http://custom:9000")
        assert custom_agent.livestore_url == "http://custom:9000"
        assert custom_agent.websocket_url == "ws://custom:9000/ws/raw_data"
    
    # === TESTY ANALIZY ARTYKUŁÓW ===
    
    @pytest.mark.asyncio
    async def test_analyze_article_solana_opportunity(self, scout_agent, sample_article):
        """Test: Czy agent rozpoznaje okazję związaną z Solana"""
        analysis = await scout_agent._analyze_article(sample_article)
        
        assert analysis is not None
        assert analysis['solana_related'] == True
        assert analysis['is_opportunity'] == True
        assert analysis['score'] > 0
        assert 'Solana mentions' in str(analysis['reasons'])
    
    @pytest.mark.asyncio
    async def test_analyze_article_no_solana(self, scout_agent):
        """Test: Czy agent odrzuca artykuły niezwiązane z Solana"""
        bitcoin_article = {
            'title': 'Bitcoin reaches new all-time high',
            'description': 'BTC price surges to $100,000',
            'url': 'https://example.com/btc-news',
            'source': 'TestNews'
        }
        
        analysis = await scout_agent._analyze_article(bitcoin_article)
        
        assert analysis is not None
        assert analysis['solana_related'] == False
        assert analysis['is_opportunity'] == False
        # Score może być wysoki przez słowa kluczowe, ale bez Solana nie ma okazji
    
    @pytest.mark.asyncio
    async def test_analyze_article_risk_keywords(self, scout_agent):
        """Test: Czy agent wykrywa ryzykowne słowa kluczowe"""
        risky_article = {
            'title': 'Solana protocol hacked, users lose millions',
            'description': 'Major exploit discovered in Solana DeFi protocol',
            'url': 'https://example.com/hack-news',
            'source': 'TestNews'
        }
        
        analysis = await scout_agent._analyze_article(risky_article)
        
        assert analysis is not None
        assert analysis['score'] < 0  # Negatywny score przez risk keywords
        assert 'Risk keywords' in str(analysis['reasons'])
    
    @pytest.mark.asyncio
    async def test_analyze_article_price_mentions(self, scout_agent):
        """Test: Czy agent wykrywa wzmianki o cenach"""
        price_article = {
            'title': 'SOL hits $200, up 25% today',
            'description': 'Solana price reaches $200 milestone',
            'url': 'https://example.com/price-news',
            'source': 'TestNews'
        }
        
        analysis = await scout_agent._analyze_article(price_article)
        
        assert analysis is not None
        assert analysis['score'] > 20  # Bonus za price mentions
        assert 'Price/percentage mentioned' in str(analysis['reasons'])
    
    # === TESTY PRZETWARZANIA WIADOMOŚCI ===
    
    def test_is_news_article_valid(self, scout_agent):
        """Test: Czy agent rozpoznaje prawidłowe artykuły"""
        valid_data = {
            'data': {
                'type': 'news_article',
                'content': {'title': 'Test article'}
            }
        }
        
        assert scout_agent._is_news_article(valid_data) == True
    
    def test_is_news_article_invalid_type(self, scout_agent):
        """Test: Czy agent odrzuca nieprawidłowe typy"""
        invalid_data = {
            'data': {
                'type': 'other_type',
                'content': {'title': 'Test article'}
            }
        }
        
        assert scout_agent._is_news_article(invalid_data) == False
    
    def test_is_news_article_missing_content(self, scout_agent):
        """Test: Czy agent radzi sobie z brakującym contentem"""
        invalid_data = {
            'data': {
                'type': 'news_article'
                # Brak 'content'
            }
        }
        
        assert scout_agent._is_news_article(invalid_data) == False
    
    def test_is_news_article_malformed(self, scout_agent):
        """Test: Czy agent radzi sobie ze zniekształconymi danymi"""
        malformed_data = {'invalid': 'structure'}
        
        assert scout_agent._is_news_article(malformed_data) == False
    
    # === TESTY ERROR HANDLING ===
    
    @pytest.mark.asyncio
    async def test_analyze_article_empty_title(self, scout_agent):
        """Test: Czy agent radzi sobie z pustym tytułem"""
        empty_article = {
            'title': '',
            'description': '',
            'url': 'https://example.com',
            'source': 'TestNews'
        }
        
        analysis = await scout_agent._analyze_article(empty_article)
        
        assert analysis is not None
        assert analysis['score'] == 0
        assert analysis['is_opportunity'] == False
    
    @pytest.mark.asyncio
    async def test_analyze_article_missing_fields(self, scout_agent):
        """Test: Czy agent radzi sobie z brakującymi polami"""
        incomplete_article = {
            'title': 'Solana news'
            # Brak description, url, source
        }
        
        analysis = await scout_agent._analyze_article(incomplete_article)
        
        assert analysis is not None  # Nie powinien crashować
        assert analysis['solana_related'] == True
    
    @pytest.mark.asyncio
    async def test_process_message_invalid_json(self, scout_agent):
        """Test: Czy agent radzi sobie z nieprawidłowym JSON"""
        invalid_json = "{'invalid': json}"
        
        # Mock session i stats
        scout_agent.session = AsyncMock()
        scout_agent.stats = {'messages_received': 0, 'last_activity': None}
        
        # Nie powinien crashować
        await scout_agent._process_message(invalid_json)
        
        # Stats nie powinny się zmienić
        assert scout_agent.stats['messages_received'] == 0
    
    # === TESTY PUBLIKACJI ===
    
    @pytest.mark.asyncio
    async def test_publish_opportunity_success(self, scout_agent, sample_article):
        """Test: Czy agent publikuje okazje poprawnie"""
        # Użyj naszego helpera do mockowania sesji
        scout_agent.session = create_mock_session()
        scout_agent.stats = {'opportunities_published': 0}

        analysis = {
            'is_opportunity': True,
            'score': 25,
            'reasons': ['Test reason'],
            'solana_related': True,
            'risk_level': 'low'
        }

        await scout_agent._publish_opportunity(sample_article, analysis)

        # Sprawdź czy POST został wywołany
        scout_agent.session.post.assert_called_once()

        # Sprawdź czy stats zostały zaktualizowane
        assert scout_agent.stats['opportunities_published'] == 1
    
    @pytest.mark.asyncio
    async def test_publish_opportunity_http_error(self, scout_agent, sample_article):
        """Test: Czy agent radzi sobie z błędami HTTP"""
        # Użyj helpera do tworzenia sesji z błędami
        scout_agent.session = create_mock_error_session(error_status=500)
        scout_agent.stats = {'opportunities_published': 0}

        analysis = {
            'is_opportunity': True,
            'score': 25,
            'reasons': ['Test reason'],
            'solana_related': True,
            'risk_level': 'low'
        }

        # Nie powinien crashować
        await scout_agent._publish_opportunity(sample_article, analysis)

        # Stats nie powinny się zmienić przy błędzie
        assert scout_agent.stats['opportunities_published'] == 0
    
    @pytest.mark.asyncio
    async def test_publish_opportunity_network_error(self, scout_agent, sample_article):
        """Test: Czy agent radzi sobie z błędami sieci"""
        mock_session = AsyncMock()
        mock_session.post.side_effect = Exception("Network error")
        
        scout_agent.session = mock_session
        scout_agent.stats = {'opportunities_published': 0}
        
        analysis = {
            'is_opportunity': True,
            'score': 25,
            'reasons': ['Test reason'],
            'solana_related': True,
            'risk_level': 'low'
        }
        
        # Nie powinien crashować
        await scout_agent._publish_opportunity(sample_article, analysis)
        
        # Stats nie powinny się zmienić przy błędzie
        assert scout_agent.stats['opportunities_published'] == 0
    
    # === TESTY EDGE CASES ===
    
    @pytest.mark.asyncio
    async def test_analyze_article_unicode_characters(self, scout_agent):
        """Test: Czy agent radzi sobie z Unicode"""
        unicode_article = {
            'title': 'Solana 🚀 price surges! 💎 HODL 📈',
            'description': 'Solana (SOL) 币价上涨 🇨🇳',
            'url': 'https://example.com',
            'source': 'TestNews'
        }
        
        analysis = await scout_agent._analyze_article(unicode_article)
        
        assert analysis is not None
        assert analysis['solana_related'] == True
    
    @pytest.mark.asyncio
    async def test_analyze_article_very_long_text(self, scout_agent):
        """Test: Czy agent radzi sobie z bardzo długim tekstem"""
        long_text = "Solana " * 1000  # 6000 znaków
        
        long_article = {
            'title': long_text,
            'description': long_text,
            'url': 'https://example.com',
            'source': 'TestNews'
        }
        
        analysis = await scout_agent._analyze_article(long_article)
        
        assert analysis is not None
        assert analysis['solana_related'] == True
    
    def test_keyword_case_insensitivity(self, scout_agent):
        """Test: Czy słowa kluczowe są case-insensitive"""
        test_cases = [
            'SOLANA price up',
            'Solana Price Up', 
            'solana price up',
            'SoLaNa PrIcE uP'
        ]
        
        for title in test_cases:
            article = {
                'title': title,
                'description': '',
                'url': 'https://example.com',
                'source': 'TestNews'
            }
            
            # Symulacja analizy (bez async)
            text = f"{title.lower()} "
            solana_mentions = sum(1 for keyword in scout_agent.solana_keywords if keyword in text)
            
            assert solana_mentions > 0, f"Failed for: {title}"

# === TESTY INTEGRACYJNE ===

@pytest.mark.asyncio
async def test_full_message_processing_flow():
    """Test integracyjny: Pełny przepływ przetwarzania wiadomości"""
    scout_agent = ScoutAgent(livestore_url="http://test:8000")

    # Użyj helpera do mockowania sesji
    scout_agent.session = create_mock_session()
    
    # Przygotuj wiadomość
    message = json.dumps({
        'data': {
            'type': 'news_article',
            'content': {
                'title': 'Solana DeFi protocol launches with $50M TVL',
                'description': 'New Jupiter integration brings massive liquidity',
                'url': 'https://example.com/solana-defi',
                'source': 'CryptoNews',
                'published_date': '2025-06-09T10:00:00Z'
            }
        }
    })
    
    # Przetwórz wiadomość
    await scout_agent._process_message(message)
    
    # Sprawdź czy okazja została opublikowana
    scout_agent.session.post.assert_called_once()

    # Sprawdź czy stats zostały zaktualizowane
    assert scout_agent.stats['messages_received'] == 1
    assert scout_agent.stats['opportunities_found'] == 1

if __name__ == "__main__":
    pytest.main([__file__, "-v"])

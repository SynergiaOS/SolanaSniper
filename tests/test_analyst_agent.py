# SolanaSniper 3.0 - Analyst Agent Tests
# OPERACJA "STRESS TEST" - Unit Tests dla Analyst Agent

import pytest
import asyncio
import json
from unittest.mock import Mock, AsyncMock, patch, MagicMock
from datetime import datetime
import sys
import os

# Dodaj ścieżkę do agentów
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))

from agents.analyst.analyst_agent import AnalystAgent

# Import naszych helperów AsyncMock
class AsyncContextManagerMock:
    """Poprawny mock dla async context manager"""

    def __init__(self, status: int = 200, json_data: dict = None):
        self.status = status
        self.json_data = json_data or {'success': True}

    async def __aenter__(self):
        response = AsyncMock()
        response.status = self.status
        response.json = AsyncMock(return_value=self.json_data)
        response.text = AsyncMock(return_value=json.dumps(self.json_data))
        return response

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        return None

def create_ai_mock_session(ai_response: dict):
    """Tworzy sesję dla AI z określoną odpowiedzią"""
    session = AsyncMock()

    def mock_post(*args, **kwargs):
        return AsyncContextManagerMock(
            status=200,
            json_data={'response': json.dumps(ai_response)}
        )

    def mock_get(*args, **kwargs):
        return AsyncContextManagerMock(
            status=200,
            json_data={'models': [{'name': 'gemma:2b'}]}
        )

    session.post = MagicMock(side_effect=mock_post)
    session.get = MagicMock(side_effect=mock_get)
    return session

class TestAnalystAgent:
    """
    Kompletny test suite dla Analyst Agent
    
    Testuje wszystkie funkcjonalności:
    - Inicjalizację agenta
    - Połączenia z LiveStore i Ollama
    - Analizę AI
    - Publikację raportów
    - Error handling
    - Edge cases
    """
    
    @pytest.fixture
    def analyst_agent(self):
        """Fixture tworzący instancję Analyst Agent do testów"""
        return AnalystAgent(
            livestore_url="http://test:8000",
            ollama_url="http://test:11434"
        )
    
    @pytest.fixture
    def sample_opportunity(self):
        """Fixture z przykładową okazją"""
        return {
            'type': 'trading_opportunity',
            'source_article': {
                'title': 'Solana price surges 15% as new DeFi protocol launches',
                'url': 'https://example.com/solana-news',
                'source': 'TestNews',
                'published_date': '2025-06-09T10:00:00Z'
            },
            'analysis': {
                'is_opportunity': True,
                'score': 25,
                'reasons': ['Solana mentions: 1'],
                'solana_related': True,
                'risk_level': 'low'
            }
        }
    
    @pytest.fixture
    def sample_message(self, sample_opportunity):
        """Fixture z przykładową wiadomością z LiveStore"""
        return json.dumps({
            'data': sample_opportunity
        })
    
    @pytest.fixture
    def mock_ai_response(self):
        """Fixture z przykładową odpowiedzią AI"""
        return {
            'sentiment_score': 0.8,
            'key_insight': 'Strong bullish signal for Solana',
            'confidence_score': 0.9,
            'risk_level': 'low',
            'trading_signal': 'buy',
            'time_horizon': 'short'
        }
    
    # === TESTY INICJALIZACJI ===
    
    def test_analyst_agent_initialization(self, analyst_agent):
        """Test: Czy agent inicjalizuje się poprawnie"""
        assert analyst_agent.livestore_url == "http://test:8000"
        assert analyst_agent.ollama_url == "http://test:11434"
        assert analyst_agent.websocket_url == "ws://test:8000/ws/opportunities"
        assert analyst_agent.running == False
        assert analyst_agent.ai_model == "gemma:2b"
        assert analyst_agent.stats['opportunities_received'] == 0
    
    def test_analyst_agent_custom_urls(self):
        """Test: Czy agent akceptuje custom URLs"""
        custom_agent = AnalystAgent(
            livestore_url="http://custom:9000",
            ollama_url="http://custom:12000"
        )
        assert custom_agent.livestore_url == "http://custom:9000"
        assert custom_agent.ollama_url == "http://custom:12000"
        assert custom_agent.websocket_url == "ws://custom:9000/ws/opportunities"
    
    # === TESTY PRZETWARZANIA OKAZJI ===
    
    def test_is_trading_opportunity_valid(self, analyst_agent):
        """Test: Czy agent rozpoznaje prawidłowe okazje"""
        valid_data = {
            'data': {
                'type': 'trading_opportunity',
                'source_article': {'title': 'Test article'}
            }
        }
        
        assert analyst_agent._is_trading_opportunity(valid_data) == True
    
    def test_is_trading_opportunity_invalid_type(self, analyst_agent):
        """Test: Czy agent odrzuca nieprawidłowe typy"""
        invalid_data = {
            'data': {
                'type': 'other_type',
                'source_article': {'title': 'Test article'}
            }
        }
        
        assert analyst_agent._is_trading_opportunity(invalid_data) == False
    
    def test_is_trading_opportunity_missing_article(self, analyst_agent):
        """Test: Czy agent radzi sobie z brakującym artykułem"""
        invalid_data = {
            'data': {
                'type': 'trading_opportunity'
                # Brak 'source_article'
            }
        }
        
        assert analyst_agent._is_trading_opportunity(invalid_data) == False
    
    # === TESTY ANALIZY AI ===
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_success(self, analyst_agent, sample_opportunity, mock_ai_response):
        """Test: Czy analiza AI działa poprawnie"""
        # Użyj naszego naprawionego mocka
        analyst_agent.session = create_ai_mock_session(mock_ai_response)

        result = await analyst_agent._analyze_with_ai(sample_opportunity)

        assert result is not None
        assert result['sentiment_score'] == 0.8
        assert result['trading_signal'] == 'buy'
        assert result['confidence_score'] == 0.9
        
        # Sprawdź czy POST został wywołany z prawidłowymi parametrami
        analyst_agent.session.post.assert_called_once()
        call_args = analyst_agent.session.post.call_args
        assert call_args is not None  # Sprawdź że został wywołany
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_invalid_json_response(self, analyst_agent, sample_opportunity):
        """Test: Czy agent radzi sobie z nieprawidłowym JSON od AI"""
        # Mock session z nieprawidłowym JSON
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json.return_value = {
            'response': 'invalid json response'
        }
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        analyst_agent.stats = {'ai_errors': 0}
        
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        assert result is None
        assert analyst_agent.stats['ai_errors'] == 1
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_incomplete_response(self, analyst_agent, sample_opportunity):
        """Test: Czy agent radzi sobie z niekompletną odpowiedzią AI"""
        # Mock session z niekompletną odpowiedzią
        incomplete_response = {
            'sentiment_score': 0.8,
            # Brak wymaganych pól
        }
        
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json.return_value = {
            'response': json.dumps(incomplete_response)
        }
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        assert result is None
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_http_error(self, analyst_agent, sample_opportunity):
        """Test: Czy agent radzi sobie z błędami HTTP od Ollama"""
        # Mock session z błędem HTTP
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 500
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        analyst_agent.stats = {'ai_errors': 0}
        
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        assert result is None
        assert analyst_agent.stats['ai_errors'] == 1
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_timeout(self, analyst_agent, sample_opportunity):
        """Test: Czy agent radzi sobie z timeout"""
        # Mock session z timeout
        mock_session = AsyncMock()
        mock_session.post.side_effect = asyncio.TimeoutError()
        
        analyst_agent.session = mock_session
        analyst_agent.stats = {'ai_errors': 0}
        
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        assert result is None
        assert analyst_agent.stats['ai_errors'] == 1
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_network_error(self, analyst_agent, sample_opportunity):
        """Test: Czy agent radzi sobie z błędami sieci"""
        # Mock session z błędem sieci
        mock_session = AsyncMock()
        mock_session.post.side_effect = Exception("Network error")
        
        analyst_agent.session = mock_session
        analyst_agent.stats = {'ai_errors': 0}
        
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        assert result is None
        assert analyst_agent.stats['ai_errors'] == 1
    
    # === TESTY PUBLIKACJI RAPORTÓW ===
    
    @pytest.mark.asyncio
    async def test_publish_analysis_report_success(self, analyst_agent, sample_opportunity, mock_ai_response):
        """Test: Czy agent publikuje raporty poprawnie"""
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        analyst_agent.stats = {'reports_published': 0}
        
        await analyst_agent._publish_analysis_report(sample_opportunity, mock_ai_response)
        
        # Sprawdź czy POST został wywołany
        mock_session.post.assert_called_once()
        
        # Sprawdź czy stats zostały zaktualizowane
        assert analyst_agent.stats['reports_published'] == 1
        
        # Sprawdź strukturę raportu
        call_args = mock_session.post.call_args
        report_data = call_args[1]['json']
        
        assert report_data['type'] == 'analysis_report'
        assert 'original_opportunity' in report_data
        assert 'ai_analysis' in report_data
        assert 'summary' in report_data
    
    @pytest.mark.asyncio
    async def test_publish_analysis_report_http_error(self, analyst_agent, sample_opportunity, mock_ai_response):
        """Test: Czy agent radzi sobie z błędami HTTP przy publikacji"""
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 500
        mock_response.text.return_value = "Internal Server Error"
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        analyst_agent.stats = {'reports_published': 0}
        
        # Nie powinien crashować
        await analyst_agent._publish_analysis_report(sample_opportunity, mock_ai_response)
        
        # Stats nie powinny się zmienić przy błędzie
        assert analyst_agent.stats['reports_published'] == 0
    
    # === TESTY ERROR HANDLING ===
    
    @pytest.mark.asyncio
    async def test_process_opportunity_invalid_json(self, analyst_agent):
        """Test: Czy agent radzi sobie z nieprawidłowym JSON"""
        invalid_json = "{'invalid': json}"
        
        # Mock session i stats
        analyst_agent.session = AsyncMock()
        analyst_agent.stats = {'opportunities_received': 0, 'last_activity': None}
        
        # Nie powinien crashować
        await analyst_agent._process_opportunity(invalid_json)
        
        # Stats nie powinny się zmienić
        assert analyst_agent.stats['opportunities_received'] == 0
    
    @pytest.mark.asyncio
    async def test_process_opportunity_malformed_data(self, analyst_agent):
        """Test: Czy agent radzi sobie ze zniekształconymi danymi"""
        malformed_message = json.dumps({'invalid': 'structure'})
        
        # Mock session i stats
        analyst_agent.session = AsyncMock()
        analyst_agent.stats = {'opportunities_received': 0, 'last_activity': None}
        
        # Nie powinien crashować
        await analyst_agent._process_opportunity(malformed_message)
        
        # Stats nie powinny się zmienić
        assert analyst_agent.stats['opportunities_received'] == 0
    
    # === TESTY EDGE CASES ===
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_empty_title(self, analyst_agent):
        """Test: Czy agent radzi sobie z pustym tytułem"""
        empty_opportunity = {
            'source_article': {
                'title': '',
                'url': 'https://example.com',
                'source': 'TestNews'
            }
        }
        
        # Mock session
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json.return_value = {
            'response': json.dumps({
                'sentiment_score': 0.0,
                'key_insight': 'No clear signal',
                'confidence_score': 0.1,
                'risk_level': 'high',
                'trading_signal': 'hold',
                'time_horizon': 'medium'
            })
        }
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        
        result = await analyst_agent._analyze_with_ai(empty_opportunity)
        
        assert result is not None
        assert result['confidence_score'] <= 0.5  # Niska pewność dla pustego tytułu
    
    @pytest.mark.asyncio
    async def test_analyze_with_ai_unicode_title(self, analyst_agent):
        """Test: Czy agent radzi sobie z Unicode w tytule"""
        unicode_opportunity = {
            'source_article': {
                'title': 'Solana 🚀 price surges! 💎 HODL 📈',
                'url': 'https://example.com',
                'source': 'TestNews'
            }
        }
        
        # Mock session
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json.return_value = {
            'response': json.dumps({
                'sentiment_score': 0.8,
                'key_insight': 'Strong bullish signal',
                'confidence_score': 0.9,
                'risk_level': 'low',
                'trading_signal': 'buy',
                'time_horizon': 'short'
            })
        }
        mock_session.post.return_value.__aenter__.return_value = mock_response
        
        analyst_agent.session = mock_session
        
        result = await analyst_agent._analyze_with_ai(unicode_opportunity)
        
        assert result is not None
        assert result['sentiment_score'] > 0

# === TESTY INTEGRACYJNE ===

@pytest.mark.asyncio
async def test_full_opportunity_processing_flow():
    """Test integracyjny: Pełny przepływ przetwarzania okazji"""
    analyst_agent = AnalystAgent(
        livestore_url="http://test:8000",
        ollama_url="http://test:11434"
    )
    
    # Mock session dla AI i publikacji
    mock_session = AsyncMock()
    
    # Mock odpowiedź AI
    ai_response = AsyncMock()
    ai_response.status = 200
    ai_response.json.return_value = {
        'response': json.dumps({
            'sentiment_score': 0.8,
            'key_insight': 'Strong bullish signal for Solana',
            'confidence_score': 0.9,
            'risk_level': 'low',
            'trading_signal': 'buy',
            'time_horizon': 'short'
        })
    }
    
    # Mock odpowiedź publikacji
    publish_response = AsyncMock()
    publish_response.status = 200
    
    # Konfiguruj mock session
    mock_session.post.return_value.__aenter__.side_effect = [ai_response, publish_response]
    analyst_agent.session = mock_session
    
    # Przygotuj wiadomość
    message = json.dumps({
        'data': {
            'type': 'trading_opportunity',
            'source_article': {
                'title': 'Solana DeFi protocol launches with $50M TVL',
                'url': 'https://example.com/solana-defi',
                'source': 'CryptoNews',
                'published_date': '2025-06-09T10:00:00Z'
            },
            'analysis': {
                'is_opportunity': True,
                'score': 25,
                'solana_related': True
            }
        }
    })
    
    # Przetwórz wiadomość
    await analyst_agent._process_opportunity(message)
    
    # Sprawdź czy AI została wywołana i raport opublikowany
    assert mock_session.post.call_count == 2
    
    # Sprawdź czy stats zostały zaktualizowane
    assert analyst_agent.stats['opportunities_received'] == 1
    assert analyst_agent.stats['analyses_completed'] == 1
    assert analyst_agent.stats['reports_published'] == 1

if __name__ == "__main__":
    pytest.main([__file__, "-v"])

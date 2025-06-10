# SolanaSniper 3.0 - Test Helpers
# OPERACJA "ASYNC FIX" - Uniwersalne helpery do testów

import asyncio
from unittest.mock import AsyncMock, MagicMock
from typing import Dict, Any, Optional
import json

class AsyncContextManagerMock:
    """
    Poprawny mock dla async context manager
    
    Rozwiązuje problem: 'coroutine' object does not support the asynchronous context manager protocol
    """
    
    def __init__(self, return_value: Any = None, status: int = 200, json_data: Optional[Dict] = None):
        self.return_value = return_value
        self.status = status
        self.json_data = json_data or {}
        
    async def __aenter__(self):
        # Tworzymy mock response
        response = AsyncMock()
        response.status = self.status
        response.json = AsyncMock(return_value=self.json_data)
        response.text = AsyncMock(return_value=json.dumps(self.json_data))
        return response
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        return None

def create_mock_session():
    """
    Tworzy poprawnie zmockowaną sesję aiohttp
    
    Returns:
        AsyncMock: Sesja z poprawnie zmockowanymi metodami get/post
    """
    session = AsyncMock()
    
    # Mock dla GET requests
    def mock_get(*args, **kwargs):
        return AsyncContextManagerMock(
            status=200,
            json_data={'service': 'LiveStore', 'status': 'running'}
        )
    
    # Mock dla POST requests  
    def mock_post(*args, **kwargs):
        return AsyncContextManagerMock(
            status=200,
            json_data={'success': True, 'message': 'Published'}
        )
    
    session.get = MagicMock(side_effect=mock_get)
    session.post = MagicMock(side_effect=mock_post)
    
    return session

def create_mock_ai_session(ai_response: Dict):
    """
    Tworzy sesję zmockowaną dla AI (Ollama)
    
    Args:
        ai_response: Odpowiedź AI do zwrócenia
        
    Returns:
        AsyncMock: Sesja z mockiem dla Ollama API
    """
    session = AsyncMock()
    
    def mock_post(*args, **kwargs):
        return AsyncContextManagerMock(
            status=200,
            json_data={'response': json.dumps(ai_response)}
        )
    
    session.post = MagicMock(side_effect=mock_post)
    session.get = MagicMock(side_effect=lambda *args, **kwargs: AsyncContextManagerMock(
        status=200,
        json_data={'models': [{'name': 'gemma:2b'}]}
    ))
    
    return session

def create_mock_error_session(error_status: int = 500, error_message: str = "Server Error"):
    """
    Tworzy sesję która zwraca błędy
    
    Args:
        error_status: Kod błędu HTTP
        error_message: Wiadomość błędu
        
    Returns:
        AsyncMock: Sesja zwracająca błędy
    """
    session = AsyncMock()
    
    def mock_error(*args, **kwargs):
        return AsyncContextManagerMock(
            status=error_status,
            json_data={'error': error_message}
        )
    
    session.get = MagicMock(side_effect=mock_error)
    session.post = MagicMock(side_effect=mock_error)
    
    return session

class MockWebSocket:
    """Mock dla WebSocket connections"""
    
    def __init__(self, messages: list = None):
        self.messages = messages or []
        self.index = 0
        
    def __aiter__(self):
        return self
        
    async def __anext__(self):
        if self.index >= len(self.messages):
            raise StopAsyncIteration
        message = self.messages[self.index]
        self.index += 1
        return message
        
    async def send(self, message):
        pass
        
    async def close(self):
        pass

def create_sample_opportunity():
    """Tworzy przykładową okazję handlową do testów"""
    return {
        'type': 'trading_opportunity',
        'source_article': {
            'title': 'Solana DeFi TVL Surges 300% as New DEX Launches',
            'url': 'https://coindesk.com/solana-defi-surge',
            'source': 'CoinDesk',
            'published_date': '2025-06-09T10:00:00Z',
            'content': 'Solana ecosystem sees massive growth with new decentralized exchange.'
        },
        'analysis': {
            'is_opportunity': True,
            'score': 42,
            'reasons': ['Solana mentions: 3', 'DeFi keywords: 5', 'Bullish sentiment'],
            'risk_level': 'medium',
            'solana_related': True
        }
    }

def create_sample_ai_response():
    """Tworzy przykładową odpowiedź AI do testów"""
    return {
        'sentiment_score': 0.8,
        'key_insight': 'Strong bullish signal for Solana DeFi growth',
        'confidence_score': 0.9,
        'risk_level': 'low',
        'trading_signal': 'buy',
        'time_horizon': 'short',
        'price_impact': 'positive',
        'market_context': 'bullish'
    }

def create_sample_news_article():
    """Tworzy przykładowy artykuł do testów"""
    return {
        'title': 'Solana Price Surges 15% Following Major DeFi Protocol Launch',
        'url': 'https://example.com/solana-news',
        'source': 'CryptoNews',
        'published_date': '2025-06-09T10:00:00Z',
        'content': 'Solana blockchain experiences significant price movement as new DeFi protocol launches with innovative features.',
        'summary': 'Major DeFi protocol launches on Solana, driving price surge.'
    }

async def wait_for_async_operations(timeout: float = 1.0):
    """
    Czeka na zakończenie operacji asynchronicznych w testach
    
    Args:
        timeout: Maksymalny czas oczekiwania w sekundach
    """
    await asyncio.sleep(0.1)  # Daj czas na wykonanie async operacji

def assert_mock_called_with_url(mock_method, expected_url_part: str):
    """
    Sprawdza czy mock został wywołany z URL zawierającym określoną część
    
    Args:
        mock_method: Mock metody (get/post)
        expected_url_part: Oczekiwana część URL
    """
    assert mock_method.called, "Mock nie został wywołany"
    call_args = mock_method.call_args
    url = call_args[0][0] if call_args[0] else ""
    assert expected_url_part in url, f"URL '{url}' nie zawiera '{expected_url_part}'"

def print_test_summary(test_name: str, passed: bool, details: str = ""):
    """Wyświetla podsumowanie testu"""
    status = "✅ PASSED" if passed else "❌ FAILED"
    print(f"{status} {test_name}")
    if details:
        print(f"   {details}")

# Fixtures dla pytest
import pytest

@pytest.fixture
def mock_session():
    """Fixture z poprawnie zmockowaną sesją"""
    return create_mock_session()

@pytest.fixture
def mock_ai_session():
    """Fixture z sesją dla AI"""
    return create_mock_ai_session(create_sample_ai_response())

@pytest.fixture
def sample_opportunity():
    """Fixture z przykładową okazją"""
    return create_sample_opportunity()

@pytest.fixture
def sample_ai_response():
    """Fixture z odpowiedzią AI"""
    return create_sample_ai_response()

@pytest.fixture
def sample_article():
    """Fixture z artykułem"""
    return create_sample_news_article()

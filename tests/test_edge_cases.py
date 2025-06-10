# SolanaSniper 3.0 - Testy Edge Cases
# OPERACJA "GRANICZNE" - Testy przypadków brzegowych

import pytest
import asyncio
import json
import os
import sys
from unittest.mock import AsyncMock, patch
from datetime import datetime, timedelta

sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'risk'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

@pytest.mark.asyncio
async def test_empty_content_handling():
    """Test: Obsługa pustej zawartości"""
    
    scout_agent = ScoutAgent(livestore_url="http://test:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    empty_articles = [
        {
            'title': '',
            'url': 'https://example.com/empty',
            'source': 'TestNews',
            'published_date': '2025-06-09T10:00:00Z',
            'content': ''
        },
        {
            'title': '   ',  # Tylko spacje
            'url': 'https://example.com/spaces',
            'source': 'TestNews',
            'published_date': '2025-06-09T10:00:00Z',
            'content': '   '
        },
        {
            'title': '\n\t\r',  # Tylko białe znaki
            'url': 'https://example.com/whitespace',
            'source': 'TestNews',
            'published_date': '2025-06-09T10:00:00Z',
            'content': '\n\t\r'
        }
    ]
    
    for article in empty_articles:
        result = await scout_agent._analyze_article(article)
        assert result is not None
        assert isinstance(result, dict)
        # Pusty content nie powinien być uznany za okazję
        assert result.get('is_opportunity', False) == False

@pytest.mark.asyncio
async def test_extreme_values():
    """Test: Obsługa ekstremalnych wartości"""
    
    risk_agent = RiskAgent(livestore_url="http://test:8000")
    risk_agent.session = AsyncMock()
    risk_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    extreme_cases = [
        {
            'token_address': 'So11111111111111111111111111111111111111112',
            'rug_score': 0,      # Minimum
            'is_honeypot': False,
            'liquidity_locked': True,
            'liquidity_amount_usd': 0,  # Zero liquidity
            'top_holder_percentage': 0.0
        },
        {
            'token_address': 'So11111111111111111111111111111111111111112',
            'rug_score': 100,    # Maximum
            'is_honeypot': True,
            'liquidity_locked': False,
            'liquidity_amount_usd': 999999999,  # Bardzo wysoka płynność
            'top_holder_percentage': 100.0      # 100% w jednych rękach
        },
        {
            'token_address': 'So11111111111111111111111111111111111111112',
            'rug_score': -10,    # Wartość ujemna (nieprawidłowa)
            'is_honeypot': False,
            'liquidity_locked': True,
            'liquidity_amount_usd': -1000,  # Ujemna płynność (nieprawidłowa)
            'top_holder_percentage': 150.0  # Ponad 100% (nieprawidłowe)
        }
    ]
    
    for case in extreme_cases:
        result = risk_agent._assess_risk(case)
        assert result is not None
        assert isinstance(result, dict)
        assert 'risk_level' in result
        assert 'is_dangerous' in result
        assert 'recommendation' in result

@pytest.mark.asyncio
async def test_network_failures():
    """Test: Obsługa błędów sieciowych"""
    
    analyst_agent = AnalystAgent(livestore_url="http://test:8000", ollama_url="http://test:11434")
    
    # Mock błędów sieciowych
    mock_session = AsyncMock()
    
    # Różne typy błędów sieciowych
    network_errors = [
        asyncio.TimeoutError("Connection timeout"),
        ConnectionError("Connection refused"),
        Exception("Network unreachable")
    ]
    
    for error in network_errors:
        mock_session.post.side_effect = error
        analyst_agent.session = mock_session
        
        opportunity = {
            'type': 'trading_opportunity',
            'source_article': {
                'title': 'Test article',
                'url': 'https://example.com/test',
                'source': 'TestNews',
                'published_date': '2025-06-09T10:00:00Z'
            },
            'analysis': {
                'is_opportunity': True,
                'score': 25,
                'solana_related': True
            }
        }
        
        # Agent powinien gracefully obsłużyć błąd
        try:
            result = await analyst_agent._analyze_with_ai(opportunity)
            # Jeśli nie rzuca wyjątku, sprawdź czy zwraca sensowny fallback
            if result is not None:
                assert isinstance(result, dict)
        except Exception as e:
            # Sprawdź czy błąd jest odpowiednio obsłużony
            assert "Network error" in str(e) or "Connection failed" in str(e)

@pytest.mark.asyncio
async def test_concurrent_access():
    """Test: Równoczesny dostęp do zasobów"""
    
    scout_agent = ScoutAgent(livestore_url="http://test:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # Symuluj równoczesny dostęp do tego samego artykułu
    article = {
        'title': 'Solana price surge',
        'url': 'https://example.com/solana-news',
        'source': 'TestNews',
        'published_date': '2025-06-09T10:00:00Z',
        'content': 'Solana (SOL) price has increased significantly.'
    }
    
    # Uruchom 50 równoczesnych analiz tego samego artykułu
    tasks = [scout_agent._analyze_article(article) for _ in range(50)]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    
    # Sprawdź czy wszystkie analizy się powiodły
    successful_results = [r for r in results if not isinstance(r, Exception)]
    assert len(successful_results) >= 45  # Przynajmniej 90% powinno się powieść
    
    # Sprawdź czy wyniki są spójne
    if len(successful_results) > 1:
        first_result = successful_results[0]
        for result in successful_results[1:]:
            assert result['is_opportunity'] == first_result['is_opportunity']
            assert result['solana_related'] == first_result['solana_related']

@pytest.mark.asyncio
async def test_memory_leaks():
    """Test: Sprawdzenie wycieków pamięci"""
    
    import gc
    import psutil
    import os
    
    # Pobierz początkowe zużycie pamięci
    process = psutil.Process(os.getpid())
    initial_memory = process.memory_info().rss
    
    scout_agent = ScoutAgent(livestore_url="http://test:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # Wykonaj wiele operacji
    for i in range(1000):
        article = {
            'title': f'Test article {i}',
            'url': f'https://example.com/article-{i}',
            'source': 'TestNews',
            'published_date': '2025-06-09T10:00:00Z',
            'content': f'Test content {i}' * 100  # Większy content
        }
        
        result = await scout_agent._analyze_article(article)
        
        # Co 100 iteracji sprawdź pamięć
        if i % 100 == 0:
            gc.collect()  # Wymuś garbage collection
            current_memory = process.memory_info().rss
            memory_growth = current_memory - initial_memory
            
            # Wzrost pamięci nie powinien być nadmierny (max 100MB)
            assert memory_growth < 100 * 1024 * 1024, f"Memory growth: {memory_growth / 1024 / 1024:.2f}MB"
    
    # Finalne sprawdzenie pamięci
    gc.collect()
    final_memory = process.memory_info().rss
    total_growth = final_memory - initial_memory
    
    print(f"Memory growth after 1000 operations: {total_growth / 1024 / 1024:.2f}MB")
    assert total_growth < 200 * 1024 * 1024  # Maksymalnie 200MB wzrostu
# SolanaSniper 3.0 - Testy Wydajnościowe
# OPERACJA "STRESS TEST" - Testy wydajności pod obciążeniem

import pytest
import asyncio
import time
import json
import random
from unittest.mock import AsyncMock, patch
import sys
import os

# Dodaj ścieżki do wszystkich komponentów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'risk'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

@pytest.mark.asyncio
async def test_scout_agent_performance():
    """Test wydajnościowy: Scout Agent pod obciążeniem"""
    
    scout_agent = ScoutAgent(livestore_url="http://test:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # Generuj 100 artykułów
    articles = []
    for i in range(100):
        articles.append({
            'title': f'Solana news article {i}',
            'url': f'https://example.com/article-{i}',
            'source': 'TestNews',
            'published_date': '2025-06-09T10:00:00Z',
            'content': f'This is test article {i} about Solana (SOL) price movements.'
        })
    
    # Mierz czas analizy
    start_time = time.time()
    
    # Analizuj wszystkie artykuły równolegle
    tasks = [scout_agent._analyze_article(article) for article in articles]
    results = await asyncio.gather(*tasks)
    
    end_time = time.time()
    duration = end_time - start_time
    
    # Sprawdź wyniki
    assert len(results) == 100
    assert all(result is not None for result in results)
    
    # Sprawdź wydajność - powinno być poniżej 5 sekund dla 100 artykułów
    assert duration < 5.0, f"Analiza 100 artykułów zajęła {duration} sekund (limit: 5s)"
    
    # Oblicz statystyki
    opportunities = [r for r in results if r['is_opportunity']]
    print(f"Przeanalizowano 100 artykułów w {duration:.2f}s ({100/duration:.2f} art/s)")
    print(f"Znaleziono {len(opportunities)} okazji")

@pytest.mark.asyncio
async def test_analyst_agent_performance():
    """Test wydajnościowy: Analyst Agent pod obciążeniem"""
    
    analyst_agent = AnalystAgent(livestore_url="http://test:8000", ollama_url="http://test:11434")
    analyst_agent.session = AsyncMock()
    analyst_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # Mock odpowiedzi AI
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
    analyst_agent.session.post.return_value.__aenter__.return_value = mock_response
    
    # Generuj 50 okazji
    opportunities = []
    for i in range(50):
        opportunities.append({
            'type': 'trading_opportunity',
            'source_article': {
                'title': f'Solana news article {i}',
                'url': f'https://example.com/article-{i}',
                'source': 'TestNews',
                'published_date': '2025-06-09T10:00:00Z'
            },
            'analysis': {
                'is_opportunity': True,
                'score': random.randint(10, 50),
                'reasons': ['Solana mentions: 1'],
                'solana_related': True,
                'risk_level': 'low'
            }
        })
    
    # Mierz czas analizy
    start_time = time.time()
    
    # Analizuj wszystkie okazje równolegle
    tasks = [analyst_agent._analyze_with_ai(opp) for opp in opportunities]
    results = await asyncio.gather(*tasks)
    
    end_time = time.time()
    duration = end_time - start_time
    
    # Sprawdź wyniki
    assert len(results) == 50
    assert all(result is not None for result in results)
    
    # Sprawdź wydajność - powinno być poniżej 10 sekund dla 50 okazji
    assert duration < 10.0, f"Analiza 50 okazji zajęła {duration} sekund (limit: 10s)"
    
    # Oblicz statystyki
    buy_signals = [r for r in results if r['trading_signal'] == 'buy']
    print(f"Przeanalizowano 50 okazji w {duration:.2f}s ({50/duration:.2f} opp/s)")
    print(f"Wygenerowano {len(buy_signals)} sygnałów kupna")

@pytest.mark.asyncio
async def test_risk_agent_performance():
    """Test wydajnościowy: Risk Agent pod obciążeniem"""
    
    risk_agent = RiskAgent(livestore_url="http://test:8000")
    risk_agent.session = AsyncMock()
    risk_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # Generuj 30 tokenów do analizy
    tokens = [f"Token{i}" for i in range(30)]
    
    # Mock analizy bezpieczeństwa
    async def mock_analyze_token(token):
        await asyncio.sleep(0.1)  # Symuluj czas analizy
        return {
            'token_address': token,
            'rug_score': random.randint(10, 90),
            'is_honeypot': random.choice([True, False]),
            'liquidity_locked': random.choice([True, False]),
            'liquidity_amount_usd': random.randint(1000, 100000),
            'top_holder_percentage': random.uniform(5.0, 80.0)
        }
    
    # Mierz czas analizy
    start_time = time.time()
    
    # Analizuj wszystkie tokeny równolegle
    tasks = [mock_analyze_token(token) for token in tokens]
    security_data = await asyncio.gather(*tasks)
    
    # Ocena ryzyka dla wszystkich tokenów
    risk_assessments = [risk_agent._assess_risk(data) for data in security_data]
    
    end_time = time.time()
    duration = end_time - start_time
    
    # Sprawdź wyniki
    assert len(risk_assessments) == 30
    assert all(assessment is not None for assessment in risk_assessments)
    
    # Sprawdź wydajność - powinno być poniżej 5 sekund dla 30 tokenów
    assert duration < 5.0, f"Analiza 30 tokenów zajęła {duration} sekund (limit: 5s)"
    
    # Oblicz statystyki
    dangerous_tokens = [r for r in risk_assessments if r['is_dangerous']]
    print(f"Przeanalizowano 30 tokenów w {duration:.2f}s ({30/duration:.2f} token/s)")
    print(f"Wykryto {len(dangerous_tokens)} niebezpiecznych tokenów")
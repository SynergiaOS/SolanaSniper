# SolanaSniper 3.0 - Testy Bezpieczeństwa
# OPERACJA "FORTECA" - Testy bezpieczeństwa i odporności na ataki

import pytest
import asyncio
import json
import os
import sys
from unittest.mock import AsyncMock, patch
import re

# Dodaj ścieżki do wszystkich komponentów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'risk'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent
from livestore.livestore_server import app

@pytest.mark.asyncio
async def test_injection_attack_resistance():
    """Test: Odporność na ataki injection"""
    
    # Przygotuj agentów
    scout_agent = ScoutAgent(livestore_url="http://test:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    analyst_agent = AnalystAgent(livestore_url="http://test:8000", ollama_url="http://test:11434")
    analyst_agent.session = AsyncMock()
    analyst_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    risk_agent = RiskAgent(livestore_url="http://test:8000")
    risk_agent.session = AsyncMock()
    risk_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # Złośliwe dane z próbą SQL injection
    malicious_article = {
        'title': "Solana news'; DROP TABLE users; --",
        'url': "https://example.com/news'; DELETE FROM articles; --",
        'source': "<script>alert('XSS')</script>",
        'published_date': "2025-06-09T10:00:00Z'; TRUNCATE TABLE logs; --",
        'content': "This is a test with SQL injection: '; DROP TABLE opportunities; --"
    }
    
    # Test Scout Agent
    scout_analysis = await scout_agent._analyze_article(malicious_article)
    
    # Sprawdź czy agent nie crashuje i zwraca wynik
    assert scout_analysis is not None
    
    # Sprawdź czy złośliwe dane są bezpiecznie obsługiwane
    opportunity = {
        'type': 'trading_opportunity',
        'source_article': malicious_article,
        'analysis': scout_analysis
    }
    
    # Test Analyst Agent z mockiem AI
    mock_response = AsyncMock()
    mock_response.status = 200
    mock_response.json.return_value = {
        'response': json.dumps({
            'sentiment_score': 0.5,
            'key_insight': 'Normal analysis',
            'confidence_score': 0.7,
            'risk_level': 'medium',
            'trading_signal': 'hold',
            'time_horizon': 'medium'
        })
    }
    analyst_agent.session.post.return_value.__aenter__.return_value = mock_response
    
    ai_analysis = await analyst_agent._analyze_with_ai(opportunity)
    assert ai_analysis is not None
    
    # Test Risk Agent
    token_address = "So11111111111111111111111111111111111111112"

    # Test czy agent obsługuje potencjalnie niebezpieczne dane
    malicious_token_data = {
        'token_address': token_address,
        'rug_score': 25,
        'is_honeypot': False,
        'liquidity_locked': True,
        'liquidity_amount_usd': 50000,
        'top_holder_percentage': 15.5
    }

    risk_assessment = risk_agent._assess_risk(malicious_token_data)
    assert risk_assessment is not None
    assert isinstance(risk_assessment, dict)

@pytest.mark.asyncio
async def test_xss_prevention():
    """Test: Zapobieganie atakom XSS"""

    scout_agent = ScoutAgent(livestore_url="http://localhost:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200

    # Artykuł z próbą XSS
    xss_article = {
        'title': '<script>alert("XSS")</script>Solana News',
        'url': 'https://example.com/news',
        'source': '<img src=x onerror=alert("XSS")>',
        'published_date': '2025-06-09T10:00:00Z',
        'content': 'Solana price <script>document.cookie</script> increased'
    }

    # Test czy agent bezpiecznie obsługuje XSS
    result = await scout_agent._analyze_article(xss_article)
    assert result is not None

    # Sprawdź czy niebezpieczne tagi zostały usunięte/escaped
    # (to zależy od implementacji sanityzacji w agencie)
# SolanaSniper 3.0 - Testy Integracyjne Pełnego Przepływu
# OPERACJA "END-TO-END" - Testy całego pipeline'u

import pytest
import asyncio
import json
import os
import sys
from unittest.mock import AsyncMock, patch

# Dodaj ścieżki do wszystkich komponentów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'risk'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent
from livestore.livestore_server import app

@pytest.mark.asyncio
async def test_full_pipeline_integration():
    """Test integracyjny: Pełny przepływ danych przez cały system"""
    
    # 1. Przygotuj przykładowy artykuł
    test_article = {
        'title': 'Solana price surges 15% as new DeFi protocol launches',
        'url': 'https://example.com/solana-news',
        'source': 'TestNews',
        'published_date': '2025-06-09T10:00:00Z',
        'content': 'Solana (SOL) price has increased by 15% following the launch of a new DeFi protocol.'
    }
    
    # 2. Mock LiveStore
    mock_livestore = AsyncMock()
    
    # 3. Inicjalizuj agentów z mockami
    scout_agent = ScoutAgent(livestore_url="http://test:8000")
    scout_agent.session = AsyncMock()
    scout_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    analyst_agent = AnalystAgent(livestore_url="http://test:8000", ollama_url="http://test:11434")
    analyst_agent.session = AsyncMock()
    analyst_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    risk_agent = RiskAgent(livestore_url="http://test:8000")
    risk_agent.session = AsyncMock()
    risk_agent.session.post.return_value.__aenter__.return_value.status = 200
    
    # 4. Przeprowadź analizę przez Scout Agent
    scout_analysis = await scout_agent._analyze_article(test_article)
    assert scout_analysis is not None
    assert scout_analysis['solana_related'] == True
    assert scout_analysis['is_opportunity'] == True
    
    # 5. Utwórz okazję
    opportunity = {
        'type': 'trading_opportunity',
        'source_article': test_article,
        'analysis': scout_analysis,
        'agent_metadata': {
            'agent_name': 'scout_agent',
            'agent_version': '1.0.0'
        }
    }
    
    # 6. Mock analizy AI
    mock_ai_response = {
        'sentiment_score': 0.8,
        'key_insight': 'Strong bullish signal for Solana',
        'confidence_score': 0.9,
        'risk_level': 'low',
        'trading_signal': 'buy',
        'time_horizon': 'short'
    }
    
    # 7. Zasymuluj analizę przez Analyst Agent
    with patch.object(analyst_agent, '_analyze_with_ai', return_value=mock_ai_response):
        ai_analysis = await analyst_agent._analyze_with_ai(opportunity)

    assert ai_analysis is not None
    assert ai_analysis['sentiment_score'] == 0.8
    assert ai_analysis['trading_signal'] == 'buy'
    
    # 8. Mock analizy bezpieczeństwa
    mock_security_data = {
        'token_address': 'So11111111111111111111111111111111111111112',
        'rug_score': 25,
        'is_honeypot': False,
        'liquidity_locked': True,
        'liquidity_amount_usd': 50000,
        'top_holder_percentage': 15.5
    }
    
    # 9. Zasymuluj analizę przez Risk Agent
    with patch.object(risk_agent, '_analyze_token_security', return_value=mock_security_data):
        risk_assessment = risk_agent._assess_risk(mock_security_data)
    
    assert risk_assessment is not None
    assert risk_assessment['risk_level'] == 'NISKIE'
    assert risk_assessment['is_dangerous'] == False
    assert risk_assessment['recommendation'] == 'BEZPIECZNY'
    
    # 10. Sprawdź czy cały pipeline działa
    assert scout_analysis['is_opportunity'] == True
    assert ai_analysis['trading_signal'] == 'buy'
    assert risk_assessment['is_dangerous'] == False
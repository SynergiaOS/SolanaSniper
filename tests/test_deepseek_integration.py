# SolanaSniper 3.0 - Testy integracji DeepSeek API
# OPERACJA "DEEPSEEK MASTER" - Testy Real-Time RAG z DeepSeek

import pytest
import asyncio
import json
from unittest.mock import AsyncMock, MagicMock
import sys
import os

# Dodaj ścieżki
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))

from agents.analyst.analyst_agent import AnalystAgent

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

def create_deepseek_mock_session(deepseek_response: dict):
    """Tworzy sesję zmockowaną dla DeepSeek API"""
    session = AsyncMock()
    
    def mock_post(*args, **kwargs):
        # Mock dla DeepSeek API
        if 'deepseek.com' in args[0]:
            return AsyncContextManagerMock(
                status=200,
                json_data={
                    'choices': [
                        {
                            'message': {
                                'content': json.dumps(deepseek_response)
                            }
                        }
                    ],
                    'usage': {
                        'prompt_tokens': 500,
                        'completion_tokens': 150,
                        'total_tokens': 650
                    }
                }
            )
        # Mock dla LiveStore
        else:
            return AsyncContextManagerMock(status=200, json_data={'success': True})
    
    def mock_get(*args, **kwargs):
        return AsyncContextManagerMock(
            status=200,
            json_data={'service': 'LiveStore', 'status': 'running'}
        )
    
    session.post = MagicMock(side_effect=mock_post)
    session.get = MagicMock(side_effect=mock_get)
    return session

class TestDeepSeekIntegration:
    """Testy integracji z DeepSeek API"""

    @pytest.fixture
    def analyst_agent(self):
        """Fixture z Analyst Agent używającym DeepSeek"""
        agent = AnalystAgent(
            livestore_url="http://localhost:8000",
            deepseek_api_key="sk-test-key"
        )
        return agent

    @pytest.fixture
    def sample_opportunity(self):
        """Przykładowa okazja handlowa"""
        return {
            'type': 'trading_opportunity',
            'source_article': {
                'title': 'Solana DeFi TVL Surges 300% as New DEX Launches with Revolutionary AMM',
                'url': 'https://coindesk.com/solana-defi-surge',
                'source': 'CoinDesk',
                'published_date': '2025-06-09T10:00:00Z',
                'content': 'Solana ecosystem sees massive growth with new decentralized exchange launching innovative automated market maker technology.'
            },
            'analysis': {
                'is_opportunity': True,
                'score': 42,
                'reasons': ['Solana mentions: 3', 'DeFi keywords: 5', 'Bullish sentiment detected'],
                'risk_level': 'medium',
                'solana_related': True
            }
        }

    @pytest.fixture
    def deepseek_response(self):
        """Przykładowa odpowiedź DeepSeek"""
        return {
            'sentiment_score': 0.8,
            'key_insight': 'Strong bullish signal for Solana DeFi growth with new DEX launch',
            'confidence_score': 0.9,
            'risk_level': 'low',
            'trading_signal': 'buy',
            'time_horizon': 'short',
            'price_impact': 'positive',
            'market_context': 'bullish'
        }

    @pytest.mark.asyncio
    async def test_deepseek_api_configuration(self, analyst_agent):
        """Test: Konfiguracja DeepSeek API"""
        
        assert analyst_agent.deepseek_api_key == "sk-test-key"
        assert analyst_agent.deepseek_url == "https://api.deepseek.com/v1/chat/completions"
        assert analyst_agent.ai_model == "deepseek-reasoner"
        
        # Sprawdź statystyki
        assert 'api_calls' in analyst_agent.stats
        assert 'tokens_used' in analyst_agent.stats
        assert 'cost_usd' in analyst_agent.stats

    @pytest.mark.asyncio
    async def test_deepseek_connection_test(self, analyst_agent, deepseek_response):
        """Test: Test połączenia z DeepSeek API"""
        
        # Mock session
        analyst_agent.session = create_deepseek_mock_session(deepseek_response)
        
        # Test połączenia
        await analyst_agent._test_connections()
        
        # Sprawdź czy wywołania zostały wykonane
        assert analyst_agent.session.get.called  # LiveStore test
        assert analyst_agent.session.post.called  # DeepSeek test

    @pytest.mark.asyncio
    async def test_deepseek_rag_analysis(self, analyst_agent, sample_opportunity, deepseek_response):
        """Test: Pełna analiza RAG z DeepSeek"""
        
        # Mock session
        analyst_agent.session = create_deepseek_mock_session(deepseek_response)
        
        # Wykonaj analizę
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        # Sprawdź wyniki
        assert result is not None
        assert result['sentiment_score'] == 0.8
        assert result['trading_signal'] == 'buy'
        assert result['price_impact'] == 'positive'
        assert result['market_context'] == 'bullish'
        
        # Sprawdź statystyki
        assert analyst_agent.stats['api_calls'] == 1
        assert analyst_agent.stats['tokens_used'] == 650
        assert analyst_agent.stats['cost_usd'] > 0

    @pytest.mark.asyncio
    async def test_deepseek_cost_calculation(self, analyst_agent, sample_opportunity, deepseek_response):
        """Test: Kalkulacja kosztów DeepSeek"""
        
        # Mock session
        analyst_agent.session = create_deepseek_mock_session(deepseek_response)
        
        # Wykonaj analizę
        await analyst_agent._analyze_with_ai(sample_opportunity)
        
        # Sprawdź koszty (off-peak pricing)
        expected_input_cost = (500 / 1000000) * 0.135  # 500 tokens * $0.135/1M
        expected_output_cost = (150 / 1000000) * 0.550  # 150 tokens * $0.550/1M
        expected_total = expected_input_cost + expected_output_cost
        
        assert abs(analyst_agent.stats['cost_usd'] - expected_total) < 0.0001

    @pytest.mark.asyncio
    async def test_deepseek_prompt_structure(self, analyst_agent, sample_opportunity):
        """Test: Struktura promptu dla DeepSeek"""
        
        article = sample_opportunity['source_article']
        analysis = sample_opportunity['analysis']
        
        # Zbuduj wzbogacony kontekst
        enriched_context = await analyst_agent._build_enriched_context(article, analysis)
        
        # Zbuduj prompt
        prompt = await analyst_agent._build_rag_prompt(enriched_context)
        
        # Sprawdź strukturę promptu
        assert 'SOLANA TRADING ANALYSIS' in prompt
        assert 'Real-Time RAG System' in prompt
        assert article['title'] in prompt
        assert 'SCOUT AGENT ANALYSIS' in prompt
        assert 'KEYWORD ANALYSIS' in prompt
        assert 'sentiment_score' in prompt
        assert 'trading_signal' in prompt
        assert 'price_impact' in prompt
        assert 'market_context' in prompt

    @pytest.mark.asyncio
    async def test_deepseek_error_handling(self, analyst_agent, sample_opportunity):
        """Test: Obsługa błędów DeepSeek API"""
        
        # Mock session z błędem
        session = AsyncMock()
        
        def mock_post_error(*args, **kwargs):
            return AsyncContextManagerMock(
                status=401,
                json_data={'error': 'Invalid API key'}
            )
        
        session.post = MagicMock(side_effect=mock_post_error)
        analyst_agent.session = session
        
        # Wykonaj analizę
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        # Sprawdź obsługę błędu
        assert result is None
        assert analyst_agent.stats['ai_errors'] == 1

    @pytest.mark.asyncio
    async def test_deepseek_json_validation(self, analyst_agent, sample_opportunity):
        """Test: Walidacja JSON z DeepSeek"""
        
        # Mock session z nieprawidłowym JSON
        invalid_response = {
            'sentiment_score': 0.8,
            # Brakuje wymaganych pól
        }
        
        analyst_agent.session = create_deepseek_mock_session(invalid_response)
        
        # Wykonaj analizę
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        # Sprawdź walidację
        assert result is None  # Powinno zwrócić None dla niepełnej odpowiedzi

    @pytest.mark.asyncio
    async def test_deepseek_vs_ollama_comparison(self):
        """Test: Porównanie DeepSeek vs Ollama"""
        
        # DeepSeek Agent
        deepseek_agent = AnalystAgent(
            livestore_url="http://localhost:8000",
            deepseek_api_key="sk-test-key"
        )
        
        # Sprawdź różnice
        assert deepseek_agent.ai_model == "deepseek-reasoner"
        assert deepseek_agent.deepseek_url == "https://api.deepseek.com/v1/chat/completions"
        assert 'cost_usd' in deepseek_agent.stats
        
        print("✅ DeepSeek Integration:")
        print(f"  Model: {deepseek_agent.ai_model}")
        print(f"  API URL: {deepseek_agent.deepseek_url}")
        print(f"  Cost tracking: {'cost_usd' in deepseek_agent.stats}")

    def test_deepseek_budget_estimation(self):
        """Test: Oszacowanie budżetu DeepSeek"""
        
        # Parametry
        budget_usd = 15.0
        avg_tokens_per_analysis = 650
        analyses_per_day = 100
        
        # Koszty off-peak (75% zniżka dla reasoner)
        cost_per_analysis = (650 / 1000000) * (0.135 + 0.550)  # Input + Output
        
        analyses_possible = budget_usd / cost_per_analysis
        days_possible = analyses_possible / analyses_per_day
        
        print(f"💰 DeepSeek Budget Analysis:")
        print(f"  Budget: ${budget_usd}")
        print(f"  Cost per analysis: ${cost_per_analysis:.6f}")
        print(f"  Possible analyses: {analyses_possible:.0f}")
        print(f"  Days of operation: {days_possible:.1f}")
        
        # $15 powinno starczyć na długo
        assert days_possible > 30  # Ponad miesiąc

if __name__ == "__main__":
    # Uruchom testy bezpośrednio
    import asyncio
    
    async def run_tests():
        print("🧪 Testing DeepSeek Integration...")
        
        # Test 1: Konfiguracja
        agent = AnalystAgent(deepseek_api_key="sk-test-key")
        assert agent.ai_model == "deepseek-reasoner"
        print("✅ Test 1: Configuration - PASSED")
        
        # Test 2: Budget estimation
        test = TestDeepSeekIntegration()
        test.test_deepseek_budget_estimation()
        print("✅ Test 2: Budget estimation - PASSED")
        
        print("🎉 All DeepSeek integration tests PASSED!")
    
    asyncio.run(run_tests())

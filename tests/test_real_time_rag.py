# SolanaSniper 3.0 - Testy Real-Time RAG System
# OPERACJA "RAG MASTER" - Testy dynamicznego systemu RAG

import pytest
import asyncio
import json
from unittest.mock import AsyncMock, patch
import sys
import os
from datetime import datetime

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))

from agents.analyst.analyst_agent import AnalystAgent

class TestRealTimeRAG:
    """Testy Real-Time RAG System - Serce SolanaSniper 3.0"""

    @pytest.fixture
    def analyst_agent(self):
        """Fixture z Analyst Agent"""
        agent = AnalystAgent(
            livestore_url="http://localhost:8000",
            ollama_url="http://localhost:11434"
        )
        agent.session = AsyncMock()
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
                'content': 'Solana ecosystem sees massive growth with new decentralized exchange launching innovative automated market maker technology. Total Value Locked increases dramatically.'
            },
            'analysis': {
                'is_opportunity': True,
                'score': 42,
                'reasons': ['Solana mentions: 3', 'DeFi keywords: 5', 'Bullish sentiment detected'],
                'risk_level': 'medium',
                'solana_related': True
            }
        }

    @pytest.mark.asyncio
    async def test_build_enriched_context(self, analyst_agent, sample_opportunity):
        """Test: Budowanie wzbogaconego kontekstu RAG"""
        
        article = sample_opportunity['source_article']
        analysis = sample_opportunity['analysis']
        
        # Test wzbogacania kontekstu
        enriched_context = await analyst_agent._build_enriched_context(article, analysis)
        
        # Sprawdź strukturę
        assert 'article' in enriched_context
        assert 'scout_analysis' in enriched_context
        assert 'context_metadata' in enriched_context
        
        # Sprawdź dane artykułu
        assert enriched_context['article']['title'] == article['title']
        assert enriched_context['article']['source'] == article['source']
        assert enriched_context['article']['word_count'] > 0
        
        # Sprawdź analizę Scout
        assert enriched_context['scout_analysis']['opportunity_score'] == 42
        assert enriched_context['scout_analysis']['solana_relevance'] == True
        
        # Sprawdź metadane
        assert 'source_credibility' in enriched_context['context_metadata']
        assert 'temporal_relevance' in enriched_context['context_metadata']
        assert 'keyword_density' in enriched_context['context_metadata']

    @pytest.mark.asyncio
    async def test_assess_source_credibility(self, analyst_agent):
        """Test: Ocena wiarygodności źródeł"""
        
        # Test wysokiej wiarygodności
        high_cred = await analyst_agent._assess_source_credibility('CoinDesk')
        assert high_cred == 'high'
        
        # Test średniej wiarygodności
        medium_cred = await analyst_agent._assess_source_credibility('CryptoNews')
        assert medium_cred == 'medium'
        
        # Test niskiej wiarygodności
        low_cred = await analyst_agent._assess_source_credibility('UnknownBlog')
        assert low_cred == 'low'

    @pytest.mark.asyncio
    async def test_assess_temporal_relevance(self, analyst_agent):
        """Test: Ocena aktualności informacji"""
        
        # Test bardzo świeżej informacji (30 min temu)
        recent_time = datetime.now().replace(minute=datetime.now().minute-30).isoformat() + 'Z'
        relevance = await analyst_agent._assess_temporal_relevance(recent_time)
        assert relevance in ['very_fresh', 'fresh']
        
        # Test starej informacji
        old_relevance = await analyst_agent._assess_temporal_relevance('2025-01-01T10:00:00Z')
        assert old_relevance in ['recent', 'old']
        
        # Test nieprawidłowej daty
        invalid_relevance = await analyst_agent._assess_temporal_relevance('invalid-date')
        assert invalid_relevance == 'unknown'

    @pytest.mark.asyncio
    async def test_analyze_keyword_density(self, analyst_agent):
        """Test: Analiza gęstości słów kluczowych"""
        
        title = "Solana DeFi surge with new DEX"
        content = "Solana ecosystem grows with DeFi protocols. New yield farming opportunities on Raydium and Orca DEX platforms."
        
        keyword_analysis = await analyst_agent._analyze_keyword_density(title, content)
        
        # Sprawdź strukturę
        assert 'solana_mentions' in keyword_analysis
        assert 'defi_mentions' in keyword_analysis
        assert 'bullish_signals' in keyword_analysis
        assert 'bearish_signals' in keyword_analysis
        assert 'total_words' in keyword_analysis
        
        # Sprawdź wartości
        assert keyword_analysis['solana_mentions'] >= 2  # "solana" + "raydium"/"orca"
        assert keyword_analysis['defi_mentions'] >= 3    # "defi" + "yield" + "dex"
        assert keyword_analysis['bullish_signals'] >= 1  # "surge" + "grow"
        assert keyword_analysis['total_words'] > 0

    @pytest.mark.asyncio
    async def test_build_rag_prompt(self, analyst_agent, sample_opportunity):
        """Test: Budowanie promptu RAG"""
        
        article = sample_opportunity['source_article']
        analysis = sample_opportunity['analysis']
        
        # Przygotuj wzbogacony kontekst
        enriched_context = await analyst_agent._build_enriched_context(article, analysis)
        
        # Zbuduj prompt
        prompt = await analyst_agent._build_rag_prompt(enriched_context)
        
        # Sprawdź czy prompt zawiera kluczowe elementy
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
    async def test_full_rag_analysis_pipeline(self, analyst_agent, sample_opportunity):
        """Test: Pełny pipeline analizy RAG"""
        
        # Mock odpowiedzi AI z nowymi polami RAG
        mock_ai_response = {
            'sentiment_score': 0.8,
            'key_insight': 'Strong bullish signal for Solana DeFi growth',
            'confidence_score': 0.9,
            'risk_level': 'low',
            'trading_signal': 'buy',
            'time_horizon': 'short',
            'price_impact': 'positive',
            'market_context': 'bullish'
        }
        
        # Mock session z poprawnym async context manager
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={
            'response': json.dumps(mock_ai_response)
        })

        mock_context_manager = AsyncMock()
        mock_context_manager.__aenter__ = AsyncMock(return_value=mock_response)
        mock_context_manager.__aexit__ = AsyncMock(return_value=None)

        analyst_agent.session.post = AsyncMock(return_value=mock_context_manager)
        
        # Wykonaj analizę RAG
        result = await analyst_agent._analyze_with_ai(sample_opportunity)
        
        # Sprawdź wyniki
        assert result is not None
        assert result['sentiment_score'] == 0.8
        assert result['trading_signal'] == 'buy'
        assert result['price_impact'] == 'positive'
        assert result['market_context'] == 'bullish'
        
        # Sprawdź czy wszystkie nowe pola RAG są obecne
        required_rag_fields = [
            'sentiment_score', 'key_insight', 'confidence_score',
            'risk_level', 'trading_signal', 'time_horizon',
            'price_impact', 'market_context'
        ]
        
        for field in required_rag_fields:
            assert field in result, f"Brakuje pola RAG: {field}"

    @pytest.mark.asyncio
    async def test_rag_report_generation(self, analyst_agent, sample_opportunity):
        """Test: Generowanie raportu RAG"""
        
        # Mock AI analysis
        ai_analysis = {
            'sentiment_score': 0.75,
            'key_insight': 'Solana ecosystem expansion drives positive sentiment',
            'confidence_score': 0.85,
            'risk_level': 'medium',
            'trading_signal': 'buy',
            'time_horizon': 'medium',
            'price_impact': 'positive',
            'market_context': 'bullish'
        }
        
        # Mock session dla publikacji
        mock_response = AsyncMock()
        mock_response.status = 200
        analyst_agent.session.post.return_value.__aenter__.return_value = mock_response
        
        # Publikuj raport
        await analyst_agent._publish_analysis_report(sample_opportunity, ai_analysis)
        
        # Sprawdź czy POST został wywołany
        analyst_agent.session.post.assert_called_once()
        
        # Sprawdź argumenty wywołania
        call_args = analyst_agent.session.post.call_args
        published_report = call_args[1]['json']  # kwargs['json']
        
        # Sprawdź strukturę raportu RAG
        assert published_report['type'] == 'analysis_report'
        assert published_report['analyst_metadata']['analysis_type'] == 'real_time_rag'
        assert 'processing_pipeline' in published_report['analyst_metadata']
        assert 'rag_context' in published_report
        
        # Sprawdź summary z nowymi polami RAG
        summary = published_report['summary']
        assert summary['price_impact'] == 'positive'
        assert summary['market_context'] == 'bullish'
        assert summary['time_horizon'] == 'medium'

    @pytest.mark.asyncio
    async def test_rag_performance_metrics(self, analyst_agent):
        """Test: Metryki wydajności RAG"""
        
        # Symuluj wiele analiz RAG
        opportunities = []
        for i in range(10):
            opportunities.append({
                'source_article': {
                    'title': f'Solana news {i}',
                    'source': 'CoinDesk',
                    'published_date': '2025-06-09T10:00:00Z',
                    'content': f'Solana content {i}'
                },
                'analysis': {
                    'score': 20 + i,
                    'solana_related': True,
                    'reasons': ['test']
                }
            })
        
        # Mierz czas wykonania RAG
        import time
        start_time = time.time()
        
        # Wykonaj analizy kontekstu
        for opportunity in opportunities:
            enriched_context = await analyst_agent._build_enriched_context(
                opportunity['source_article'], 
                opportunity['analysis']
            )
            prompt = await analyst_agent._build_rag_prompt(enriched_context)
            
            # Sprawdź czy prompt został wygenerowany
            assert len(prompt) > 1000  # Prompt RAG powinien być obszerny
        
        end_time = time.time()
        duration = end_time - start_time
        
        # RAG powinien być szybki - poniżej 1 sekundy dla 10 analiz
        assert duration < 1.0, f"RAG zbyt wolny: {duration}s dla 10 analiz"
        
        print(f"✅ RAG Performance: {len(opportunities)} analiz w {duration:.3f}s ({len(opportunities)/duration:.1f} analiz/s)")

    @pytest.mark.asyncio
    async def test_rag_error_handling(self, analyst_agent):
        """Test: Obsługa błędów w systemie RAG"""
        
        # Test z nieprawidłowymi danymi
        invalid_opportunity = {
            'source_article': {
                'title': None,  # Nieprawidłowy tytuł
                'source': '',   # Puste źródło
                'published_date': 'invalid-date',
                'content': None
            },
            'analysis': {}  # Pusta analiza
        }
        
        # RAG powinien gracefully obsłużyć błędne dane
        try:
            enriched_context = await analyst_agent._build_enriched_context(
                invalid_opportunity['source_article'],
                invalid_opportunity['analysis']
            )
            
            # Sprawdź czy kontekst został utworzony mimo błędów
            assert enriched_context is not None
            assert 'article' in enriched_context
            assert 'context_metadata' in enriched_context
            
        except Exception as e:
            pytest.fail(f"RAG nie obsłużył błędnych danych: {e}")

    def test_rag_system_architecture(self):
        """Test: Architektura systemu RAG"""
        
        # Sprawdź czy wszystkie komponenty RAG są zaimplementowane
        analyst = AnalystAgent()
        
        # Sprawdź metody RAG
        assert hasattr(analyst, '_build_enriched_context')
        assert hasattr(analyst, '_build_rag_prompt')
        assert hasattr(analyst, '_assess_source_credibility')
        assert hasattr(analyst, '_assess_temporal_relevance')
        assert hasattr(analyst, '_analyze_keyword_density')
        
        # Sprawdź czy metody są async (RAG wymaga async)
        import inspect
        assert inspect.iscoroutinefunction(analyst._build_enriched_context)
        assert inspect.iscoroutinefunction(analyst._build_rag_prompt)
        assert inspect.iscoroutinefunction(analyst._analyze_with_ai)
        
        print("✅ Real-Time RAG Architecture: Wszystkie komponenty zaimplementowane!")

@pytest.mark.asyncio
async def test_rag_vs_traditional_comparison():
    """Test porównawczy: RAG vs tradycyjna analiza"""
    
    # Symuluj tradycyjną analizę (tylko tytuł)
    traditional_prompt = "Analyze: Solana price surge"
    
    # Symuluj analizę RAG (pełny kontekst)
    analyst = AnalystAgent()
    
    sample_article = {
        'title': 'Solana price surge',
        'source': 'CoinDesk',
        'published_date': '2025-06-09T10:00:00Z',
        'content': 'Detailed analysis of Solana ecosystem growth'
    }
    
    sample_analysis = {
        'score': 35,
        'solana_related': True,
        'reasons': ['Solana mentions: 2']
    }
    
    # Zbuduj kontekst RAG
    enriched_context = await analyst._build_enriched_context(sample_article, sample_analysis)
    rag_prompt = await analyst._build_rag_prompt(enriched_context)
    
    # Porównaj długość i jakość
    assert len(rag_prompt) > len(traditional_prompt) * 10
    assert 'Credibility:' in rag_prompt  # Sprawdź faktyczny format
    assert 'Relevance:' in rag_prompt    # Sprawdź faktyczny format
    assert 'KEYWORD ANALYSIS' in rag_prompt
    
    print(f"📊 RAG vs Traditional:")
    print(f"  Traditional prompt: {len(traditional_prompt)} znaków")
    print(f"  RAG prompt: {len(rag_prompt)} znaków")
    print(f"  RAG improvement: {len(rag_prompt)/len(traditional_prompt):.1f}x więcej kontekstu!")

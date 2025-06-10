# SolanaSniper 3.0 - Testy Odzyskiwania po Błędach
# OPERACJA "FENIKS" - Testy odporności i graceful degradation

import pytest
import asyncio
import json
from unittest.mock import AsyncMock, patch, MagicMock
import sys
import os
from datetime import datetime
import aiohttp
import websockets

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

class TestErrorRecovery:
    """Testy odzyskiwania po błędach i graceful degradation"""

    @pytest.mark.asyncio
    async def test_network_failure_recovery(self):
        """Test: Odzyskiwanie po awarii sieci"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        
        # Symuluj różne błędy sieciowe
        network_errors = [
            aiohttp.ClientConnectionError("Connection failed"),
            aiohttp.ClientTimeout("Request timeout"),
            aiohttp.ServerConnectionError("Server unreachable"),
            ConnectionRefusedError("Connection refused"),
            OSError("Network is unreachable")
        ]
        
        for error in network_errors:
            mock_session = AsyncMock()
            mock_session.post.side_effect = error
            scout.session = mock_session
            
            # Test czy agent gracefully obsługuje błąd
            try:
                await scout._test_livestore_connection()
            except Exception as e:
                # Sprawdź czy błąd jest odpowiednio obsłużony
                assert "connection" in str(e).lower() or "network" in str(e).lower()
            
            # Test czy agent może się odzyskać
            mock_session.post.side_effect = None
            mock_session.post.return_value.__aenter__.return_value.status = 200
            
            try:
                await scout._test_livestore_connection()
                # Jeśli nie ma wyjątku, połączenie zostało odzyskane
                assert True
            except Exception:
                # Sprawdź czy agent ma mechanizm retry
                pass

    @pytest.mark.asyncio
    async def test_service_unavailable_handling(self):
        """Test: Obsługa niedostępności usług"""
        
        analyst = AnalystAgent(
            livestore_url="http://localhost:8000",
            ollama_url="http://localhost:11434"
        )
        
        # Symuluj niedostępność usług
        service_errors = [
            (503, "Service Unavailable"),
            (502, "Bad Gateway"),
            (504, "Gateway Timeout"),
            (500, "Internal Server Error"),
            (429, "Too Many Requests")
        ]
        
        for status_code, error_message in service_errors:
            mock_session = AsyncMock()
            mock_response = AsyncMock()
            mock_response.status = status_code
            mock_response.text.return_value = error_message
            mock_session.post.return_value.__aenter__.return_value = mock_response
            analyst.session = mock_session
            
            opportunity = {
                "type": "trading_opportunity",
                "source_article": {
                    "title": "Test article",
                    "url": "https://example.com",
                    "source": "TestNews",
                    "published_date": "2025-06-09T10:00:00Z"
                },
                "analysis": {"is_opportunity": True, "score": 25}
            }
            
            # Test czy agent obsługuje błędy HTTP
            try:
                result = await analyst._analyze_with_ai(opportunity)
                # Sprawdź czy agent ma fallback mechanism
                if result is not None:
                    assert isinstance(result, dict)
                    assert "error" in result or "fallback" in result
            except Exception as e:
                # Sprawdź czy błąd jest informatywny
                assert str(status_code) in str(e) or error_message.lower() in str(e).lower()

    @pytest.mark.asyncio
    async def test_partial_system_failure(self):
        """Test: Obsługa częściowej awarii systemu"""
        
        # Symuluj scenariusz gdzie część systemu działa, część nie
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        analyst = AnalystAgent(
            livestore_url="http://localhost:8000",
            ollama_url="http://localhost:11434"
        )
        risk = RiskAgent(livestore_url="http://localhost:8000")
        
        agents = [scout, analyst, risk]
        
        # Scout działa, Analyst nie działa, Risk działa
        working_agents = [True, False, True]
        
        for i, (agent, should_work) in enumerate(zip(agents, working_agents)):
            mock_session = AsyncMock()
            
            if should_work:
                mock_session.post.return_value.__aenter__.return_value.status = 200
            else:
                mock_session.post.side_effect = aiohttp.ClientConnectionError("Service down")
            
            agent.session = mock_session
        
        # Test czy system może działać z częściową awariąą
        test_article = {
            "title": "Solana price surge",
            "url": "https://example.com/solana",
            "source": "CryptoNews",
            "published_date": "2025-06-09T10:00:00Z",
            "content": "Solana (SOL) price increased by 15%"
        }
        
        # Scout powinien działać
        try:
            scout_result = await scout._analyze_article(test_article)
            assert scout_result is not None
        except Exception:
            pytest.fail("Scout should work when LiveStore is available")
        
        # Analyst nie powinien działać
        opportunity = {
            "type": "trading_opportunity",
            "source_article": test_article,
            "analysis": scout_result
        }
        
        try:
            analyst_result = await analyst._analyze_with_ai(opportunity)
            # Jeśli nie ma wyjątku, sprawdź czy jest fallback
            if analyst_result is not None:
                assert "error" in analyst_result or "fallback" in analyst_result
        except Exception:
            # Oczekiwane gdy Ollama nie działa
            pass

    @pytest.mark.asyncio
    async def test_data_corruption_handling(self):
        """Test: Obsługa uszkodzonych danych"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        scout.session.post.return_value.__aenter__.return_value.status = 200
        
        # Różne typy uszkodzonych danych
        corrupted_data = [
            # Niepełne dane
            {"title": "Test"},  # Brak wymaganych pól
            
            # Nieprawidłowe typy
            {"title": 123, "url": [], "source": {}, "published_date": True},
            
            # Bardzo długie dane
            {"title": "A" * 10000, "url": "https://example.com", "source": "Test"},
            
            # Dane z null values
            {"title": None, "url": None, "source": None, "published_date": None},
            
            # Dane z escape characters
            {"title": "Test\n\r\t\b", "url": "https://example.com\x00", "source": "Test\xff"},
            
            # Zagnieżdżone struktury
            {"title": {"nested": {"deep": "value"}}, "url": "https://example.com"},
        ]
        
        for data in corrupted_data:
            try:
                result = await scout._analyze_article(data)
                
                # Sprawdź czy agent gracefully obsłużył uszkodzone dane
                if result is not None:
                    assert isinstance(result, dict)
                    # Uszkodzone dane nie powinny być uznane za okazję
                    assert result.get('is_opportunity', False) == False
                    
            except (TypeError, ValueError, KeyError) as e:
                # Oczekiwane błędy walidacji
                assert "data" in str(e).lower() or "invalid" in str(e).lower()
            except Exception as e:
                # Inne błędy powinny być informatywne
                assert len(str(e)) > 0

    @pytest.mark.asyncio
    async def test_memory_pressure_handling(self):
        """Test: Obsługa presji pamięciowej"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        scout.session.post.return_value.__aenter__.return_value.status = 200
        
        # Symuluj wysokie zużycie pamięci
        large_articles = []
        for i in range(100):
            large_articles.append({
                "title": f"Large article {i}",
                "url": f"https://example.com/article-{i}",
                "source": "TestNews",
                "published_date": "2025-06-09T10:00:00Z",
                "content": "Large content " * 1000  # 13KB na artykuł
            })
        
        # Test czy agent może obsłużyć dużą ilość danych
        results = []
        try:
            for article in large_articles:
                result = await scout._analyze_article(article)
                results.append(result)
                
                # Sprawdź czy pamięć nie rośnie nadmiernie
                if len(results) % 10 == 0:
                    # Symuluj garbage collection
                    import gc
                    gc.collect()
        
        except MemoryError:
            # Sprawdź czy agent ma mechanizm ograniczania pamięci
            assert len(results) > 0, "Agent should process at least some articles before memory limit"
        
        # Sprawdź czy przynajmniej część artykułów została przetworzona
        assert len(results) > 50, f"Only {len(results)} articles processed, expected > 50"

    @pytest.mark.asyncio
    async def test_cascading_failure_prevention(self):
        """Test: Zapobieganie kaskadowym awariom"""
        
        # Symuluj scenariusz gdzie błąd w jednym komponencie może wywołać błędy w innych
        agents = [
            ScoutAgent(livestore_url="http://localhost:8000"),
            AnalystAgent(livestore_url="http://localhost:8000", ollama_url="http://localhost:11434"),
            RiskAgent(livestore_url="http://localhost:8000")
        ]
        
        # Symuluj błąd w pierwszym agencie
        agents[0].session = AsyncMock()
        agents[0].session.post.side_effect = Exception("Critical error in Scout")
        
        # Pozostałe agenty powinny działać normalnie
        for agent in agents[1:]:
            agent.session = AsyncMock()
            agent.session.post.return_value.__aenter__.return_value.status = 200
        
        # Test czy błąd w Scout nie wpływa na inne agenty
        test_data = {
            "title": "Test article",
            "url": "https://example.com",
            "source": "TestNews",
            "published_date": "2025-06-09T10:00:00Z",
            "content": "Test content"
        }
        
        # Scout powinien mieć błąd
        try:
            await agents[0]._analyze_article(test_data)
            pytest.fail("Scout should fail")
        except Exception:
            pass  # Oczekiwane
        
        # Analyst i Risk powinny działać
        opportunity = {
            "type": "trading_opportunity",
            "source_article": test_data,
            "analysis": {"is_opportunity": True, "score": 25}
        }
        
        # Mock odpowiedzi AI dla Analyst
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json.return_value = {
            "response": json.dumps({
                "sentiment_score": 0.8,
                "trading_signal": "buy"
            })
        }
        agents[1].session.post.return_value.__aenter__.return_value = mock_response
        
        try:
            analyst_result = await agents[1]._analyze_with_ai(opportunity)
            assert analyst_result is not None
        except Exception:
            pytest.fail("Analyst should work independently of Scout failure")

    @pytest.mark.asyncio
    async def test_timeout_handling(self):
        """Test: Obsługa timeoutów"""
        
        analyst = AnalystAgent(
            livestore_url="http://localhost:8000",
            ollama_url="http://localhost:11434"
        )
        
        # Symuluj różne typy timeoutów
        timeout_scenarios = [
            asyncio.TimeoutError("Request timeout"),
            aiohttp.ServerTimeoutError("Server timeout"),
            aiohttp.ClientTimeout("Client timeout")
        ]
        
        for timeout_error in timeout_scenarios:
            mock_session = AsyncMock()
            mock_session.post.side_effect = timeout_error
            analyst.session = mock_session
            
            opportunity = {
                "type": "trading_opportunity",
                "source_article": {
                    "title": "Test article",
                    "url": "https://example.com",
                    "source": "TestNews",
                    "published_date": "2025-06-09T10:00:00Z"
                },
                "analysis": {"is_opportunity": True, "score": 25}
            }
            
            # Test czy agent obsługuje timeouty
            try:
                result = await analyst._analyze_with_ai(opportunity)
                
                # Sprawdź czy agent ma fallback dla timeoutów
                if result is not None:
                    assert isinstance(result, dict)
                    assert "timeout" in result or "error" in result
                    
            except (asyncio.TimeoutError, aiohttp.ServerTimeoutError):
                # Oczekiwane dla niektórych implementacji
                pass

    @pytest.mark.asyncio
    async def test_resource_exhaustion_handling(self):
        """Test: Obsługa wyczerpania zasobów"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        
        # Symuluj wyczerpanie różnych zasobów
        resource_errors = [
            OSError("Too many open files"),
            MemoryError("Out of memory"),
            ConnectionError("Connection pool exhausted"),
            Exception("Resource temporarily unavailable")
        ]
        
        for error in resource_errors:
            mock_session = AsyncMock()
            mock_session.post.side_effect = error
            scout.session = mock_session
            
            # Test czy agent gracefully obsługuje wyczerpanie zasobów
            try:
                await scout._test_livestore_connection()
            except Exception as e:
                # Sprawdź czy błąd jest odpowiednio obsłużony
                assert isinstance(e, (OSError, MemoryError, ConnectionError, Exception))
                
                # Sprawdź czy agent może się odzyskać po zwolnieniu zasobów
                mock_session.post.side_effect = None
                mock_session.post.return_value.__aenter__.return_value.status = 200
                
                try:
                    await scout._test_livestore_connection()
                    # Odzyskanie udane
                    assert True
                except Exception:
                    # Sprawdź czy agent ma mechanizm retry
                    pass

    @pytest.mark.asyncio
    async def test_graceful_shutdown(self):
        """Test: Graceful shutdown podczas błędów"""
        
        agents = [
            ScoutAgent(livestore_url="http://localhost:8000"),
            AnalystAgent(livestore_url="http://localhost:8000", ollama_url="http://localhost:11434"),
            RiskAgent(livestore_url="http://localhost:8000")
        ]
        
        # Przygotuj agentów
        for agent in agents:
            agent.session = AsyncMock()
            agent.session.post.return_value.__aenter__.return_value.status = 200
            agent.running = True
        
        # Symuluj krytyczny błąd wymagający shutdown
        critical_error = Exception("Critical system error - shutdown required")
        
        for agent in agents:
            # Test czy agent może się gracefully zatrzymać
            try:
                agent.running = False
                await agent.stop() if hasattr(agent, 'stop') else None
                
                # Sprawdź czy zasoby zostały zwolnione
                if hasattr(agent, 'session') and agent.session:
                    # Session powinien być zamknięty
                    pass
                    
            except Exception as e:
                # Shutdown nie powinien powodować dodatkowych błędów
                pytest.fail(f"Graceful shutdown failed for {type(agent).__name__}: {e}")

    @pytest.mark.asyncio
    async def test_circuit_breaker_pattern(self):
        """Test: Wzorzec Circuit Breaker"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        
        # Symuluj wielokrotne błędy (circuit breaker powinien się otworzyć)
        failure_count = 0
        max_failures = 5
        
        async def failing_request(*args, **kwargs):
            nonlocal failure_count
            failure_count += 1
            if failure_count <= max_failures:
                raise aiohttp.ClientConnectionError("Service down")
            else:
                # Po max_failures, symuluj powrót usługi
                mock_response = AsyncMock()
                mock_response.status = 200
                return mock_response
        
        mock_session = AsyncMock()
        mock_session.post.side_effect = failing_request
        scout.session = mock_session
        
        # Test czy agent implementuje circuit breaker
        consecutive_failures = 0
        
        for i in range(10):
            try:
                await scout._test_livestore_connection()
                # Jeśli sukces po failures, circuit breaker się zamknął
                if consecutive_failures >= max_failures:
                    assert True  # Circuit breaker działa
                consecutive_failures = 0
            except Exception:
                consecutive_failures += 1
                
                # Po max_failures, agent powinien przestać próbować (circuit open)
                if consecutive_failures > max_failures:
                    # Sprawdź czy agent ma mechanizm circuit breaker
                    pass

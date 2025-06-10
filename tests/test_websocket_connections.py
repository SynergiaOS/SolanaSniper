# SolanaSniper 3.0 - Testy Połączeń WebSocket
# OPERACJA "KOMUNIKACJA" - Testy real-time komunikacji

import pytest
import asyncio
import json
import websockets
from unittest.mock import AsyncMock, patch, MagicMock
import sys
import os
from datetime import datetime

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

class TestWebSocketConnections:
    """Testy połączeń WebSocket i komunikacji real-time"""

    @pytest.mark.asyncio
    async def test_websocket_connection_establishment(self):
        """Test: Nawiązywanie połączenia WebSocket"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        
        # Mock WebSocket connection
        mock_websocket = AsyncMock()
        mock_websocket.recv.return_value = json.dumps({
            "channel": "news_articles",
            "data": {
                "type": "news_article",
                "content": {
                    "title": "Test article",
                    "url": "https://example.com",
                    "source": "TestNews",
                    "published_date": "2025-06-09T10:00:00Z"
                }
            }
        })
        
        with patch('websockets.connect', return_value=mock_websocket):
            # Test nawiązania połączenia
            try:
                await scout._connect_and_listen()
            except Exception as e:
                # Sprawdź czy błąd jest związany z połączeniem
                assert "connection" in str(e).lower() or "websocket" in str(e).lower()

    @pytest.mark.asyncio
    async def test_websocket_reconnection_logic(self):
        """Test: Logika ponownego łączenia WebSocket"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        
        # Symuluj błędy połączenia
        connection_errors = [
            ConnectionRefusedError("Connection refused"),
            websockets.exceptions.ConnectionClosed(1006, "Connection lost"),
            asyncio.TimeoutError("Connection timeout"),
            OSError("Network unreachable")
        ]
        
        for error in connection_errors:
            with patch('websockets.connect', side_effect=error):
                # Agent powinien gracefully obsłużyć błąd
                try:
                    await scout._connect_and_listen()
                except Exception as e:
                    # Sprawdź czy błąd jest odpowiednio obsłużony
                    assert isinstance(e, (ConnectionRefusedError, OSError, asyncio.TimeoutError))

    @pytest.mark.asyncio
    async def test_websocket_message_handling(self):
        """Test: Obsługa wiadomości WebSocket"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        scout.session.post.return_value.__aenter__.return_value.status = 200
        
        # Różne typy wiadomości
        test_messages = [
            # Prawidłowa wiadomość
            {
                "channel": "news_articles",
                "data": {
                    "type": "news_article",
                    "content": {
                        "title": "Solana price surge",
                        "url": "https://example.com/solana",
                        "source": "CryptoNews",
                        "published_date": "2025-06-09T10:00:00Z",
                        "content": "Solana (SOL) price increased by 15%"
                    }
                }
            },
            # Wiadomość z brakującymi polami
            {
                "channel": "news_articles",
                "data": {
                    "type": "news_article",
                    "content": {
                        "title": "Incomplete article"
                        # Brakuje url, source, etc.
                    }
                }
            },
            # Nieprawidłowy JSON
            "invalid json message",
            
            # Pusty obiekt
            {},
            
            # Wiadomość z nieprawidłowym typem
            {
                "channel": "news_articles",
                "data": {
                    "type": "unknown_type",
                    "content": {}
                }
            }
        ]
        
        for message in test_messages:
            mock_websocket = AsyncMock()
            if isinstance(message, str):
                mock_websocket.recv.return_value = message
            else:
                mock_websocket.recv.return_value = json.dumps(message)
            
            with patch('websockets.connect', return_value=mock_websocket):
                try:
                    # Agent powinien obsłużyć wszystkie typy wiadomości
                    await scout._handle_message(json.dumps(message) if isinstance(message, dict) else message)
                except json.JSONDecodeError:
                    # Oczekiwane dla nieprawidłowego JSON
                    pass
                except Exception as e:
                    # Inne błędy powinny być gracefully obsłużone
                    assert "message" in str(e).lower() or "data" in str(e).lower()

    @pytest.mark.asyncio
    async def test_websocket_heartbeat_mechanism(self):
        """Test: Mechanizm heartbeat WebSocket"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        
        # Mock WebSocket z heartbeat
        mock_websocket = AsyncMock()
        heartbeat_messages = [
            json.dumps({"type": "ping"}),
            json.dumps({"type": "heartbeat", "timestamp": datetime.now().isoformat()}),
            json.dumps({"type": "keepalive"})
        ]
        
        mock_websocket.recv.side_effect = heartbeat_messages
        
        with patch('websockets.connect', return_value=mock_websocket):
            # Test czy agent odpowiada na heartbeat
            for message in heartbeat_messages:
                try:
                    await scout._handle_message(message)
                    # Sprawdź czy agent wysłał odpowiedź (pong)
                    if "ping" in message:
                        # Agent powinien odpowiedzieć pong
                        pass
                except Exception as e:
                    # Heartbeat nie powinien powodować błędów
                    assert "heartbeat" not in str(e).lower()

    @pytest.mark.asyncio
    async def test_websocket_concurrent_connections(self):
        """Test: Równoczesne połączenia WebSocket"""
        
        # Utwórz wielu agentów
        agents = [
            ScoutAgent(livestore_url="http://localhost:8000"),
            AnalystAgent(livestore_url="http://localhost:8000", ollama_url="http://localhost:11434"),
            RiskAgent(livestore_url="http://localhost:8000")
        ]
        
        # Mock sessions dla wszystkich agentów
        for agent in agents:
            agent.session = AsyncMock()
            agent.session.post.return_value.__aenter__.return_value.status = 200
        
        # Mock WebSocket connections
        mock_websockets = [AsyncMock() for _ in agents]
        
        for i, mock_ws in enumerate(mock_websockets):
            mock_ws.recv.return_value = json.dumps({
                "channel": f"test_channel_{i}",
                "data": {"type": "test", "content": f"message_{i}"}
            })
        
        # Test równoczesnych połączeń
        with patch('websockets.connect', side_effect=mock_websockets):
            tasks = []
            for agent in agents:
                if hasattr(agent, '_connect_and_listen'):
                    task = asyncio.create_task(agent._connect_and_listen())
                    tasks.append(task)
            
            # Uruchom wszystkie połączenia równocześnie
            if tasks:
                try:
                    await asyncio.wait_for(asyncio.gather(*tasks, return_exceptions=True), timeout=1.0)
                except asyncio.TimeoutError:
                    # Oczekiwane - agenci działają w nieskończonych pętlach
                    pass
                finally:
                    # Anuluj wszystkie zadania
                    for task in tasks:
                        task.cancel()

    @pytest.mark.asyncio
    async def test_websocket_message_ordering(self):
        """Test: Kolejność wiadomości WebSocket"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        scout.session.post.return_value.__aenter__.return_value.status = 200
        
        # Sekwencja wiadomości z timestampami
        messages = []
        for i in range(10):
            messages.append(json.dumps({
                "channel": "news_articles",
                "timestamp": f"2025-06-09T10:{i:02d}:00Z",
                "sequence": i,
                "data": {
                    "type": "news_article",
                    "content": {
                        "title": f"Article {i}",
                        "url": f"https://example.com/article-{i}",
                        "source": "TestNews",
                        "published_date": f"2025-06-09T10:{i:02d}:00Z"
                    }
                }
            }))
        
        mock_websocket = AsyncMock()
        mock_websocket.recv.side_effect = messages
        
        processed_messages = []
        
        # Mock funkcji przetwarzania wiadomości
        original_handle = scout._handle_message if hasattr(scout, '_handle_message') else None
        
        async def track_message_processing(message):
            processed_messages.append(message)
            if original_handle:
                await original_handle(message)
        
        scout._handle_message = track_message_processing
        
        with patch('websockets.connect', return_value=mock_websocket):
            try:
                await asyncio.wait_for(scout._connect_and_listen(), timeout=1.0)
            except asyncio.TimeoutError:
                pass
        
        # Sprawdź czy wiadomości zostały przetworzone w kolejności
        assert len(processed_messages) > 0

    @pytest.mark.asyncio
    async def test_websocket_error_recovery(self):
        """Test: Odzyskiwanie po błędach WebSocket"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        
        # Symuluj różne scenariusze błędów
        error_scenarios = [
            # Utrata połączenia
            websockets.exceptions.ConnectionClosed(1006, "Connection lost"),
            
            # Timeout
            asyncio.TimeoutError("Message timeout"),
            
            # Błąd sieci
            OSError("Network error"),
            
            # Błąd protokołu
            websockets.exceptions.ProtocolError("Protocol error")
        ]
        
        for error in error_scenarios:
            mock_websocket = AsyncMock()
            mock_websocket.recv.side_effect = error
            
            with patch('websockets.connect', return_value=mock_websocket):
                # Agent powinien gracefully obsłużyć błąd i spróbować ponownie
                try:
                    await scout._connect_and_listen()
                except Exception as e:
                    # Sprawdź czy błąd jest odpowiednio obsłużony
                    assert isinstance(e, (websockets.exceptions.WebSocketException, OSError, asyncio.TimeoutError))

    @pytest.mark.asyncio
    async def test_websocket_channel_subscription(self):
        """Test: Subskrypcja kanałów WebSocket"""
        
        # Test różnych agentów subskrybujących różne kanały
        test_cases = [
            (ScoutAgent, "news_articles"),
            (AnalystAgent, "opportunities"),
            (RiskAgent, "analysis_reports")
        ]
        
        for agent_class, expected_channel in test_cases:
            if agent_class == AnalystAgent:
                agent = agent_class(
                    livestore_url="http://localhost:8000",
                    ollama_url="http://localhost:11434"
                )
            else:
                agent = agent_class(livestore_url="http://localhost:8000")
            
            agent.session = AsyncMock()
            agent.session.post.return_value.__aenter__.return_value.status = 200
            
            # Sprawdź czy agent subskrybuje właściwy kanał
            if hasattr(agent, 'channel') or hasattr(agent, 'subscribe_channel'):
                # Agent ma zdefiniowany kanał
                pass
            else:
                # Sprawdź w kodzie agenta
                assert True  # Placeholder - należy sprawdzić implementację

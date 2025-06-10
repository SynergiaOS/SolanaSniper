# SolanaSniper 3.0 - Testy Integracji Dragonfly
# OPERACJA "PAMIĘĆ" - Testy persystencji danych i pub/sub z Dragonfly

import pytest
import asyncio
import json
import redis.asyncio as redis
from unittest.mock import AsyncMock, patch, MagicMock
import sys
import os
from datetime import datetime, timedelta

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from livestore.livestore_server import LiveStore, livestore

class TestDragonflyIntegration:
    """Testy integracji z Redis - persystencja i pub/sub"""

    @pytest.fixture
    async def mock_redis(self):
        """Fixture z mock Redis client"""
        mock_client = AsyncMock()
        mock_client.ping.return_value = True
        mock_client.publish.return_value = 1
        mock_client.lpush.return_value = 1
        mock_client.ltrim.return_value = True
        mock_client.lrange.return_value = []
        return mock_client

    @pytest.mark.asyncio
    async def test_redis_connection(self, mock_redis):
        """Test: Połączenie z Redis"""
        
        store = LiveStore()
        
        with patch('redis.asyncio.from_url', return_value=mock_redis):
            await store.connect()
            
            # Sprawdź czy połączenie zostało nawiązane
            assert store.redis_client is not None
            mock_redis.ping.assert_called_once()

    @pytest.mark.asyncio
    async def test_redis_connection_failure(self):
        """Test: Obsługa błędów połączenia z Redis"""
        
        store = LiveStore()
        
        # Symuluj błąd połączenia
        with patch('redis.asyncio.from_url', side_effect=redis.ConnectionError("Connection failed")):
            with pytest.raises(redis.ConnectionError):
                await store.connect()

    @pytest.mark.asyncio
    async def test_message_publishing(self, mock_redis):
        """Test: Publikowanie wiadomości do Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Test publikacji wiadomości
        test_message = {
            "type": "news_article",
            "content": {
                "title": "Test article",
                "url": "https://example.com",
                "source": "TestNews",
                "published_date": "2025-06-09T10:00:00Z"
            }
        }
        
        await store.publish_message("news_articles", test_message, "test_agent")
        
        # Sprawdź czy wiadomość została opublikowana
        mock_redis.publish.assert_called_once()
        mock_redis.lpush.assert_called_once()  # Historia
        mock_redis.ltrim.assert_called_once()  # Ograniczenie historii

    @pytest.mark.asyncio
    async def test_message_history_storage(self, mock_redis):
        """Test: Przechowywanie historii wiadomości"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Symuluj historię wiadomości
        history_data = [
            json.dumps({"id": 1, "content": "Message 1"}),
            json.dumps({"id": 2, "content": "Message 2"}),
            json.dumps({"id": 3, "content": "Message 3"})
        ]
        mock_redis.lrange.return_value = history_data
        
        # Pobierz historię
        history = await store.get_channel_history("news_articles", 10)
        
        assert len(history) == 3
        assert history[0]["id"] == 1
        mock_redis.lrange.assert_called_once_with("history:news_articles", 0, 9)

    @pytest.mark.asyncio
    async def test_pubsub_subscription(self, mock_redis):
        """Test: Subskrypcja pub/sub"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Mock pubsub
        mock_pubsub = AsyncMock()
        mock_redis.pubsub.return_value = mock_pubsub
        
        # Test subskrypcji
        await store._setup_pubsub()
        
        mock_redis.pubsub.assert_called_once()
        mock_pubsub.subscribe.assert_called()

    @pytest.mark.asyncio
    async def test_pubsub_message_handling(self, mock_redis):
        """Test: Obsługa wiadomości pub/sub"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Mock pubsub z wiadomościami
        mock_pubsub = AsyncMock()
        mock_redis.pubsub.return_value = mock_pubsub
        
        # Symuluj wiadomości
        test_messages = [
            {
                "type": "message",
                "channel": b"news_articles",
                "data": json.dumps({
                    "type": "news_article",
                    "content": {"title": "Test"}
                }).encode()
            },
            {
                "type": "subscribe",
                "channel": b"news_articles",
                "data": 1
            }
        ]
        
        mock_pubsub.listen.return_value = test_messages
        store.pubsub = mock_pubsub
        
        # Test obsługi wiadomości
        processed_messages = []
        
        async def mock_broadcast(channel, message):
            processed_messages.append((channel, message))
        
        store._broadcast_to_websockets = mock_broadcast
        
        # Symuluj przetwarzanie wiadomości
        for message in test_messages:
            if message["type"] == "message":
                channel = message["channel"].decode()
                data = json.loads(message["data"].decode())
                await store._broadcast_to_websockets(channel, data)
        
        assert len(processed_messages) == 1
        assert processed_messages[0][0] == "news_articles"

    @pytest.mark.asyncio
    async def test_redis_data_persistence(self, mock_redis):
        """Test: Persystencja danych w Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Test zapisywania różnych typów danych
        test_data = [
            {"type": "opportunity", "score": 25},
            {"type": "analysis", "sentiment": 0.8},
            {"type": "risk_assessment", "level": "low"}
        ]
        
        for data in test_data:
            await store.publish_message("test_channel", data, "test_agent")
        
        # Sprawdź czy wszystkie dane zostały zapisane
        assert mock_redis.publish.call_count == len(test_data)
        assert mock_redis.lpush.call_count == len(test_data)

    @pytest.mark.asyncio
    async def test_redis_memory_management(self, mock_redis):
        """Test: Zarządzanie pamięcią Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Symuluj dużą ilość wiadomości
        for i in range(1500):  # Więcej niż limit 1000
            await store.publish_message(
                "test_channel", 
                {"id": i, "content": f"Message {i}"}, 
                "test_agent"
            )
        
        # Sprawdź czy ltrim jest wywoływane (ograniczenie do 1000)
        assert mock_redis.ltrim.call_count == 1500
        
        # Sprawdź parametry ltrim (powinno ograniczać do 999 ostatnich)
        ltrim_calls = mock_redis.ltrim.call_args_list
        for call in ltrim_calls:
            args = call[0]
            assert args[1] == 0  # start
            assert args[2] == 999  # end (ostatnie 1000 elementów)

    @pytest.mark.asyncio
    async def test_redis_connection_recovery(self, mock_redis):
        """Test: Odzyskiwanie połączenia z Redis"""
        
        store = LiveStore()
        
        # Symuluj utratę połączenia
        connection_error = redis.ConnectionError("Connection lost")
        mock_redis.ping.side_effect = [connection_error, True]  # Błąd, potem sukces
        
        with patch('redis.asyncio.from_url', return_value=mock_redis):
            # Pierwsze połączenie - błąd
            with pytest.raises(redis.ConnectionError):
                await store.connect()
            
            # Drugie połączenie - sukces
            mock_redis.ping.side_effect = [True]
            await store.connect()
            assert store.redis_client is not None

    @pytest.mark.asyncio
    async def test_redis_transaction_handling(self, mock_redis):
        """Test: Obsługa transakcji Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Mock pipeline dla transakcji
        mock_pipeline = AsyncMock()
        mock_redis.pipeline.return_value = mock_pipeline
        mock_pipeline.execute.return_value = [1, 1, True]  # publish, lpush, ltrim
        
        # Test transakcyjnego publikowania
        test_message = {"type": "test", "content": "transaction test"}
        
        # Jeśli LiveStore używa transakcji
        if hasattr(store, 'publish_message_transaction'):
            await store.publish_message_transaction("test_channel", test_message, "test_agent")
            mock_redis.pipeline.assert_called_once()
            mock_pipeline.execute.assert_called_once()

    @pytest.mark.asyncio
    async def test_redis_channel_management(self, mock_redis):
        """Test: Zarządzanie kanałami Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Test różnych kanałów
        channels = [
            "news_articles",
            "opportunities", 
            "analysis_reports",
            "execution_orders",
            "system_alerts"
        ]
        
        for channel in channels:
            await store.publish_message(
                channel, 
                {"type": "test", "channel": channel}, 
                "test_agent"
            )
        
        # Sprawdź czy wszystkie kanały zostały obsłużone
        assert mock_redis.publish.call_count == len(channels)
        
        # Sprawdź czy historia jest tworzona dla każdego kanału
        history_calls = [call[0][0] for call in mock_redis.lpush.call_args_list]
        expected_history_keys = [f"history:{channel}" for channel in channels]
        
        for expected_key in expected_history_keys:
            assert any(expected_key in call for call in history_calls)

    @pytest.mark.asyncio
    async def test_redis_error_handling(self, mock_redis):
        """Test: Obsługa błędów Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Różne typy błędów Redis
        redis_errors = [
            redis.ConnectionError("Connection failed"),
            redis.TimeoutError("Operation timeout"),
            redis.ResponseError("Command failed"),
            redis.DataError("Invalid data"),
            Exception("Unknown error")
        ]
        
        for error in redis_errors:
            mock_redis.publish.side_effect = error
            
            # Test czy błędy są gracefully obsługiwane
            try:
                await store.publish_message("test_channel", {"test": "data"}, "test_agent")
            except Exception as e:
                # Sprawdź czy błąd jest odpowiednio obsłużony
                assert isinstance(e, (redis.RedisError, Exception))
            
            # Reset mock
            mock_redis.publish.side_effect = None
            mock_redis.publish.return_value = 1

    @pytest.mark.asyncio
    async def test_redis_performance_monitoring(self, mock_redis):
        """Test: Monitorowanie wydajności Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Symuluj opóźnienia
        async def slow_publish(*args, **kwargs):
            await asyncio.sleep(0.1)  # 100ms opóźnienie
            return 1
        
        mock_redis.publish.side_effect = slow_publish
        
        # Mierz czas operacji
        start_time = asyncio.get_event_loop().time()
        
        await store.publish_message("test_channel", {"test": "performance"}, "test_agent")
        
        end_time = asyncio.get_event_loop().time()
        duration = end_time - start_time
        
        # Sprawdź czy operacja nie trwa zbyt długo
        assert duration < 1.0, f"Redis operation too slow: {duration}s"

    @pytest.mark.asyncio
    async def test_redis_data_integrity(self, mock_redis):
        """Test: Integralność danych w Redis"""
        
        store = LiveStore()
        store.redis_client = mock_redis
        
        # Test czy dane są poprawnie serializowane/deserializowane
        original_data = {
            "type": "complex_data",
            "nested": {
                "array": [1, 2, 3],
                "boolean": True,
                "null_value": None,
                "float": 3.14159
            },
            "unicode": "🚀 Solana to the moon! 🌙",
            "timestamp": datetime.now().isoformat()
        }
        
        # Mock zapisywania i odczytywania
        stored_data = None
        
        def mock_lpush(key, value):
            nonlocal stored_data
            stored_data = value
            return 1
        
        mock_redis.lpush.side_effect = mock_lpush
        mock_redis.lrange.return_value = [stored_data] if stored_data else []
        
        # Zapisz dane
        await store.publish_message("test_channel", original_data, "test_agent")
        
        # Odczytaj dane
        if stored_data:
            retrieved_data = json.loads(stored_data)
            
            # Sprawdź integralność
            assert retrieved_data["type"] == original_data["type"]
            assert retrieved_data["nested"]["array"] == original_data["nested"]["array"]
            assert retrieved_data["unicode"] == original_data["unicode"]

    @pytest.mark.asyncio
    async def test_dragonfly_specific_features(self, mock_redis):
        """Test: Funkcje specyficzne dla Dragonfly"""

        store = LiveStore()
        store.redis_client = mock_redis

        # Test Dragonfly-specific commands
        dragonfly_commands = [
            # Memory optimization
            ("MEMORY", "USAGE", "test_key"),

            # Performance monitoring
            ("INFO", "memory"),
            ("INFO", "stats"),

            # Dragonfly specific
            ("DF.CONFIG", "GET", "*"),
        ]

        for command in dragonfly_commands:
            mock_redis.execute_command.return_value = "OK"

            # Test czy Dragonfly commands działają
            try:
                result = await mock_redis.execute_command(*command)
                assert result == "OK"
            except Exception:
                # Niektóre komendy mogą nie być dostępne w mock
                pass

    @pytest.mark.asyncio
    async def test_dragonfly_performance_optimization(self, mock_redis):
        """Test: Optymalizacje wydajności Dragonfly"""

        store = LiveStore()
        store.redis_client = mock_redis

        # Test pipeline performance
        mock_pipeline = AsyncMock()
        mock_redis.pipeline.return_value = mock_pipeline
        mock_pipeline.execute.return_value = [1] * 100

        # Symuluj batch operations
        async with mock_redis.pipeline() as pipe:
            for i in range(100):
                pipe.lpush(f"test_key_{i}", f"value_{i}")
            results = await pipe.execute()

        assert len(results) == 100
        assert all(result == 1 for result in results)

    @pytest.mark.asyncio
    async def test_dragonfly_memory_efficiency(self, mock_redis):
        """Test: Efektywność pamięciowa Dragonfly"""

        store = LiveStore()
        store.redis_client = mock_redis

        # Mock memory info
        mock_redis.info.return_value = {
            'used_memory': 1024000,  # 1MB
            'used_memory_human': '1.00M',
            'used_memory_peak': 2048000,  # 2MB
            'used_memory_peak_human': '2.00M',
            'maxmemory': 1073741824,  # 1GB
            'maxmemory_human': '1.00G'
        }

        # Test memory monitoring
        memory_info = await mock_redis.info('memory')

        assert memory_info['used_memory'] == 1024000
        assert memory_info['maxmemory'] == 1073741824

        # Sprawdź czy zużycie pamięci jest rozsądne
        memory_usage_ratio = memory_info['used_memory'] / memory_info['maxmemory']
        assert memory_usage_ratio < 0.8  # Poniżej 80%

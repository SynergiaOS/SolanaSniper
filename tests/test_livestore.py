# SolanaSniper 3.0 - LiveStore Tests
# OPERACJA "STRESS TEST" - Unit Tests dla LiveStore

import pytest
import asyncio
import json
import redis
from unittest.mock import Mock, AsyncMock, patch
from fastapi.testclient import TestClient
import sys
import os

# Dodaj ścieżkę do livestore
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from livestore_server import app, LiveStore

class TestLiveStore:
    """
    Kompletny test suite dla LiveStore
    
    Testuje wszystkie funkcjonalności:
    - API endpoints
    - WebSocket connections
    - Redis integration
    - Message publishing/subscribing
    - Error handling
    - Performance
    """
    
    @pytest.fixture
    def client(self):
        """Fixture tworzący test client"""
        return TestClient(app)
    
    @pytest.fixture
    def mock_redis(self):
        """Fixture z mock Redis"""
        mock_redis = Mock()
        mock_redis.ping.return_value = True
        mock_redis.publish.return_value = 1
        mock_redis.lrange.return_value = []
        mock_redis.lpush.return_value = 1
        mock_redis.ltrim.return_value = True
        return mock_redis
    
    @pytest.fixture
    def sample_message(self):
        """Fixture z przykładową wiadomością"""
        return {
            'type': 'test_message',
            'content': 'Test content',
            'timestamp': '2025-06-09T10:00:00Z'
        }
    
    # === TESTY API ENDPOINTS ===
    
    def test_root_endpoint(self, client):
        """Test: Czy endpoint root działa"""
        response = client.get("/")
        
        assert response.status_code == 200
        data = response.json()
        assert data['service'] == 'SolanaSniper 3.0 - LiveStore'
        assert 'version' in data
        assert 'channels' in data
    
    def test_channels_endpoint(self, client):
        """Test: Czy endpoint channels zwraca listę kanałów"""
        response = client.get("/channels")
        
        assert response.status_code == 200
        data = response.json()
        assert 'channels' in data
        assert len(data['channels']) > 0
        assert 'raw_data' in data['channels']
        assert 'opportunities' in data['channels']
        assert 'analysis_reports' in data['channels']
        assert 'security_reports' in data['channels']
    
    def test_health_endpoint(self, client):
        """Test: Czy endpoint health działa"""
        response = client.get("/health")
        
        assert response.status_code == 200
        data = response.json()
        assert data['status'] == 'healthy'
        assert 'redis_connected' in data
        assert 'uptime' in data
    
    @patch('livestore_server.redis_client')
    def test_publish_endpoint_success(self, mock_redis_client, client, sample_message):
        """Test: Czy publikacja wiadomości działa"""
        mock_redis_client.ping.return_value = True
        mock_redis_client.publish.return_value = 1
        mock_redis_client.lpush.return_value = 1
        mock_redis_client.ltrim.return_value = True
        
        response = client.post(
            "/publish/raw_data?source=test",
            json=sample_message
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data['status'] == 'published'
        assert data['channel'] == 'raw_data'
        assert 'message_id' in data
    
    def test_publish_endpoint_invalid_channel(self, client, sample_message):
        """Test: Czy publikacja do nieprawidłowego kanału zwraca błąd"""
        response = client.post(
            "/publish/invalid_channel?source=test",
            json=sample_message
        )
        
        assert response.status_code == 404
        data = response.json()
        assert 'not found' in data['detail'].lower()
    
    def test_publish_endpoint_missing_source(self, client, sample_message):
        """Test: Czy publikacja bez source zwraca błąd"""
        response = client.post(
            "/publish/raw_data",
            json=sample_message
        )
        
        assert response.status_code == 422  # Validation error
    
    @patch('livestore_server.redis_client')
    def test_history_endpoint_success(self, mock_redis_client, client):
        """Test: Czy endpoint history zwraca historię"""
        # Mock Redis response
        mock_messages = [
            json.dumps({
                'channel': 'raw_data',
                'data': {'test': 'message1'},
                'timestamp': '2025-06-09T10:00:00Z',
                'message_id': 'msg_1'
            }),
            json.dumps({
                'channel': 'raw_data', 
                'data': {'test': 'message2'},
                'timestamp': '2025-06-09T10:01:00Z',
                'message_id': 'msg_2'
            })
        ]
        mock_redis_client.lrange.return_value = mock_messages
        
        response = client.get("/history/raw_data")
        
        assert response.status_code == 200
        data = response.json()
        assert data['channel'] == 'raw_data'
        assert len(data['messages']) == 2
        assert data['count'] == 2
    
    def test_history_endpoint_invalid_channel(self, client):
        """Test: Czy history dla nieprawidłowego kanału zwraca błąd"""
        response = client.get("/history/invalid_channel")
        
        assert response.status_code == 404
    
    @patch('livestore_server.redis_client')
    def test_history_endpoint_with_limit(self, mock_redis_client, client):
        """Test: Czy limit w history działa"""
        mock_redis_client.lrange.return_value = []
        
        response = client.get("/history/raw_data?limit=5")
        
        assert response.status_code == 200
        # Sprawdź czy Redis został wywołany z prawidłowym limitem
        mock_redis_client.lrange.assert_called_with('history:raw_data', 0, 4)  # limit-1
    
    # === TESTY WEBSOCKET ===
    
    def test_websocket_invalid_channel(self, client):
        """Test: Czy WebSocket do nieprawidłowego kanału zwraca błąd"""
        with pytest.raises(Exception):
            with client.websocket_connect("/ws/invalid_channel"):
                pass
    
    # === TESTY REDIS INTEGRATION ===
    
    @patch('livestore_server.redis_client')
    def test_redis_connection_success(self, mock_redis_client):
        """Test: Czy połączenie z Redis działa"""
        mock_redis_client.ping.return_value = True
        
        livestore = LiveStore()
        result = livestore.test_redis_connection()
        
        assert result == True
        mock_redis_client.ping.assert_called_once()
    
    @patch('livestore_server.redis_client')
    def test_redis_connection_failure(self, mock_redis_client):
        """Test: Czy błąd połączenia z Redis jest obsługiwany"""
        mock_redis_client.ping.side_effect = redis.ConnectionError("Connection failed")
        
        livestore = LiveStore()
        result = livestore.test_redis_connection()
        
        assert result == False
    
    @patch('livestore_server.redis_client')
    def test_publish_message_success(self, mock_redis_client, sample_message):
        """Test: Czy publikacja wiadomości do Redis działa"""
        mock_redis_client.publish.return_value = 1
        mock_redis_client.lpush.return_value = 1
        mock_redis_client.ltrim.return_value = True
        
        livestore = LiveStore()
        result = livestore.publish_message('raw_data', sample_message, 'test_source')
        
        assert result is not None
        assert 'message_id' in result
        mock_redis_client.publish.assert_called_once()
        mock_redis_client.lpush.assert_called_once()
    
    @patch('livestore_server.redis_client')
    def test_publish_message_redis_error(self, mock_redis_client, sample_message):
        """Test: Czy błędy Redis przy publikacji są obsługiwane"""
        mock_redis_client.publish.side_effect = redis.RedisError("Redis error")
        
        livestore = LiveStore()
        
        with pytest.raises(Exception):
            livestore.publish_message('raw_data', sample_message, 'test_source')
    
    @patch('livestore_server.redis_client')
    def test_get_message_history_success(self, mock_redis_client):
        """Test: Czy pobieranie historii z Redis działa"""
        mock_messages = [
            json.dumps({'test': 'message1'}),
            json.dumps({'test': 'message2'})
        ]
        mock_redis_client.lrange.return_value = mock_messages
        
        livestore = LiveStore()
        result = livestore.get_message_history('raw_data', limit=10)
        
        assert len(result) == 2
        mock_redis_client.lrange.assert_called_with('history:raw_data', 0, 9)
    
    @patch('livestore_server.redis_client')
    def test_get_message_history_invalid_json(self, mock_redis_client):
        """Test: Czy nieprawidłowy JSON w historii jest obsługiwany"""
        mock_redis_client.lrange.return_value = ['invalid json', '{"valid": "json"}']
        
        livestore = LiveStore()
        result = livestore.get_message_history('raw_data', limit=10)
        
        # Powinien zwrócić tylko prawidłowe wiadomości
        assert len(result) == 1
        assert result[0]['valid'] == 'json'
    
    # === TESTY ERROR HANDLING ===
    
    @patch('livestore_server.redis_client')
    def test_publish_endpoint_redis_error(self, mock_redis_client, client, sample_message):
        """Test: Czy błędy Redis w endpoint są obsługiwane"""
        mock_redis_client.publish.side_effect = redis.RedisError("Redis error")
        
        response = client.post(
            "/publish/raw_data?source=test",
            json=sample_message
        )
        
        assert response.status_code == 500
    
    @patch('livestore_server.redis_client')
    def test_history_endpoint_redis_error(self, mock_redis_client, client):
        """Test: Czy błędy Redis w history są obsługiwane"""
        mock_redis_client.lrange.side_effect = redis.RedisError("Redis error")
        
        response = client.get("/history/raw_data")
        
        assert response.status_code == 500
    
    def test_publish_endpoint_invalid_json(self, client):
        """Test: Czy nieprawidłowy JSON jest obsługiwany"""
        response = client.post(
            "/publish/raw_data?source=test",
            data="invalid json",
            headers={"Content-Type": "application/json"}
        )
        
        assert response.status_code == 422  # Validation error
    
    # === TESTY PERFORMANCE ===
    
    @patch('livestore_server.redis_client')
    def test_publish_multiple_messages_performance(self, mock_redis_client, client):
        """Test: Czy system radzi sobie z wieloma wiadomościami"""
        mock_redis_client.ping.return_value = True
        mock_redis_client.publish.return_value = 1
        mock_redis_client.lpush.return_value = 1
        mock_redis_client.ltrim.return_value = True
        
        # Publikuj 100 wiadomości
        for i in range(100):
            message = {'test': f'message_{i}', 'index': i}
            response = client.post(
                f"/publish/raw_data?source=test_{i}",
                json=message
            )
            assert response.status_code == 200
        
        # Sprawdź czy wszystkie zostały opublikowane
        assert mock_redis_client.publish.call_count == 100
    
    @patch('livestore_server.redis_client')
    def test_history_large_dataset(self, mock_redis_client, client):
        """Test: Czy system radzi sobie z dużą historią"""
        # Mock dużej ilości wiadomości
        large_history = [
            json.dumps({'test': f'message_{i}'}) 
            for i in range(1000)
        ]
        mock_redis_client.lrange.return_value = large_history
        
        response = client.get("/history/raw_data?limit=1000")
        
        assert response.status_code == 200
        data = response.json()
        assert len(data['messages']) == 1000
    
    # === TESTY EDGE CASES ===
    
    def test_publish_empty_message(self, client):
        """Test: Czy pusta wiadomość jest obsługiwana"""
        response = client.post(
            "/publish/raw_data?source=test",
            json={}
        )
        
        assert response.status_code == 200  # Pusta wiadomość powinna być akceptowana
    
    def test_publish_very_large_message(self, client):
        """Test: Czy bardzo duża wiadomość jest obsługiwana"""
        large_message = {
            'data': 'x' * 10000,  # 10KB danych
            'type': 'large_test'
        }
        
        with patch('livestore_server.redis_client') as mock_redis:
            mock_redis.ping.return_value = True
            mock_redis.publish.return_value = 1
            mock_redis.lpush.return_value = 1
            mock_redis.ltrim.return_value = True
            
            response = client.post(
                "/publish/raw_data?source=test",
                json=large_message
            )
            
            assert response.status_code == 200
    
    def test_history_zero_limit(self, client):
        """Test: Czy limit 0 jest obsługiwany"""
        with patch('livestore_server.redis_client') as mock_redis:
            mock_redis.lrange.return_value = []
            
            response = client.get("/history/raw_data?limit=0")
            
            assert response.status_code == 200
            data = response.json()
            assert len(data['messages']) == 0
    
    def test_history_negative_limit(self, client):
        """Test: Czy ujemny limit jest obsługiwany"""
        response = client.get("/history/raw_data?limit=-1")
        
        # Powinien zwrócić błąd walidacji lub domyślny limit
        assert response.status_code in [200, 422]

# === TESTY INTEGRACYJNE ===

@patch('livestore_server.redis_client')
def test_full_message_flow(mock_redis_client, client):
    """Test integracyjny: Pełny przepływ wiadomości"""
    # Mock Redis
    mock_redis_client.ping.return_value = True
    mock_redis_client.publish.return_value = 1
    mock_redis_client.lpush.return_value = 1
    mock_redis_client.ltrim.return_value = True
    
    # Publikuj wiadomość
    message = {
        'type': 'test_flow',
        'content': 'Integration test message',
        'timestamp': '2025-06-09T10:00:00Z'
    }
    
    publish_response = client.post(
        "/publish/raw_data?source=integration_test",
        json=message
    )
    
    assert publish_response.status_code == 200
    
    # Mock historię z opublikowaną wiadomością
    published_message = {
        'channel': 'raw_data',
        'data': message,
        'timestamp': '2025-06-09T10:00:00Z',
        'source': 'integration_test',
        'message_id': 'msg_test'
    }
    mock_redis_client.lrange.return_value = [json.dumps(published_message)]
    
    # Pobierz historię
    history_response = client.get("/history/raw_data?limit=1")
    
    assert history_response.status_code == 200
    history_data = history_response.json()
    assert len(history_data['messages']) == 1
    assert history_data['messages'][0]['data']['type'] == 'test_flow'

if __name__ == "__main__":
    pytest.main([__file__, "-v"])

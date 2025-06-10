# SolanaSniper 3.0 - Test Configuration
# OPERACJA "STRESS TEST" - Konfiguracja testów

import pytest
import asyncio
import os
import sys
from unittest.mock import Mock, AsyncMock

# Dodaj ścieżki do wszystkich modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'analyst'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'risk'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

# Konfiguracja pytest dla testów asynchronicznych
@pytest.fixture(scope="session")
def event_loop():
    """Tworzy event loop dla całej sesji testowej"""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

# Globalne fixtures
@pytest.fixture
def mock_aiohttp_session():
    """Mock dla aiohttp.ClientSession"""
    session = AsyncMock()
    response = AsyncMock()
    response.status = 200
    response.json.return_value = {'status': 'ok'}
    response.text.return_value = 'OK'
    session.get.return_value.__aenter__.return_value = response
    session.post.return_value.__aenter__.return_value = response
    return session

@pytest.fixture
def mock_websocket():
    """Mock dla WebSocket connection"""
    websocket = AsyncMock()
    websocket.send.return_value = None
    websocket.recv.return_value = '{"test": "message"}'
    return websocket

# Test markers
pytest_plugins = []

def pytest_configure(config):
    """Konfiguracja pytest"""
    config.addinivalue_line(
        "markers", "unit: mark test as unit test"
    )
    config.addinivalue_line(
        "markers", "integration: mark test as integration test"
    )
    config.addinivalue_line(
        "markers", "performance: mark test as performance test"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow running"
    )
    config.addinivalue_line(
        "markers", "chaos: mark test as chaos engineering test"
    )
    config.addinivalue_line(
        "markers", "security: mark test as security/penetration test"
    )
    config.addinivalue_line(
        "markers", "market: mark test as market scenario test"
    )
    config.addinivalue_line(
        "markers", "coverage: mark test as coverage enhancement test"
    )

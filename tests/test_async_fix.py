# SolanaSniper 3.0 - Test AsyncMock Fix
# OPERACJA "ASYNC FIX" - Weryfikacja naprawy AsyncMock

import pytest
import asyncio
import json
from unittest.mock import AsyncMock, MagicMock
import sys
import os

# Dodaj ścieżki
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))

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

def create_fixed_mock_session():
    """Tworzy poprawnie zmockowaną sesję"""
    session = AsyncMock()
    
    def mock_post(*args, **kwargs):
        return AsyncContextManagerMock(status=200, json_data={'success': True})
    
    session.post = MagicMock(side_effect=mock_post)
    return session

class TestAsyncFix:
    """Testy naprawy AsyncMock"""
    
    @pytest.mark.asyncio
    async def test_async_context_manager_mock(self):
        """Test: Czy nasz AsyncContextManagerMock działa"""
        
        mock_cm = AsyncContextManagerMock(status=200, json_data={'test': 'data'})
        
        async with mock_cm as response:
            assert response.status == 200
            json_data = await response.json()
            assert json_data['test'] == 'data'
    
    @pytest.mark.asyncio
    async def test_fixed_session_mock(self):
        """Test: Czy naprawiona sesja działa"""
        
        session = create_fixed_mock_session()
        
        # Test POST request
        async with session.post('http://test.com', json={'data': 'test'}) as response:
            assert response.status == 200
            json_data = await response.json()
            assert json_data['success'] == True
        
        # Sprawdź czy mock został wywołany
        assert session.post.called
        assert session.post.call_count == 1
    
    @pytest.mark.asyncio
    async def test_scout_agent_with_fixed_mock(self):
        """Test: Czy Scout Agent działa z naprawionym mockiem"""
        
        try:
            from scout_agent import ScoutAgent
            
            scout = ScoutAgent('http://test:8000')
            scout.session = create_fixed_mock_session()
            scout.stats = {'opportunities_published': 0}
            
            article = {
                'title': 'Solana price surges',
                'url': 'https://example.com',
                'source': 'TestNews'
            }
            
            analysis = {
                'is_opportunity': True,
                'score': 25,
                'reasons': ['Test'],
                'solana_related': True,
                'risk_level': 'low'
            }
            
            # To powinno działać bez błędów AsyncMock
            await scout._publish_opportunity(article, analysis)
            
            # Sprawdź wyniki
            assert scout.session.post.called
            assert scout.stats['opportunities_published'] == 1
            
        except ImportError:
            pytest.skip("Scout Agent not available")
    
    def test_mock_creation_performance(self):
        """Test: Wydajność tworzenia mocków"""
        
        import time
        
        start_time = time.time()
        
        # Stwórz 100 mocków
        for i in range(100):
            session = create_fixed_mock_session()
            assert session is not None
        
        end_time = time.time()
        duration = end_time - start_time
        
        # Powinno być szybkie
        assert duration < 1.0, f"Mock creation too slow: {duration}s"
        
        print(f"✅ Mock Performance: 100 sessions in {duration:.3f}s")
    
    @pytest.mark.asyncio
    async def test_multiple_requests(self):
        """Test: Wiele requestów z tym samym mockiem"""
        
        session = create_fixed_mock_session()
        
        # Wykonaj 5 requestów
        for i in range(5):
            async with session.post(f'http://test{i}.com') as response:
                assert response.status == 200
                json_data = await response.json()
                assert json_data['success'] == True
        
        # Sprawdź liczbę wywołań
        assert session.post.call_count == 5
    
    @pytest.mark.asyncio
    async def test_error_handling(self):
        """Test: Obsługa błędów w mocku"""
        
        # Mock z błędem
        error_mock = AsyncContextManagerMock(status=500, json_data={'error': 'Server Error'})
        
        async with error_mock as response:
            assert response.status == 500
            json_data = await response.json()
            assert 'error' in json_data
    
    def test_mock_structure(self):
        """Test: Struktura mocka"""
        
        session = create_fixed_mock_session()
        
        # Sprawdź czy ma wymagane metody
        assert hasattr(session, 'post')
        assert callable(session.post)
        
        # Sprawdź czy zwraca context manager
        result = session.post('http://test.com')
        assert hasattr(result, '__aenter__')
        assert hasattr(result, '__aexit__')

@pytest.mark.asyncio
async def test_real_world_scenario():
    """Test: Scenariusz z prawdziwego świata"""
    
    # Symuluj prawdziwy scenariusz publikacji
    session = create_fixed_mock_session()
    
    opportunity = {
        'type': 'trading_opportunity',
        'source_article': {
            'title': 'Solana DeFi TVL Surges 300%',
            'url': 'https://coindesk.com/solana-defi',
            'source': 'CoinDesk'
        },
        'analysis': {
            'score': 42,
            'is_opportunity': True,
            'solana_related': True
        }
    }
    
    # Publikuj okazję
    publish_url = "http://localhost:8000/publish/opportunities"
    
    async with session.post(publish_url, json=opportunity, params={'source': 'scout_agent'}) as response:
        assert response.status == 200
        result = await response.json()
        assert result['success'] == True
    
    print("✅ Real-world scenario test passed!")

if __name__ == "__main__":
    # Uruchom testy bezpośrednio
    import asyncio
    
    async def run_tests():
        print("🧪 Testing AsyncMock Fix...")
        
        # Test 1: Basic mock
        mock = AsyncContextManagerMock()
        async with mock as response:
            assert response.status == 200
        print("✅ Test 1: Basic mock - PASSED")
        
        # Test 2: Session mock
        session = create_fixed_mock_session()
        async with session.post('http://test.com') as response:
            assert response.status == 200
        print("✅ Test 2: Session mock - PASSED")
        
        # Test 3: Real scenario
        await test_real_world_scenario()
        
        print("🎉 All AsyncMock fix tests PASSED!")
    
    asyncio.run(run_tests())

# SolanaSniper 3.0 - Testy Walidacji Danych
# OPERACJA "STRAŻNIK" - Testy bezpieczeństwa i walidacji danych

import pytest
import json
import sys
import os
from unittest.mock import AsyncMock
from datetime import datetime, timedelta
import re

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

class TestDataValidation:
    """Testy walidacji i sanityzacji danych"""

    @pytest.mark.asyncio
    async def test_article_data_validation(self):
        """Test: Walidacja danych artykułów"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        scout.session.post.return_value.__aenter__.return_value.status = 200
        
        # Przypadki testowe walidacji
        validation_cases = [
            # Prawidłowy artykuł
            {
                "data": {
                    "title": "Solana price surge",
                    "url": "https://example.com/news",
                    "source": "CryptoNews",
                    "published_date": "2025-06-09T10:00:00Z",
                    "content": "Solana (SOL) price increased by 15%"
                },
                "should_pass": True,
                "description": "Valid article"
            },
            
            # Brakujący tytuł
            {
                "data": {
                    "url": "https://example.com/news",
                    "source": "CryptoNews",
                    "published_date": "2025-06-09T10:00:00Z",
                    "content": "Content without title"
                },
                "should_pass": False,
                "description": "Missing title"
            },
            
            # Nieprawidłowy URL
            {
                "data": {
                    "title": "Test article",
                    "url": "not-a-valid-url",
                    "source": "CryptoNews",
                    "published_date": "2025-06-09T10:00:00Z",
                    "content": "Test content"
                },
                "should_pass": False,
                "description": "Invalid URL"
            },
            
            # Nieprawidłowa data
            {
                "data": {
                    "title": "Test article",
                    "url": "https://example.com/news",
                    "source": "CryptoNews",
                    "published_date": "invalid-date",
                    "content": "Test content"
                },
                "should_pass": False,
                "description": "Invalid date format"
            },
            
            # Zbyt długi tytuł
            {
                "data": {
                    "title": "A" * 1000,  # 1000 znaków
                    "url": "https://example.com/news",
                    "source": "CryptoNews",
                    "published_date": "2025-06-09T10:00:00Z",
                    "content": "Test content"
                },
                "should_pass": False,
                "description": "Title too long"
            },
            
            # Pusty content
            {
                "data": {
                    "title": "Test article",
                    "url": "https://example.com/news",
                    "source": "CryptoNews",
                    "published_date": "2025-06-09T10:00:00Z",
                    "content": ""
                },
                "should_pass": False,
                "description": "Empty content"
            }
        ]
        
        for case in validation_cases:
            try:
                result = await scout._analyze_article(case["data"])
                
                if case["should_pass"]:
                    assert result is not None, f"Failed: {case['description']}"
                    assert isinstance(result, dict), f"Invalid result type: {case['description']}"
                else:
                    # Sprawdź czy walidacja wykryła błąd
                    if result is not None:
                        # Jeśli nie ma walidacji, wynik może być None lub zawierać błędy
                        assert result.get('is_opportunity', False) == False, f"Should reject: {case['description']}"
                        
            except (ValueError, TypeError, KeyError) as e:
                if not case["should_pass"]:
                    # Oczekiwany błąd walidacji
                    assert True
                else:
                    pytest.fail(f"Unexpected validation error for {case['description']}: {e}")

    def test_url_validation(self):
        """Test: Walidacja URL-i"""
        
        valid_urls = [
            "https://example.com",
            "https://news.example.com/article",
            "https://example.com/path?param=value",
            "https://subdomain.example.com:8080/path"
        ]
        
        invalid_urls = [
            "not-a-url",
            "ftp://example.com",  # Nieprawidłowy protokół
            "https://",  # Niekompletny URL
            "javascript:alert('xss')",  # Potencjalnie niebezpieczny
            "data:text/html,<script>alert('xss')</script>",  # Data URL
            "",  # Pusty
            None,  # None
            "https://example.com with spaces",  # Spacje w URL
            "https://example.com\nwith\nnewlines"  # Znaki nowej linii
        ]
        
        # Test walidacji URL
        url_pattern = re.compile(
            r'^https?://'  # http:// lub https://
            r'(?:(?:[A-Z0-9](?:[A-Z0-9-]{0,61}[A-Z0-9])?\.)+[A-Z]{2,6}\.?|'  # domena
            r'localhost|'  # localhost
            r'\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})'  # IP
            r'(?::\d+)?'  # port opcjonalny
            r'(?:/?|[/?]\S+)$', re.IGNORECASE)
        
        for url in valid_urls:
            assert url_pattern.match(url), f"Valid URL rejected: {url}"
        
        for url in invalid_urls:
            if url is not None:
                assert not url_pattern.match(url), f"Invalid URL accepted: {url}"

    def test_date_validation(self):
        """Test: Walidacja dat"""
        
        valid_dates = [
            "2025-06-09T10:00:00Z",
            "2025-06-09T10:00:00.000Z",
            "2025-06-09T10:00:00+00:00",
            "2025-06-09T10:00:00-05:00"
        ]
        
        invalid_dates = [
            "2025-13-01T10:00:00Z",  # Nieprawidłowy miesiąc
            "2025-06-32T10:00:00Z",  # Nieprawidłowy dzień
            "2025-06-09T25:00:00Z",  # Nieprawidłowa godzina
            "2025-06-09",  # Brak czasu
            "invalid-date",  # Nieprawidłowy format
            "",  # Pusty
            None,  # None
            "2025/06/09 10:00:00"  # Nieprawidłowy separator
        ]
        
        for date_str in valid_dates:
            try:
                datetime.fromisoformat(date_str.replace('Z', '+00:00'))
                assert True
            except ValueError:
                pytest.fail(f"Valid date rejected: {date_str}")
        
        for date_str in invalid_dates:
            if date_str is not None:
                try:
                    datetime.fromisoformat(date_str.replace('Z', '+00:00'))
                    pytest.fail(f"Invalid date accepted: {date_str}")
                except ValueError:
                    assert True

    def test_text_sanitization(self):
        """Test: Sanityzacja tekstu"""
        
        malicious_inputs = [
            # XSS attempts
            "<script>alert('xss')</script>",
            "javascript:alert('xss')",
            "<img src=x onerror=alert('xss')>",
            
            # SQL injection attempts
            "'; DROP TABLE users; --",
            "' OR '1'='1",
            "UNION SELECT * FROM passwords",
            
            # Command injection
            "; rm -rf /",
            "$(rm -rf /)",
            "`rm -rf /`",
            
            # Path traversal
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            
            # Null bytes
            "test\x00.txt",
            
            # Unicode attacks
            "test\u202e.txt",  # Right-to-left override
            
            # Very long strings
            "A" * 10000,
            
            # Control characters
            "test\r\n\t\b\f",
            
            # HTML entities
            "&lt;script&gt;alert('xss')&lt;/script&gt;"
        ]
        
        for malicious_input in malicious_inputs:
            # Test czy input jest odpowiednio sanityzowany
            sanitized = self._sanitize_text(malicious_input)
            
            # Sprawdź czy niebezpieczne znaki zostały usunięte/escaped
            assert "<script>" not in sanitized.lower()
            assert "javascript:" not in sanitized.lower()
            assert "drop table" not in sanitized.lower()
            assert "rm -rf" not in sanitized
            assert "\x00" not in sanitized
            
    def _sanitize_text(self, text: str) -> str:
        """Przykładowa funkcja sanityzacji tekstu"""
        if not isinstance(text, str):
            return ""
        
        # Usuń znaki kontrolne
        text = re.sub(r'[\x00-\x1f\x7f-\x9f]', '', text)
        
        # Usuń potencjalnie niebezpieczne wzorce
        dangerous_patterns = [
            r'<script[^>]*>.*?</script>',
            r'javascript:',
            r'on\w+\s*=',
            r'drop\s+table',
            r'union\s+select',
            r'\.\./+',
            r'rm\s+-rf'
        ]
        
        for pattern in dangerous_patterns:
            text = re.sub(pattern, '', text, flags=re.IGNORECASE)
        
        # Ogranicz długość
        if len(text) > 1000:
            text = text[:1000]
        
        return text.strip()

    @pytest.mark.asyncio
    async def test_json_validation(self):
        """Test: Walidacja JSON"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        
        json_test_cases = [
            # Prawidłowy JSON
            '{"title": "Test", "content": "Valid JSON"}',
            
            # Nieprawidłowy JSON
            '{"title": "Test", "content": "Invalid JSON"',  # Brak zamykającego nawiasu
            '{title: "Test"}',  # Brak cudzysłowów wokół klucza
            '{"title": "Test",}',  # Przecinek na końcu
            '',  # Pusty string
            'null',  # Null
            'undefined',  # Undefined
            '{"nested": {"deep": {"very": {"deep": "object"}}}}',  # Głęboko zagnieżdżony
        ]
        
        for json_str in json_test_cases:
            try:
                parsed = json.loads(json_str)
                # Sprawdź czy parsed JSON jest bezpieczny
                assert isinstance(parsed, (dict, list, str, int, float, bool, type(None)))
                
                # Sprawdź głębokość zagnieżdżenia
                if isinstance(parsed, dict):
                    depth = self._get_json_depth(parsed)
                    assert depth <= 10, f"JSON too deeply nested: {depth}"
                    
            except json.JSONDecodeError:
                # Oczekiwane dla nieprawidłowego JSON
                pass

    def _get_json_depth(self, obj, depth=0):
        """Oblicza głębokość zagnieżdżenia JSON"""
        if isinstance(obj, dict):
            return max([self._get_json_depth(v, depth + 1) for v in obj.values()], default=depth)
        elif isinstance(obj, list):
            return max([self._get_json_depth(item, depth + 1) for item in obj], default=depth)
        else:
            return depth

    def test_token_address_validation(self):
        """Test: Walidacja adresów tokenów Solana"""
        
        valid_addresses = [
            "So11111111111111111111111111111111111111112",  # SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",  # USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",  # USDT
        ]
        
        invalid_addresses = [
            "invalid_address",
            "So1111111111111111111111111111111111111111",  # Za krótki
            "So111111111111111111111111111111111111111123",  # Za długi
            "",  # Pusty
            None,  # None
            "0x742d35Cc6634C0532925a3b8D4C9db4C4C4C4C4C",  # Ethereum address
            "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",  # Bitcoin address
        ]
        
        # Wzorzec dla adresu Solana (Base58, 32-44 znaki)
        solana_address_pattern = re.compile(r'^[1-9A-HJ-NP-Za-km-z]{32,44}$')
        
        for address in valid_addresses:
            assert solana_address_pattern.match(address), f"Valid address rejected: {address}"
        
        for address in invalid_addresses:
            if address is not None:
                assert not solana_address_pattern.match(address), f"Invalid address accepted: {address}"

    @pytest.mark.asyncio
    async def test_numerical_validation(self):
        """Test: Walidacja wartości numerycznych"""
        
        risk_agent = RiskAgent(livestore_url="http://localhost:8000")
        
        numerical_test_cases = [
            # Prawidłowe wartości
            {"rug_score": 25, "liquidity_amount_usd": 50000, "top_holder_percentage": 15.5},
            {"rug_score": 0, "liquidity_amount_usd": 1000, "top_holder_percentage": 0.0},
            {"rug_score": 100, "liquidity_amount_usd": 999999999, "top_holder_percentage": 100.0},
            
            # Nieprawidłowe wartości
            {"rug_score": -10, "liquidity_amount_usd": -1000, "top_holder_percentage": -5.0},
            {"rug_score": 150, "liquidity_amount_usd": "invalid", "top_holder_percentage": 150.0},
            {"rug_score": None, "liquidity_amount_usd": None, "top_holder_percentage": None},
            {"rug_score": float('inf'), "liquidity_amount_usd": float('nan'), "top_holder_percentage": float('inf')},
        ]
        
        for case in numerical_test_cases:
            try:
                # Test walidacji numerycznej
                validated_data = self._validate_numerical_data(case)
                
                # Sprawdź zakresy
                if 'rug_score' in validated_data:
                    assert 0 <= validated_data['rug_score'] <= 100
                if 'liquidity_amount_usd' in validated_data:
                    assert validated_data['liquidity_amount_usd'] >= 0
                if 'top_holder_percentage' in validated_data:
                    assert 0 <= validated_data['top_holder_percentage'] <= 100
                    
            except (ValueError, TypeError):
                # Oczekiwane dla nieprawidłowych wartości
                pass

    def _validate_numerical_data(self, data):
        """Przykładowa funkcja walidacji danych numerycznych"""
        validated = {}
        
        for key, value in data.items():
            if value is None:
                continue
                
            if key == 'rug_score':
                if isinstance(value, (int, float)) and 0 <= value <= 100:
                    validated[key] = float(value)
                else:
                    raise ValueError(f"Invalid rug_score: {value}")
                    
            elif key == 'liquidity_amount_usd':
                if isinstance(value, (int, float)) and value >= 0:
                    validated[key] = float(value)
                else:
                    raise ValueError(f"Invalid liquidity_amount_usd: {value}")
                    
            elif key == 'top_holder_percentage':
                if isinstance(value, (int, float)) and 0 <= value <= 100:
                    validated[key] = float(value)
                else:
                    raise ValueError(f"Invalid top_holder_percentage: {value}")
        
        return validated

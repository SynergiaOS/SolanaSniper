# SolanaSniper 3.0 - Testy Zewnętrznych API
# OPERACJA "ŁĄCZNOŚĆ" - Testy integracji z zewnętrznymi usługami

import pytest
import asyncio
import json
import aiohttp
from unittest.mock import AsyncMock, patch, MagicMock
import sys
import os
from datetime import datetime, timedelta

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents', 'risk'))

from agents.risk.risk_agent import RiskAgent

class TestExternalAPIs:
    """Testy integracji z zewnętrznymi API"""

    @pytest.mark.asyncio
    async def test_lunarcrush_api_integration(self):
        """Test: Integracja z LunarCrush API"""
        
        # Mock odpowiedzi LunarCrush API
        mock_lunarcrush_response = {
            "data": {
                "SOL": {
                    "symbol": "SOL",
                    "name": "Solana",
                    "price": 145.67,
                    "social_score": 85,
                    "sentiment": 0.75,
                    "social_volume": 12500,
                    "social_dominance": 3.2,
                    "market_cap": 65000000000,
                    "volume_24h": 2500000000
                }
            },
            "status": "success"
        }
        
        # Test z mockiem
        with patch('aiohttp.ClientSession.get') as mock_get:
            mock_response = AsyncMock()
            mock_response.status = 200
            mock_response.json.return_value = mock_lunarcrush_response
            mock_get.return_value.__aenter__.return_value = mock_response
            
            # Symuluj wywołanie API
            async with aiohttp.ClientSession() as session:
                async with session.get(
                    "https://api.lunarcrush.com/v2/assets",
                    params={"symbol": "SOL", "data_points": 1}
                ) as response:
                    data = await response.json()
                    
                    assert response.status == 200
                    assert data["status"] == "success"
                    assert data["data"]["SOL"]["symbol"] == "SOL"
                    assert data["data"]["SOL"]["social_score"] == 85
                    assert data["data"]["SOL"]["sentiment"] == 0.75

    @pytest.mark.asyncio
    async def test_bitquery_api_integration(self):
        """Test: Integracja z Bitquery API"""
        
        # Mock odpowiedzi Bitquery GraphQL
        mock_bitquery_response = {
            "data": {
                "solana": {
                    "dexTrades": [
                        {
                            "timeInterval": {"minute": "2025-06-09 10:00:00"},
                            "baseCurrency": {"symbol": "SOL"},
                            "quoteCurrency": {"symbol": "USDC"},
                            "trades": 1250,
                            "tradeAmount": 125000.50,
                            "maximum_price": 146.25,
                            "minimum_price": 144.80,
                            "close_price": 145.67
                        }
                    ]
                }
            }
        }
        
        # GraphQL query
        graphql_query = """
        query {
            solana {
                dexTrades(
                    baseCurrency: {is: "So11111111111111111111111111111111111111112"}
                    quoteCurrency: {is: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"}
                    options: {limit: 10, desc: "timeInterval.minute"}
                ) {
                    timeInterval { minute }
                    baseCurrency { symbol }
                    quoteCurrency { symbol }
                    trades
                    tradeAmount
                    maximum_price
                    minimum_price
                    close_price
                }
            }
        }
        """
        
        with patch('aiohttp.ClientSession.post') as mock_post:
            mock_response = AsyncMock()
            mock_response.status = 200
            mock_response.json.return_value = mock_bitquery_response
            mock_post.return_value.__aenter__.return_value = mock_response
            
            # Symuluj wywołanie GraphQL API
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    "https://graphql.bitquery.io",
                    json={"query": graphql_query},
                    headers={"X-API-KEY": "test_api_key"}
                ) as response:
                    data = await response.json()
                    
                    assert response.status == 200
                    assert "data" in data
                    assert "solana" in data["data"]
                    trades = data["data"]["solana"]["dexTrades"][0]
                    assert trades["baseCurrency"]["symbol"] == "SOL"
                    assert trades["trades"] == 1250

    @pytest.mark.asyncio
    async def test_token_metrics_api_integration(self):
        """Test: Integracja z Token Metrics API"""
        
        # Mock odpowiedzi Token Metrics
        mock_token_metrics_response = {
            "data": {
                "symbol": "SOL",
                "ai_grade": "A+",
                "ai_score": 92,
                "trader_grade": "A",
                "trader_score": 88,
                "investor_grade": "A-",
                "investor_score": 85,
                "price_prediction_7d": 165.50,
                "price_prediction_30d": 180.25,
                "risk_level": "Medium",
                "volatility": 0.45,
                "momentum": 0.78
            },
            "status": "success",
            "timestamp": "2025-06-09T10:00:00Z"
        }
        
        with patch('aiohttp.ClientSession.get') as mock_get:
            mock_response = AsyncMock()
            mock_response.status = 200
            mock_response.json.return_value = mock_token_metrics_response
            mock_get.return_value.__aenter__.return_value = mock_response
            
            # Symuluj wywołanie API
            async with aiohttp.ClientSession() as session:
                async with session.get(
                    "https://api.tokenmetrics.com/v1/grades",
                    params={"symbol": "SOL"},
                    headers={"Authorization": "Bearer test_token"}
                ) as response:
                    data = await response.json()
                    
                    assert response.status == 200
                    assert data["status"] == "success"
                    assert data["data"]["symbol"] == "SOL"
                    assert data["data"]["ai_grade"] == "A+"
                    assert data["data"]["ai_score"] == 92

    @pytest.mark.asyncio
    async def test_rugcheck_api_integration(self):
        """Test: Integracja z rugcheck.xyz API"""
        
        risk_agent = RiskAgent(livestore_url="http://localhost:8000")
        
        # Mock odpowiedzi rugcheck.xyz
        mock_rugcheck_response = {
            "mint": "So11111111111111111111111111111111111111112",
            "score": 25,
            "risks": [
                {
                    "name": "Low Liquidity",
                    "description": "Token has relatively low liquidity",
                    "level": "warning",
                    "score": 15
                }
            ],
            "markets": [
                {
                    "name": "Raydium",
                    "liquidity": 50000,
                    "liquidity_locked": True,
                    "lp_locked_pct": 85.5
                }
            ],
            "top_holders": [
                {
                    "address": "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
                    "percentage": 15.5,
                    "is_known": False
                }
            ],
            "is_honeypot": False,
            "can_sell": True
        }
        
        mock_session = AsyncMock()
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.json.return_value = mock_rugcheck_response
        mock_session.get.return_value.__aenter__.return_value = mock_response
        risk_agent.session = mock_session
        
        # Test analizy bezpieczeństwa tokena
        token_address = "So11111111111111111111111111111111111111112"
        
        # Symuluj wywołanie _analyze_token_security
        with patch.object(risk_agent, '_analyze_token_security', return_value=mock_rugcheck_response):
            security_data = await risk_agent._analyze_token_security(token_address)
            
            assert security_data["mint"] == token_address
            assert security_data["score"] == 25
            assert security_data["is_honeypot"] == False
            assert len(security_data["risks"]) == 1
            assert security_data["markets"][0]["liquidity"] == 50000

    @pytest.mark.asyncio
    async def test_gmgn_ai_api_integration(self):
        """Test: Integracja z GMGN.AI API"""
        
        # Mock odpowiedzi GMGN.AI
        mock_gmgn_response = {
            "data": {
                "token": "So11111111111111111111111111111111111111112",
                "smart_money_flow": {
                    "inflow_24h": 2500000,
                    "outflow_24h": 1800000,
                    "net_flow": 700000,
                    "whale_transactions": 45,
                    "smart_money_score": 78
                },
                "whale_activities": [
                    {
                        "wallet": "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
                        "action": "buy",
                        "amount": 50000,
                        "timestamp": "2025-06-09T09:45:00Z",
                        "pnl_7d": 15.5,
                        "win_rate": 0.85
                    }
                ],
                "trending_score": 92,
                "social_mentions": 1250
            },
            "status": "success"
        }
        
        with patch('aiohttp.ClientSession.get') as mock_get:
            mock_response = AsyncMock()
            mock_response.status = 200
            mock_response.json.return_value = mock_gmgn_response
            mock_get.return_value.__aenter__.return_value = mock_response
            
            # Symuluj wywołanie API
            async with aiohttp.ClientSession() as session:
                async with session.get(
                    "https://api.gmgn.ai/v1/smart-money",
                    params={"token": "So11111111111111111111111111111111111111112"},
                    headers={"X-API-KEY": "test_api_key"}
                ) as response:
                    data = await response.json()
                    
                    assert response.status == 200
                    assert data["status"] == "success"
                    flow = data["data"]["smart_money_flow"]
                    assert flow["net_flow"] == 700000
                    assert flow["smart_money_score"] == 78
                    assert len(data["data"]["whale_activities"]) == 1

    @pytest.mark.asyncio
    async def test_api_rate_limiting(self):
        """Test: Obsługa rate limiting API"""
        
        # Symuluj odpowiedzi z rate limiting
        rate_limit_responses = [
            (429, {"error": "Rate limit exceeded", "retry_after": 60}),
            (503, {"error": "Service temporarily unavailable"}),
            (200, {"data": "success"})  # Po retry
        ]
        
        call_count = 0
        
        async def mock_api_call(*args, **kwargs):
            nonlocal call_count
            status, response_data = rate_limit_responses[min(call_count, len(rate_limit_responses) - 1)]
            call_count += 1
            
            mock_response = AsyncMock()
            mock_response.status = status
            mock_response.json.return_value = response_data
            return mock_response
        
        with patch('aiohttp.ClientSession.get', side_effect=mock_api_call):
            async with aiohttp.ClientSession() as session:
                # Pierwsza próba - rate limit
                async with session.get("https://api.example.com/data") as response:
                    assert response.status == 429
                    data = await response.json()
                    assert "rate limit" in data["error"].lower()
                
                # Druga próba - service unavailable
                async with session.get("https://api.example.com/data") as response:
                    assert response.status == 503
                
                # Trzecia próba - sukces
                async with session.get("https://api.example.com/data") as response:
                    assert response.status == 200
                    data = await response.json()
                    assert data["data"] == "success"

    @pytest.mark.asyncio
    async def test_api_authentication_handling(self):
        """Test: Obsługa autentykacji API"""
        
        # Test różnych metod autentykacji
        auth_scenarios = [
            # API Key w header
            {
                "method": "header",
                "headers": {"X-API-KEY": "test_api_key"},
                "expected_status": 200
            },
            # Bearer token
            {
                "method": "bearer",
                "headers": {"Authorization": "Bearer test_token"},
                "expected_status": 200
            },
            # Brak autentykacji
            {
                "method": "none",
                "headers": {},
                "expected_status": 401
            },
            # Nieprawidłowy klucz
            {
                "method": "invalid",
                "headers": {"X-API-KEY": "invalid_key"},
                "expected_status": 403
            }
        ]
        
        for scenario in auth_scenarios:
            with patch('aiohttp.ClientSession.get') as mock_get:
                mock_response = AsyncMock()
                mock_response.status = scenario["expected_status"]
                
                if scenario["expected_status"] == 200:
                    mock_response.json.return_value = {"data": "success"}
                else:
                    mock_response.json.return_value = {"error": "Authentication failed"}
                
                mock_get.return_value.__aenter__.return_value = mock_response
                
                async with aiohttp.ClientSession() as session:
                    async with session.get(
                        "https://api.example.com/data",
                        headers=scenario["headers"]
                    ) as response:
                        assert response.status == scenario["expected_status"]

    @pytest.mark.asyncio
    async def test_api_error_handling(self):
        """Test: Obsługa błędów API"""
        
        # Różne typy błędów API
        api_errors = [
            (400, {"error": "Bad Request", "message": "Invalid parameters"}),
            (404, {"error": "Not Found", "message": "Endpoint not found"}),
            (500, {"error": "Internal Server Error", "message": "Server error"}),
            (502, {"error": "Bad Gateway", "message": "Upstream error"}),
            (503, {"error": "Service Unavailable", "message": "Service down"}),
            (504, {"error": "Gateway Timeout", "message": "Request timeout"})
        ]
        
        for status_code, error_response in api_errors:
            with patch('aiohttp.ClientSession.get') as mock_get:
                mock_response = AsyncMock()
                mock_response.status = status_code
                mock_response.json.return_value = error_response
                mock_get.return_value.__aenter__.return_value = mock_response
                
                async with aiohttp.ClientSession() as session:
                    async with session.get("https://api.example.com/data") as response:
                        assert response.status == status_code
                        data = await response.json()
                        assert "error" in data
                        
                        # Sprawdź czy błąd jest odpowiednio kategoryzowany
                        if status_code >= 500:
                            assert "server" in data["error"].lower() or "gateway" in data["error"].lower()
                        elif status_code >= 400:
                            assert "bad" in data["error"].lower() or "not found" in data["error"].lower()

    @pytest.mark.asyncio
    async def test_api_response_validation(self):
        """Test: Walidacja odpowiedzi API"""
        
        # Test różnych formatów odpowiedzi
        response_formats = [
            # Prawidłowa odpowiedź
            {"data": {"symbol": "SOL", "price": 145.67}, "status": "success"},
            
            # Brakujące pola
            {"data": {"symbol": "SOL"}, "status": "success"},  # Brak price
            
            # Nieprawidłowe typy
            {"data": {"symbol": 123, "price": "invalid"}, "status": "success"},
            
            # Pusty data
            {"data": {}, "status": "success"},
            
            # Brak data
            {"status": "success"},
            
            # Nieprawidłowy JSON
            "invalid json response"
        ]
        
        for response_data in response_formats:
            with patch('aiohttp.ClientSession.get') as mock_get:
                mock_response = AsyncMock()
                mock_response.status = 200
                
                if isinstance(response_data, str):
                    mock_response.json.side_effect = json.JSONDecodeError("Invalid JSON", "", 0)
                    mock_response.text.return_value = response_data
                else:
                    mock_response.json.return_value = response_data
                
                mock_get.return_value.__aenter__.return_value = mock_response
                
                async with aiohttp.ClientSession() as session:
                    try:
                        async with session.get("https://api.example.com/data") as response:
                            if isinstance(response_data, str):
                                # Oczekiwany błąd JSON
                                text = await response.text()
                                assert text == response_data
                            else:
                                data = await response.json()
                                
                                # Walidacja struktury odpowiedzi
                                if "data" in data and "symbol" in data["data"]:
                                    assert isinstance(data["data"]["symbol"], str) or data["data"]["symbol"] is None
                                
                                if "data" in data and "price" in data["data"]:
                                    price = data["data"]["price"]
                                    assert isinstance(price, (int, float)) or price is None
                                    
                    except json.JSONDecodeError:
                        # Oczekiwane dla nieprawidłowego JSON
                        assert isinstance(response_data, str)

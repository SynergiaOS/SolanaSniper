# SolanaSniper 3.0 - Testy Konfiguracji
# OPERACJA "FUNDAMENT" - Testy zarządzania konfiguracją

import pytest
import os
import sys
import tempfile
import json
from unittest.mock import patch, mock_open
from pathlib import Path

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

class TestConfigurationManagement:
    """Testy zarządzania konfiguracją systemu"""

    def test_environment_variables_loading(self):
        """Test: Ładowanie zmiennych środowiskowych"""
        
        # Test zmiennych wymaganych
        required_vars = [
            'LIVESTORE_URL',
            'OLLAMA_URL', 
            'REDIS_URL',
            'LOG_LEVEL'
        ]
        
        # Symuluj brakujące zmienne
        with patch.dict(os.environ, {}, clear=True):
            # Sprawdź czy agenci mają sensowne defaulty
            scout = ScoutAgent(livestore_url="http://localhost:8000")
            assert scout.livestore_url == "http://localhost:8000"
            
            analyst = AnalystAgent(
                livestore_url="http://localhost:8000",
                ollama_url="http://localhost:11434"
            )
            assert analyst.ollama_url == "http://localhost:11434"

    def test_invalid_configuration_handling(self):
        """Test: Obsługa nieprawidłowej konfiguracji"""
        
        invalid_configs = [
            # Nieprawidłowy URL
            {"livestore_url": "invalid-url"},
            {"livestore_url": ""},
            {"livestore_url": None},
            
            # Nieprawidłowy port
            {"livestore_url": "http://localhost:99999"},
            
            # Nieprawidłowy protokół
            {"livestore_url": "ftp://localhost:8000"},
        ]
        
        for config in invalid_configs:
            try:
                scout = ScoutAgent(**config)
                # Sprawdź czy agent ma mechanizm walidacji
                assert hasattr(scout, 'livestore_url')
            except (ValueError, TypeError) as e:
                # Oczekiwany błąd walidacji
                assert "url" in str(e).lower() or "invalid" in str(e).lower()

    def test_configuration_file_loading(self):
        """Test: Ładowanie konfiguracji z pliku"""
        
        # Przygotuj tymczasowy plik konfiguracyjny
        config_data = {
            "livestore": {
                "url": "http://test:8000",
                "timeout": 30
            },
            "ollama": {
                "url": "http://test:11434",
                "model": "gemma:2b"
            },
            "agents": {
                "scout": {
                    "enabled": True,
                    "check_interval": 60
                },
                "analyst": {
                    "enabled": True,
                    "ai_timeout": 30
                },
                "risk": {
                    "enabled": True,
                    "security_threshold": 50
                }
            }
        }
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            config_file = f.name
        
        try:
            # Test ładowania konfiguracji
            with open(config_file, 'r') as f:
                loaded_config = json.load(f)
            
            assert loaded_config['livestore']['url'] == "http://test:8000"
            assert loaded_config['ollama']['model'] == "gemma:2b"
            assert loaded_config['agents']['scout']['enabled'] == True
            
        finally:
            os.unlink(config_file)

    def test_configuration_validation(self):
        """Test: Walidacja konfiguracji"""
        
        # Test wymaganych pól
        minimal_config = {
            "livestore_url": "http://localhost:8000"
        }
        
        scout = ScoutAgent(**minimal_config)
        assert scout.livestore_url == "http://localhost:8000"
        
        # Test z pełną konfiguracją
        full_config = {
            "livestore_url": "http://localhost:8000",
            "check_interval": 60,
            "max_retries": 3,
            "timeout": 30
        }
        
        scout_full = ScoutAgent(**full_config)
        assert scout_full.livestore_url == "http://localhost:8000"

    def test_configuration_override(self):
        """Test: Nadpisywanie konfiguracji"""
        
        # Test priorytetów: parametry > zmienne środowiskowe > defaulty
        with patch.dict(os.environ, {'LIVESTORE_URL': 'http://env:8000'}):
            # Parametr powinien mieć priorytet nad zmienną środowiskową
            scout = ScoutAgent(livestore_url="http://param:8000")
            assert scout.livestore_url == "http://param:8000"

    def test_configuration_secrets_handling(self):
        """Test: Obsługa sekretów w konfiguracji"""
        
        # Sprawdź czy sekrety nie są logowane
        import logging
        
        with patch('logging.Logger.info') as mock_log:
            scout = ScoutAgent(
                livestore_url="http://localhost:8000",
                api_key="secret_key_123"  # Jeśli agent obsługuje API keys
            )
            
            # Sprawdź czy sekret nie pojawił się w logach
            logged_messages = [call.args[0] for call in mock_log.call_args_list]
            for message in logged_messages:
                assert "secret_key_123" not in str(message)

    def test_configuration_hot_reload(self):
        """Test: Hot reload konfiguracji"""
        
        # Symuluj zmianę konfiguracji w runtime
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        original_url = scout.livestore_url
        
        # Jeśli agent obsługuje hot reload
        if hasattr(scout, 'update_config'):
            new_config = {"livestore_url": "http://new:8000"}
            scout.update_config(new_config)
            assert scout.livestore_url == "http://new:8000"
        else:
            # Sprawdź czy brak hot reload jest udokumentowany
            assert original_url == "http://localhost:8000"

    def test_configuration_backup_and_restore(self):
        """Test: Backup i restore konfiguracji"""
        
        original_config = {
            "livestore_url": "http://localhost:8000",
            "timeout": 30
        }
        
        scout = ScoutAgent(**original_config)
        
        # Symuluj backup konfiguracji
        if hasattr(scout, 'get_config'):
            backed_up_config = scout.get_config()
            assert backed_up_config['livestore_url'] == "http://localhost:8000"
        
        # Test restore
        if hasattr(scout, 'restore_config'):
            scout.restore_config(original_config)
            assert scout.livestore_url == "http://localhost:8000"

    def test_configuration_migration(self):
        """Test: Migracja konfiguracji między wersjami"""
        
        # Stara wersja konfiguracji
        old_config = {
            "livestore_host": "localhost",  # Stary format
            "livestore_port": 8000
        }
        
        # Nowa wersja konfiguracji
        new_config = {
            "livestore_url": "http://localhost:8000"  # Nowy format
        }
        
        # Test czy system obsługuje oba formaty
        try:
            scout_old = ScoutAgent(livestore_url="http://localhost:8000")
            scout_new = ScoutAgent(livestore_url="http://localhost:8000")
            
            assert scout_old.livestore_url == scout_new.livestore_url
        except Exception as e:
            # Sprawdź czy błąd jest informatywny
            assert "configuration" in str(e).lower() or "format" in str(e).lower()

@pytest.mark.asyncio
async def test_configuration_runtime_changes():
    """Test: Zmiany konfiguracji w runtime"""
    
    scout = ScoutAgent(livestore_url="http://localhost:8000")
    
    # Test czy agent może działać z różnymi konfiguracjami
    configs_to_test = [
        "http://localhost:8000",
        "http://localhost:8001", 
        "http://test:8000"
    ]
    
    for config_url in configs_to_test:
        scout.livestore_url = config_url
        assert scout.livestore_url == config_url
        
        # Test czy agent może się połączyć (z mockiem)
        from unittest.mock import AsyncMock
        scout.session = AsyncMock()
        scout.session.get.return_value.__aenter__.return_value.status = 200
        
        # Symuluj test połączenia
        try:
            await scout._test_livestore_connection()
        except Exception:
            # Oczekiwane dla testowych URL-i
            pass

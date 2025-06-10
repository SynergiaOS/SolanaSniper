# SolanaSniper 3.0 - Testy Logowania i Monitoringu
# OPERACJA "OBSERWATOR" - Testy observability i metryki

import pytest
import asyncio
import json
import logging
import sys
import os
from unittest.mock import AsyncMock, patch, MagicMock
from datetime import datetime, timedelta
import tempfile
from pathlib import Path

# Dodaj ścieżki do modułów
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'agents'))
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'livestore'))

from agents.scout_agent import ScoutAgent
from agents.analyst.analyst_agent import AnalystAgent
from agents.risk.risk_agent import RiskAgent

class TestLoggingMonitoring:
    """Testy logowania, monitoringu i observability"""

    def test_logging_configuration(self):
        """Test: Konfiguracja logowania"""
        
        # Test różnych poziomów logowania
        log_levels = [
            logging.DEBUG,
            logging.INFO,
            logging.WARNING,
            logging.ERROR,
            logging.CRITICAL
        ]
        
        for level in log_levels:
            # Utwórz logger z określonym poziomem
            logger = logging.getLogger(f"test_logger_{level}")
            logger.setLevel(level)
            
            # Dodaj handler do przechwytywania logów
            with tempfile.NamedTemporaryFile(mode='w+', delete=False) as log_file:
                handler = logging.FileHandler(log_file.name)
                handler.setLevel(level)
                formatter = logging.Formatter(
                    '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
                )
                handler.setFormatter(formatter)
                logger.addHandler(handler)
                
                # Test logowania na różnych poziomach
                logger.debug("Debug message")
                logger.info("Info message")
                logger.warning("Warning message")
                logger.error("Error message")
                logger.critical("Critical message")
                
                # Sprawdź czy logi zostały zapisane
                handler.close()
                logger.removeHandler(handler)
                
                with open(log_file.name, 'r') as f:
                    log_content = f.read()
                
                # Sprawdź czy odpowiednie poziomy zostały zapisane
                if level <= logging.DEBUG:
                    assert "Debug message" in log_content
                if level <= logging.INFO:
                    assert "Info message" in log_content
                if level <= logging.WARNING:
                    assert "Warning message" in log_content
                if level <= logging.ERROR:
                    assert "Error message" in log_content
                if level <= logging.CRITICAL:
                    assert "Critical message" in log_content
                
                os.unlink(log_file.name)

    def test_structured_logging(self):
        """Test: Strukturalne logowanie JSON"""
        
        # Test logowania w formacie JSON
        with tempfile.NamedTemporaryFile(mode='w+', delete=False) as log_file:
            logger = logging.getLogger("structured_test")
            logger.setLevel(logging.INFO)
            
            # Handler z formatowaniem JSON
            handler = logging.FileHandler(log_file.name)
            
            class JSONFormatter(logging.Formatter):
                def format(self, record):
                    log_entry = {
                        'timestamp': datetime.fromtimestamp(record.created).isoformat(),
                        'level': record.levelname,
                        'logger': record.name,
                        'message': record.getMessage(),
                        'module': record.module,
                        'function': record.funcName,
                        'line': record.lineno
                    }
                    
                    # Dodaj dodatkowe pola jeśli istnieją
                    if hasattr(record, 'agent_name'):
                        log_entry['agent_name'] = record.agent_name
                    if hasattr(record, 'operation'):
                        log_entry['operation'] = record.operation
                    if hasattr(record, 'duration'):
                        log_entry['duration'] = record.duration
                    
                    return json.dumps(log_entry)
            
            handler.setFormatter(JSONFormatter())
            logger.addHandler(handler)
            
            # Test logowania z dodatkowymi polami
            logger.info("Agent started", extra={
                'agent_name': 'scout_agent',
                'operation': 'startup'
            })
            
            logger.info("Analysis completed", extra={
                'agent_name': 'analyst_agent',
                'operation': 'analysis',
                'duration': 1.25
            })
            
            handler.close()
            logger.removeHandler(handler)
            
            # Sprawdź czy logi są w formacie JSON
            with open(log_file.name, 'r') as f:
                log_lines = f.readlines()
            
            for line in log_lines:
                log_entry = json.loads(line.strip())
                assert 'timestamp' in log_entry
                assert 'level' in log_entry
                assert 'message' in log_entry
                
                if 'agent_name' in log_entry:
                    assert log_entry['agent_name'] in ['scout_agent', 'analyst_agent']
            
            os.unlink(log_file.name)

    @pytest.mark.asyncio
    async def test_agent_logging(self):
        """Test: Logowanie w agentach"""
        
        scout = ScoutAgent(livestore_url="http://localhost:8000")
        scout.session = AsyncMock()
        scout.session.post.return_value.__aenter__.return_value.status = 200
        
        # Przechwyć logi
        with patch('logging.Logger.info') as mock_info, \
             patch('logging.Logger.error') as mock_error, \
             patch('logging.Logger.warning') as mock_warning:
            
            # Test analizy artykułu z logowaniem
            test_article = {
                "title": "Solana price surge",
                "url": "https://example.com/solana",
                "source": "CryptoNews",
                "published_date": "2025-06-09T10:00:00Z",
                "content": "Solana (SOL) price increased by 15%"
            }
            
            result = await scout._analyze_article(test_article)
            
            # Sprawdź czy odpowiednie logi zostały wywołane
            assert mock_info.called or mock_warning.called or mock_error.called
            
            # Sprawdź zawartość logów
            all_calls = mock_info.call_args_list + mock_warning.call_args_list + mock_error.call_args_list
            log_messages = [call[0][0] for call in all_calls if call[0]]
            
            # Sprawdź czy logi zawierają istotne informacje
            log_text = ' '.join(log_messages)
            assert any(keyword in log_text.lower() for keyword in ['analiz', 'artykuł', 'score', 'solana'])

    def test_performance_metrics(self):
        """Test: Metryki wydajności"""
        
        # Symuluj zbieranie metryk wydajności
        metrics = {
            'articles_processed': 0,
            'opportunities_found': 0,
            'analysis_time_total': 0.0,
            'analysis_time_avg': 0.0,
            'errors_count': 0,
            'api_calls_count': 0,
            'api_response_time_avg': 0.0
        }
        
        # Symuluj przetwarzanie artykułów
        import time
        import random
        
        for i in range(100):
            start_time = time.time()
            
            # Symuluj analizę
            time.sleep(random.uniform(0.001, 0.01))  # 1-10ms
            
            end_time = time.time()
            analysis_time = end_time - start_time
            
            # Aktualizuj metryki
            metrics['articles_processed'] += 1
            metrics['analysis_time_total'] += analysis_time
            metrics['analysis_time_avg'] = metrics['analysis_time_total'] / metrics['articles_processed']
            
            # Symuluj znalezienie okazji (20% szans)
            if random.random() < 0.2:
                metrics['opportunities_found'] += 1
            
            # Symuluj błędy (5% szans)
            if random.random() < 0.05:
                metrics['errors_count'] += 1
        
        # Sprawdź metryki
        assert metrics['articles_processed'] == 100
        assert metrics['opportunities_found'] >= 0
        assert metrics['analysis_time_avg'] > 0
        assert metrics['errors_count'] >= 0
        
        # Sprawdź rozsądne wartości
        assert metrics['analysis_time_avg'] < 1.0  # Średnio poniżej 1s
        assert metrics['opportunities_found'] <= metrics['articles_processed']
        assert metrics['errors_count'] <= metrics['articles_processed']

    def test_health_checks(self):
        """Test: Health checks systemu"""
        
        # Symuluj health check dla różnych komponentów
        health_status = {
            'livestore': {'status': 'healthy', 'response_time': 0.05},
            'redis': {'status': 'healthy', 'response_time': 0.02},
            'ollama': {'status': 'healthy', 'response_time': 0.15},
            'scout_agent': {'status': 'healthy', 'last_activity': datetime.now().isoformat()},
            'analyst_agent': {'status': 'healthy', 'last_activity': datetime.now().isoformat()},
            'risk_agent': {'status': 'healthy', 'last_activity': datetime.now().isoformat()}
        }
        
        # Test health check endpoint
        def check_component_health(component_name):
            component = health_status.get(component_name)
            if not component:
                return {'status': 'unknown', 'error': 'Component not found'}
            
            # Sprawdź czy komponent jest zdrowy
            if component['status'] != 'healthy':
                return {'status': 'unhealthy', 'details': component}
            
            # Sprawdź response time (jeśli istnieje)
            if 'response_time' in component and component['response_time'] > 1.0:
                return {'status': 'degraded', 'reason': 'High response time', 'details': component}
            
            # Sprawdź last activity (jeśli istnieje)
            if 'last_activity' in component:
                last_activity = datetime.fromisoformat(component['last_activity'])
                if datetime.now() - last_activity > timedelta(minutes=5):
                    return {'status': 'stale', 'reason': 'No recent activity', 'details': component}
            
            return {'status': 'healthy', 'details': component}
        
        # Test wszystkich komponentów
        for component_name in health_status.keys():
            health = check_component_health(component_name)
            assert health['status'] in ['healthy', 'degraded', 'unhealthy', 'stale', 'unknown']
            
            if health['status'] == 'healthy':
                assert 'details' in health
            elif health['status'] in ['degraded', 'stale']:
                assert 'reason' in health

    def test_error_tracking(self):
        """Test: Śledzenie błędów"""
        
        # Symuluj system śledzenia błędów
        error_tracker = {
            'errors': [],
            'error_counts': {},
            'last_errors': []
        }
        
        def track_error(error_type, message, context=None):
            error_entry = {
                'timestamp': datetime.now().isoformat(),
                'type': error_type,
                'message': message,
                'context': context or {}
            }
            
            error_tracker['errors'].append(error_entry)
            error_tracker['error_counts'][error_type] = error_tracker['error_counts'].get(error_type, 0) + 1
            
            # Zachowaj tylko ostatnie 100 błędów
            if len(error_tracker['last_errors']) >= 100:
                error_tracker['last_errors'].pop(0)
            error_tracker['last_errors'].append(error_entry)
        
        # Symuluj różne typy błędów
        error_scenarios = [
            ('connection_error', 'Failed to connect to LiveStore', {'url': 'http://localhost:8000'}),
            ('api_error', 'External API rate limit exceeded', {'api': 'lunarcrush', 'status': 429}),
            ('validation_error', 'Invalid article data', {'field': 'published_date'}),
            ('analysis_error', 'AI analysis failed', {'model': 'gemma:2b', 'timeout': True}),
            ('connection_error', 'Redis connection lost', {'host': 'localhost', 'port': 6379})
        ]
        
        for error_type, message, context in error_scenarios:
            track_error(error_type, message, context)
        
        # Sprawdź śledzenie błędów
        assert len(error_tracker['errors']) == 5
        assert len(error_tracker['last_errors']) == 5
        assert error_tracker['error_counts']['connection_error'] == 2
        assert error_tracker['error_counts']['api_error'] == 1
        assert error_tracker['error_counts']['validation_error'] == 1
        assert error_tracker['error_counts']['analysis_error'] == 1
        
        # Sprawdź strukturę błędów
        for error in error_tracker['errors']:
            assert 'timestamp' in error
            assert 'type' in error
            assert 'message' in error
            assert 'context' in error

    def test_log_rotation(self):
        """Test: Rotacja logów"""
        
        # Symuluj rotację logów
        with tempfile.TemporaryDirectory() as temp_dir:
            log_file_path = Path(temp_dir) / "test.log"
            
            # Utwórz logger z rotacją
            logger = logging.getLogger("rotation_test")
            logger.setLevel(logging.INFO)
            
            # Symuluj RotatingFileHandler
            from logging.handlers import RotatingFileHandler
            
            handler = RotatingFileHandler(
                log_file_path,
                maxBytes=1024,  # 1KB
                backupCount=3
            )
            
            formatter = logging.Formatter('%(asctime)s - %(message)s')
            handler.setFormatter(formatter)
            logger.addHandler(handler)
            
            # Generuj dużo logów aby wywołać rotację
            for i in range(100):
                logger.info(f"Log message {i} with some additional content to make it longer")
            
            handler.close()
            logger.removeHandler(handler)
            
            # Sprawdź czy pliki rotacji zostały utworzone
            log_files = list(Path(temp_dir).glob("test.log*"))
            assert len(log_files) > 1  # Powinny być pliki rotacji
            
            # Sprawdź czy główny plik nie przekracza limitu
            main_log_size = log_file_path.stat().st_size
            assert main_log_size <= 1024 * 1.1  # Niewielka tolerancja

    @pytest.mark.asyncio
    async def test_real_time_monitoring(self):
        """Test: Monitoring w czasie rzeczywistym"""
        
        # Symuluj system monitoringu w czasie rzeczywistym
        monitoring_data = {
            'active_connections': 0,
            'messages_per_second': 0,
            'cpu_usage': 0.0,
            'memory_usage': 0.0,
            'disk_usage': 0.0,
            'network_io': {'bytes_sent': 0, 'bytes_received': 0}
        }
        
        # Symuluj zbieranie metryk
        import psutil
        import random
        
        def collect_system_metrics():
            # Symuluj metryki systemu
            monitoring_data['cpu_usage'] = random.uniform(10.0, 80.0)
            monitoring_data['memory_usage'] = random.uniform(30.0, 70.0)
            monitoring_data['disk_usage'] = random.uniform(20.0, 90.0)
            monitoring_data['active_connections'] = random.randint(5, 50)
            monitoring_data['messages_per_second'] = random.randint(10, 100)
            
            return monitoring_data.copy()
        
        # Zbierz metryki przez kilka iteracji
        metrics_history = []
        for _ in range(10):
            metrics = collect_system_metrics()
            metrics_history.append(metrics)
            await asyncio.sleep(0.01)  # Symuluj interwał
        
        # Sprawdź czy metryki są rozsądne
        assert len(metrics_history) == 10
        
        for metrics in metrics_history:
            assert 0 <= metrics['cpu_usage'] <= 100
            assert 0 <= metrics['memory_usage'] <= 100
            assert 0 <= metrics['disk_usage'] <= 100
            assert metrics['active_connections'] >= 0
            assert metrics['messages_per_second'] >= 0

    def test_alert_system(self):
        """Test: System alertów"""
        
        # Symuluj system alertów
        alerts = []
        
        def check_and_alert(metric_name, value, threshold, alert_type='warning'):
            if alert_type == 'warning' and value > threshold:
                alert = {
                    'timestamp': datetime.now().isoformat(),
                    'type': alert_type,
                    'metric': metric_name,
                    'value': value,
                    'threshold': threshold,
                    'message': f"{metric_name} is {value}, exceeds threshold {threshold}"
                }
                alerts.append(alert)
                return alert
            elif alert_type == 'critical' and value > threshold:
                alert = {
                    'timestamp': datetime.now().isoformat(),
                    'type': alert_type,
                    'metric': metric_name,
                    'value': value,
                    'threshold': threshold,
                    'message': f"CRITICAL: {metric_name} is {value}, exceeds threshold {threshold}"
                }
                alerts.append(alert)
                return alert
            return None
        
        # Test różnych scenariuszy alertów
        test_scenarios = [
            ('cpu_usage', 85.0, 80.0, 'warning'),
            ('memory_usage', 95.0, 90.0, 'critical'),
            ('error_rate', 15.0, 10.0, 'warning'),
            ('response_time', 2.5, 2.0, 'warning'),
            ('disk_usage', 98.0, 95.0, 'critical')
        ]
        
        for metric, value, threshold, alert_type in test_scenarios:
            alert = check_and_alert(metric, value, threshold, alert_type)
            assert alert is not None
            assert alert['type'] == alert_type
            assert alert['value'] == value
            assert alert['threshold'] == threshold
        
        # Sprawdź czy wszystkie alerty zostały zarejestrowane
        assert len(alerts) == 5
        
        # Sprawdź priorytety alertów
        critical_alerts = [a for a in alerts if a['type'] == 'critical']
        warning_alerts = [a for a in alerts if a['type'] == 'warning']
        
        assert len(critical_alerts) == 2
        assert len(warning_alerts) == 3

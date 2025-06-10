#!/usr/bin/env python3
# SolanaSniper 3.0 - Test Runner
# OPERACJA "STRESS TEST" - Skrypt uruchamiający wszystkie testy

import subprocess
import sys
import os
import time
from pathlib import Path

class TestRunner:
    """
    Zaawansowany runner testów dla SolanaSniper 3.0
    
    Uruchamia różne typy testów:
    - Unit tests
    - Integration tests  
    - Performance tests
    - Coverage analysis
    """
    
    def __init__(self):
        self.test_dir = Path(__file__).parent
        self.project_root = self.test_dir.parent
        
    def run_command(self, command, description):
        """Uruchamia komendę i wyświetla wyniki"""
        print(f"\n🔥 {description}")
        print("=" * 60)
        
        start_time = time.time()
        
        try:
            result = subprocess.run(
                command,
                shell=True,
                cwd=self.test_dir,
                capture_output=True,
                text=True,
                timeout=300  # 5 minut timeout
            )
            
            duration = time.time() - start_time
            
            if result.returncode == 0:
                print(f"✅ SUKCES ({duration:.2f}s)")
                if result.stdout:
                    print(result.stdout)
            else:
                print(f"❌ BŁĄD ({duration:.2f}s)")
                if result.stderr:
                    print("STDERR:", result.stderr)
                if result.stdout:
                    print("STDOUT:", result.stdout)
                    
            return result.returncode == 0
            
        except subprocess.TimeoutExpired:
            print(f"⏰ TIMEOUT po 5 minutach")
            return False
        except Exception as e:
            print(f"💥 WYJĄTEK: {e}")
            return False
    
    def install_dependencies(self):
        """Instaluje zależności testowe"""
        return self.run_command(
            "pip install -r requirements.txt",
            "INSTALACJA ZALEŻNOŚCI TESTOWYCH"
        )
    
    def run_unit_tests(self):
        """Uruchamia testy jednostkowe"""
        return self.run_command(
            "python -m pytest -v -m 'not integration and not performance' --tb=short",
            "TESTY JEDNOSTKOWE (UNIT TESTS)"
        )
    
    def run_integration_tests(self):
        """Uruchamia testy integracyjne"""
        return self.run_command(
            "python -m pytest -v -m integration --tb=short",
            "TESTY INTEGRACYJNE"
        )
    
    def run_performance_tests(self):
        """Uruchamia testy wydajności"""
        return self.run_command(
            "python -m pytest -v -m performance --tb=short --benchmark-only",
            "TESTY WYDAJNOŚCI"
        )
    
    def run_coverage_analysis(self):
        """Uruchamia analizę pokrycia kodu"""
        return self.run_command(
            "python -m pytest --cov=../agents --cov=../livestore --cov-report=html --cov-report=term",
            "ANALIZA POKRYCIA KODU"
        )
    
    def run_specific_test(self, test_file):
        """Uruchamia konkretny plik testowy"""
        return self.run_command(
            f"python -m pytest -v {test_file} --tb=short",
            f"TEST KONKRETNEGO PLIKU: {test_file}"
        )
    
    def run_stress_test(self):
        """Uruchamia test obciążeniowy"""
        return self.run_command(
            "python -m pytest -v test_stress.py --tb=short",
            "TEST OBCIĄŻENIOWY (STRESS TEST)"
        )
    
    def run_all_tests(self):
        """Uruchamia wszystkie testy w kolejności"""
        print("🚀 OPERACJA 'STRESS TEST' - ROZPOCZĘTA!")
        print("=" * 60)
        
        results = []
        
        # 1. Instalacja zależności
        results.append(("Instalacja zależności", self.install_dependencies()))
        
        # 2. Testy jednostkowe
        results.append(("Testy jednostkowe", self.run_unit_tests()))
        
        # 3. Testy integracyjne
        results.append(("Testy integracyjne", self.run_integration_tests()))
        
        # 4. Analiza pokrycia
        results.append(("Analiza pokrycia", self.run_coverage_analysis()))
        
        # 5. Testy wydajności
        results.append(("Testy wydajności", self.run_performance_tests()))
        
        # Podsumowanie
        self.print_summary(results)
        
        return all(result for _, result in results)
    
    def print_summary(self, results):
        """Wyświetla podsumowanie testów"""
        print("\n" + "=" * 60)
        print("📊 PODSUMOWANIE OPERACJI 'STRESS TEST'")
        print("=" * 60)
        
        total_tests = len(results)
        passed_tests = sum(1 for _, result in results if result)
        failed_tests = total_tests - passed_tests
        
        for test_name, result in results:
            status = "✅ PASS" if result else "❌ FAIL"
            print(f"{status} {test_name}")
        
        print("-" * 60)
        print(f"📈 WYNIKI: {passed_tests}/{total_tests} testów przeszło")
        print(f"🎯 SUKCES: {(passed_tests/total_tests)*100:.1f}%")
        
        if failed_tests == 0:
            print("\n🎉 OPERACJA 'STRESS TEST' ZAKOŃCZONA SUKCESEM!")
            print("🛡️ System jest gotowy do produkcji!")
        else:
            print(f"\n⚠️ WYKRYTO {failed_tests} PROBLEMÓW!")
            print("🔧 System wymaga napraw przed produkcją!")

def main():
    """Główna funkcja"""
    runner = TestRunner()
    
    if len(sys.argv) > 1:
        command = sys.argv[1]
        
        if command == "unit":
            runner.run_unit_tests()
        elif command == "integration":
            runner.run_integration_tests()
        elif command == "performance":
            runner.run_performance_tests()
        elif command == "coverage":
            runner.run_coverage_analysis()
        elif command == "stress":
            runner.run_stress_test()
        elif command.endswith(".py"):
            runner.run_specific_test(command)
        else:
            print("❌ Nieznana komenda!")
            print("Dostępne komendy:")
            print("  unit         - testy jednostkowe")
            print("  integration  - testy integracyjne") 
            print("  performance  - testy wydajności")
            print("  coverage     - analiza pokrycia")
            print("  stress       - test obciążeniowy")
            print("  <file.py>    - konkretny plik testowy")
            print("  (brak)       - wszystkie testy")
    else:
        # Uruchom wszystkie testy
        success = runner.run_all_tests()
        sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()

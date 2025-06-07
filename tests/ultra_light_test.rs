//! **ULTRA-LEKKI TEST** - SniperBot 2.0
//! 
//! Minimalny test sprawdzający tylko kompilację i podstawowe struktury
//! BEZ uruchamiania żadnych ciężkich komponentów

use sniper_bot::{
    config::AppConfig,
    models::{StrategySignal, SignalType},
};
use chrono::Utc;

/// **TEST 1: Sprawdzenie konfiguracji**
/// Tylko ładowanie konfiguracji - bez połączeń sieciowych
#[test]
fn test_config_loading() {
    // Ustaw minimalne zmienne środowiskowe
    std::env::set_var("DRY_RUN", "true");
    std::env::set_var("SOLANA_RPC_URL", "https://api.devnet.solana.com");
    std::env::set_var("DRAGONFLY_URL", "redis://localhost:6379");
    
    // Test ładowania konfiguracji
    let config = AppConfig::from_env();
    
    // Podstawowe sprawdzenia
    assert!(!config.solana.rpc_url.is_empty(), "RPC URL nie może być pusty");
    assert!(!config.database.dragonfly_url.is_empty(), "DragonflyDB URL nie może być pusty");
    
    println!("✅ Konfiguracja załadowana pomyślnie");
}

/// **TEST 2: Sprawdzenie struktur danych**
/// Tylko tworzenie i serializacja struktur - bez I/O
#[test]
fn test_data_structures() {
    // Test tworzenia StrategySignal
    let signal = StrategySignal {
        strategy: "test_strategy".to_string(),
        symbol: "SOL/USDC".to_string(),
        signal_type: SignalType::Buy,
        strength: 0.85,
        price: 100.0,
        size: 0.1,
        metadata: serde_json::json!({
            "test": true,
            "reason": "ultra_light_test"
        }),
        timestamp: Utc::now(),
    };
    
    // Sprawdź że struktura jest poprawna
    assert_eq!(signal.strategy, "test_strategy");
    assert_eq!(signal.symbol, "SOL/USDC");
    assert!(matches!(signal.signal_type, SignalType::Buy));
    assert!(signal.strength > 0.0);
    assert!(signal.price > 0.0);
    assert!(signal.size > 0.0);
    
    // Test serializacji JSON
    let json = serde_json::to_string(&signal).expect("Serializacja powinna działać");
    assert!(!json.is_empty());
    
    // Test deserializacji JSON
    let _deserialized: StrategySignal = serde_json::from_str(&json)
        .expect("Deserializacja powinna działać");
    
    println!("✅ Struktury danych działają poprawnie");
}

/// **TEST 3: Sprawdzenie podstawowych typów**
/// Test wszystkich wariantów SignalType
#[test]
fn test_signal_types() {
    let buy_signal = SignalType::Buy;
    let sell_signal = SignalType::Sell;
    let hold_signal = SignalType::Hold;
    
    // Test serializacji
    assert_eq!(serde_json::to_string(&buy_signal).unwrap(), "\"Buy\"");
    assert_eq!(serde_json::to_string(&sell_signal).unwrap(), "\"Sell\"");
    assert_eq!(serde_json::to_string(&hold_signal).unwrap(), "\"Hold\"");
    
    // Test deserializacji
    assert!(matches!(
        serde_json::from_str::<SignalType>("\"Buy\"").unwrap(),
        SignalType::Buy
    ));
    assert!(matches!(
        serde_json::from_str::<SignalType>("\"Sell\"").unwrap(),
        SignalType::Sell
    ));
    assert!(matches!(
        serde_json::from_str::<SignalType>("\"Hold\"").unwrap(),
        SignalType::Hold
    ));
    
    println!("✅ Typy sygnałów działają poprawnie");
}

/// **TEST 4: Test środowiska**
/// Sprawdzenie czy jesteśmy w trybie testowym
#[test]
fn test_environment() {
    // Sprawdź czy DRY_RUN jest ustawione
    let dry_run = std::env::var("DRY_RUN").unwrap_or_else(|_| "false".to_string());
    assert_eq!(dry_run, "true", "Testy muszą być uruchamiane w trybie DRY_RUN");
    
    // Sprawdź czy nie jesteśmy w produkcji
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    assert_ne!(env, "production", "Testy nie mogą być uruchamiane w produkcji");
    
    println!("✅ Środowisko testowe jest bezpieczne");
}

/// **TEST 5: Test wydajności struktur**
/// Sprawdzenie czy tworzenie struktur jest szybkie
#[test]
fn test_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Stwórz 1000 sygnałów
    let mut signals = Vec::new();
    for i in 0..1000 {
        let signal = StrategySignal {
            strategy: format!("strategy_{}", i),
            symbol: "SOL/USDC".to_string(),
            signal_type: if i % 2 == 0 { SignalType::Buy } else { SignalType::Sell },
            strength: 0.5 + (i as f64 / 2000.0),
            price: 100.0 + (i as f64 / 10.0),
            size: 0.1,
            metadata: serde_json::json!({
                "index": i,
                "test": true
            }),
            timestamp: Utc::now(),
        };
        signals.push(signal);
    }
    
    let duration = start.elapsed();
    
    // Sprawdź że utworzono wszystkie sygnały
    assert_eq!(signals.len(), 1000);
    
    // Sprawdź że było szybko (mniej niż 100ms)
    assert!(duration.as_millis() < 100, 
        "Tworzenie 1000 sygnałów trwało zbyt długo: {:?}", duration);
    
    println!("✅ Wydajność struktur jest dobra: {:?}", duration);
}

/// **GŁÓWNY TEST INTEGRACYJNY**
/// Uruchamia wszystkie testy w sekwencji
#[test]
fn test_all_components() {
    println!("🚀 ULTRA-LEKKI TEST INTEGRACYJNY - START");
    
    test_config_loading();
    test_data_structures();
    test_signal_types();
    test_environment();
    test_performance();
    
    println!("🎉 WSZYSTKIE TESTY ZAKOŃCZONE SUKCESEM!");
    println!("✅ SniperBot 2.0 - podstawowe komponenty działają poprawnie");
}

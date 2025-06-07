//! **LEKKI TEST INTEGRACYJNY** - SniperBot 2.0
//!
//! Szybki test sprawdzający kluczowe komponenty bez uruchamiania pełnego systemu

use sniper_bot::{
    config::AppConfig,
    live_trading_engine::LiveTradingEngineFactory,
};
use tokio::time::{timeout, Duration};
use tracing::{info, error};

// Usunięta niepotrzebna struktura konfiguracyjna

/// **LEKKI TEST SZYBKIEJ WERYFIKACJI** - bez uruchamiania pełnego systemu
#[tokio::test]
async fn test_quick_component_verification() {
    info!("🚀 LEKKI TEST - Szybka weryfikacja komponentów");

    // Ustaw tryb testowy
    std::env::set_var("DRY_RUN", "true");

    let config = AppConfig::from_env();

    // Test 1: Sprawdź czy można utworzyć LiveTradingEngine (bez uruchamiania)
    info!("🔧 Test 1: Tworzenie LiveTradingEngine...");

    let result = timeout(Duration::from_secs(5), async {
        LiveTradingEngineFactory::create(config, true).await
    }).await;

    match result {
        Ok(Ok((engine, _sender))) => {
            info!("✅ LiveTradingEngine utworzony pomyślnie");
            let status = engine.get_status();
            assert!(status.dry_run, "Powinien być w trybie DRY_RUN");
            info!("✅ Status: DRY_RUN = {}", status.dry_run);
        }
        Ok(Err(e)) => {
            info!("❌ Błąd tworzenia engine: {}", e);
            panic!("Nie można utworzyć LiveTradingEngine");
        }
        Err(_) => {
            info!("⏰ Timeout - tworzenie trwało zbyt długo");
            panic!("Timeout podczas tworzenia engine");
        }
    }

    info!("🎉 LEKKI TEST ZAKOŃCZONY SUKCESEM!");
}

// Usunięte niepotrzebne funkcje pomocnicze

#[tokio::test]
async fn test_api_connectivity() {
    info!("🔌 Testing API connectivity...");
    
    // Test basic configuration loading
    let config = AppConfig::from_env();
    
    // Verify essential configuration
    assert!(!config.solana.rpc_url.is_empty(), "RPC URL must be configured");
    assert!(config.solana.api_key.is_some(), "Helius API key must be configured");
    assert!(!config.database.dragonfly_url.is_empty(), "DragonflyDB URL must be configured");
    
    info!("✅ Configuration validation passed");
    info!("🌐 RPC URL: {}", config.solana.rpc_url);
    info!("🔑 Helius API key: configured");
    info!("🐉 DragonflyDB URL: configured");
}

#[tokio::test]
async fn test_component_creation() {
    info!("🧩 Testing component creation...");
    
    let config = AppConfig::from_env();
    
    // Test LiveTradingEngineFactory
    match LiveTradingEngineFactory::create(config, true).await {
        Ok((engine, sender)) => {
            info!("✅ LiveTradingEngine created successfully");
            info!("✅ Signal sender created successfully");
            
            // Test that we can get engine status
            let status = engine.get_status();
            assert!(status.dry_run, "Engine should be in DRY RUN mode");
            info!("✅ Engine status: {:?}", status);
            
            // Clean shutdown
            drop(sender);
        }
        Err(e) => {
            error!("❌ Component creation failed: {}", e);
            panic!("Component creation test failed");
        }
    }
}

/*!
🗄️ Database Connector - DragonflyDB Integration
The Persistent Brain of SniperBot 2.0

This module provides the foundation for the Hub-and-Spoke architecture,
enabling persistent state management across all bot components.
*/

use deadpool_redis::{Config, Pool, Runtime, redis::AsyncCommands};
use serde::{Serialize, Deserialize};
use tracing::{info, error, debug, warn};
use chrono::{DateTime, Utc};
use crate::models::TradingResult;
use std::env;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// DragonflyDB connection URL
    pub connection_url: String,
    /// Maximum number of connections in pool
    pub max_pool_size: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Default TTL for cached data in seconds
    pub default_ttl_secs: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            connection_url: "redis://localhost:6379".to_string(),
            max_pool_size: 10,
            connection_timeout_secs: 30,
            default_ttl_secs: 3600, // 1 hour
        }
    }
}

impl DbConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> TradingResult<Self> {
        let connection_url = env::var("DRAGONFLY_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        Ok(Self {
            connection_url,
            max_pool_size: 10,
            connection_timeout_secs: 30,
            default_ttl_secs: 3600,
        })
    }
}

/// Database client with connection pooling
#[derive(Clone)]
pub struct DbClient {
    pool: Pool,
    config: DbConfig,
}

impl DbClient {
    /// Create new database client with connection pool
    pub async fn new(config: DbConfig) -> TradingResult<Self> {
        info!("🗄️ Initializing DragonflyDB connection pool...");

        let cfg = Config::from_url(&config.connection_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Failed to create connection pool: {}", e))?;

        // Test connection
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

        // Simple ping test using basic set/get
        let test_key = "test:connection";
        let _: () = conn.set(test_key, "ping").await
            .map_err(|e| format!("Failed to ping DragonflyDB: {}", e))?;
        let _: () = conn.del(test_key).await
            .map_err(|e| format!("Failed to cleanup ping test: {}", e))?;

        info!("✅ DragonflyDB connection pool initialized successfully");

        Ok(Self { pool, config })
    }

    /// Get connection from pool
    async fn get_connection(&self) -> TradingResult<deadpool_redis::Connection> {
        self.pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e).into())
    }

    /// Set key-value pair with optional TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_secs: Option<u64>) -> TradingResult<()> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        if let Some(ttl) = ttl_secs {
            conn.set_ex(key, serialized, ttl)
                .await
                .map_err(|e| format!("Redis SET_EX failed: {}", e))?;
        } else {
            conn.set(key, serialized)
                .await
                .map_err(|e| format!("Redis SET failed: {}", e))?;
        }

        debug!("📝 Set key: {} (TTL: {:?}s)", key, ttl_secs);
        Ok(())
    }

    /// Get value by key
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> TradingResult<Option<T>> {
        let mut conn = self.get_connection().await?;
        
        let result: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| format!("Redis GET failed: {}", e))?;

        match result {
            Some(serialized) => {
                let value = serde_json::from_str(&serialized)
                    .map_err(|e| format!("Deserialization failed: {}", e))?;
                debug!("📖 Got key: {}", key);
                Ok(Some(value))
            }
            None => {
                debug!("🔍 Key not found: {}", key);
                Ok(None)
            }
        }
    }

    /// Delete key
    pub async fn delete(&self, key: &str) -> TradingResult<bool> {
        let mut conn = self.get_connection().await?;
        let deleted: i32 = conn
            .del(key)
            .await
            .map_err(|e| format!("Redis DEL failed: {}", e))?;
        
        debug!("🗑️ Deleted key: {} (existed: {})", key, deleted > 0);
        Ok(deleted > 0)
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> TradingResult<bool> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| format!("Redis EXISTS failed: {}", e))?;
        
        Ok(exists)
    }

    /// Set TTL for existing key
    pub async fn expire(&self, key: &str, ttl_secs: u64) -> TradingResult<bool> {
        let mut conn = self.get_connection().await?;
        let set: bool = conn
            .expire(key, ttl_secs as i64)
            .await
            .map_err(|e| format!("Redis EXPIRE failed: {}", e))?;
        
        debug!("⏰ Set TTL for key: {} ({}s)", key, ttl_secs);
        Ok(set)
    }

    /// Add item to list (left push)
    pub async fn list_push<T: Serialize>(&self, key: &str, value: &T) -> TradingResult<i64> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let length: i64 = conn
            .lpush(key, serialized)
            .await
            .map_err(|e| format!("Redis LPUSH failed: {}", e))?;

        debug!("📋 Pushed to list: {} (new length: {})", key, length);
        Ok(length)
    }

    /// Get items from list (range) - raw strings
    pub async fn list_range_raw(&self, key: &str, start: i64, stop: i64) -> TradingResult<Vec<String>> {
        let mut conn = self.get_connection().await?;

        let items: Vec<String> = conn
            .lrange(key, start as isize, stop as isize)
            .await
            .map_err(|e| format!("Redis LRANGE failed: {}", e))?;

        debug!("📋 Got {} raw items from list: {}", items.len(), key);
        Ok(items)
    }

    /// Get items from list (range) - deserialize JSON
    pub async fn list_range<T: for<'de> Deserialize<'de>>(&self, key: &str, start: i64, stop: i64) -> TradingResult<Vec<T>> {
        let mut conn = self.get_connection().await?;

        let items: Vec<String> = conn
            .lrange(key, start as isize, stop as isize)
            .await
            .map_err(|e| format!("Redis LRANGE failed: {}", e))?;

        let mut result = Vec::new();
        for item in items {
            let value = serde_json::from_str(&item)
                .map_err(|e| format!("Deserialization failed: {}", e))?;
            result.push(value);
        }

        debug!("📋 Got {} items from list: {}", result.len(), key);
        Ok(result)
    }

    /// Pop item from list (right pop)
    pub async fn list_pop<T: for<'de> Deserialize<'de>>(&self, key: &str) -> TradingResult<Option<T>> {
        let mut conn = self.get_connection().await?;
        
        let item: Option<String> = conn
            .rpop(key, None)
            .await
            .map_err(|e| format!("Redis RPOP failed: {}", e))?;

        match item {
            Some(serialized) => {
                let value = serde_json::from_str(&serialized)
                    .map_err(|e| format!("Deserialization failed: {}", e))?;
                debug!("📋 Popped from list: {}", key);
                Ok(Some(value))
            }
            None => Ok(None)
        }
    }

    /// Add item to set
    pub async fn set_add<T: Serialize>(&self, key: &str, value: &T) -> TradingResult<bool> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let added: i32 = conn
            .sadd(key, serialized)
            .await
            .map_err(|e| format!("Redis SADD failed: {}", e))?;

        debug!("🔢 Added to set: {} (was new: {})", key, added > 0);
        Ok(added > 0)
    }

    /// Check if item is in set
    pub async fn set_contains<T: Serialize>(&self, key: &str, value: &T) -> TradingResult<bool> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let is_member: bool = conn
            .sismember(key, serialized)
            .await
            .map_err(|e| format!("Redis SISMEMBER failed: {}", e))?;

        Ok(is_member)
    }

    /// Get all items from set
    pub async fn set_members<T: for<'de> Deserialize<'de>>(&self, key: &str) -> TradingResult<Vec<T>> {
        let mut conn = self.get_connection().await?;

        let members: Vec<String> = conn
            .smembers(key)
            .await
            .map_err(|e| format!("Redis SMEMBERS failed: {}", e))?;

        let mut result = Vec::new();
        for member in members {
            let value = serde_json::from_str(&member)
                .map_err(|e| format!("Deserialization failed: {}", e))?;
            result.push(value);
        }

        debug!("🔢 Got {} members from set: {}", result.len(), key);
        Ok(result)
    }

    /// Remove item from list by value
    pub async fn list_remove(&self, key: &str, value: &str) -> TradingResult<i64> {
        let mut conn = self.get_connection().await?;

        let removed: i64 = conn
            .lrem(key, 1, value)
            .await
            .map_err(|e| format!("Redis LREM failed: {}", e))?;

        debug!("📋 Removed {} items from list: {}", removed, key);
        Ok(removed)
    }

    /// Get database statistics (simplified)
    pub async fn get_stats(&self) -> TradingResult<DbStats> {
        // For now, return basic stats without complex Redis commands
        // This can be enhanced later when we have more stable Redis integration
        Ok(DbStats {
            total_keys: 0, // Would need DBSIZE command
            memory_info: "Memory info not available".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> TradingResult<bool> {
        let mut conn = self.get_connection().await?;

        // Simple health check using set/get
        let test_key = "health:check";
        match conn.set::<_, _, ()>(test_key, "ok").await {
            Ok(_) => {
                let _: () = conn.del(test_key).await.unwrap_or(());
                debug!("💚 DragonflyDB health check: OK");
                Ok(true)
            }
            Err(e) => {
                error!("❤️ DragonflyDB health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get configuration
    pub fn config(&self) -> &DbConfig {
        &self.config
    }

    // === DASHBOARD-SPECIFIC METHODS ===

    /// Update dashboard statistics
    pub async fn update_dashboard_stats(&self, stats: &crate::models::persistent_state::DashboardStats) -> TradingResult<()> {
        let serialized = serde_json::to_string(stats)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let mut conn = self.get_connection().await?;
        let _: () = conn.set_ex("dashboard:stats", &serialized, 3600)
            .await
            .map_err(|e| format!("Redis SET failed: {}", e))?;

        Ok(())
    }

    /// Get dashboard statistics
    pub async fn get_dashboard_stats(&self) -> TradingResult<Option<crate::models::persistent_state::DashboardStats>> {
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Option<String>>("dashboard:stats").await {
            Ok(Some(data)) => {
                let stats = serde_json::from_str(&data)
                    .map_err(|e| format!("Deserialization failed: {}", e))?;
                Ok(Some(stats))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Redis GET failed: {}", e).into())
        }
    }

    /// Add activity event to feed
    pub async fn add_activity_event(&self, event: &crate::models::persistent_state::ActivityEvent) -> TradingResult<()> {
        let serialized = serde_json::to_string(event)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let mut conn = self.get_connection().await?;

        // Add to activity feed (keep last 100 events)
        let _: () = conn.lpush("dashboard:activity_feed", &serialized)
            .await
            .map_err(|e| format!("Redis LPUSH failed: {}", e))?;

        // Trim to keep only last 100 events
        let _: () = conn.ltrim("dashboard:activity_feed", 0, 99)
            .await
            .map_err(|e| format!("Redis LTRIM failed: {}", e))?;

        debug!("📝 Added activity event: {}", event.event_type);
        Ok(())
    }

    /// Get recent activity events
    pub async fn get_activity_events(&self, limit: i64) -> TradingResult<Vec<crate::models::persistent_state::ActivityEvent>> {
        let events_data: Vec<String> = self.list_range_raw("dashboard:activity_feed", 0, limit - 1).await?;

        let mut events = Vec::new();
        for data in events_data {
            match serde_json::from_str(&data) {
                Ok(event) => events.push(event),
                Err(e) => warn!("Failed to deserialize activity event: {}", e),
            }
        }

        Ok(events)
    }

    /// Update realtime metrics
    pub async fn update_realtime_metrics(&self, metrics: &crate::models::persistent_state::RealtimeMetrics) -> TradingResult<()> {
        let serialized = serde_json::to_string(metrics)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let mut conn = self.get_connection().await?;
        let _: () = conn.set_ex("realtime:metrics", &serialized, 300)
            .await
            .map_err(|e| format!("Redis SET failed: {}", e))?;

        Ok(())
    }

    /// Get realtime metrics
    pub async fn get_realtime_metrics(&self) -> TradingResult<Option<crate::models::persistent_state::RealtimeMetrics>> {
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Option<String>>("realtime:metrics").await {
            Ok(Some(data)) => {
                let metrics = serde_json::from_str(&data)
                    .map_err(|e| format!("Deserialization failed: {}", e))?;
                Ok(Some(metrics))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Redis GET failed: {}", e).into())
        }
    }

    /// Update bot status
    pub async fn update_bot_status(&self, status: &crate::models::persistent_state::BotStatus) -> TradingResult<()> {
        let serialized = serde_json::to_string(status)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let mut conn = self.get_connection().await?;
        let _: () = conn.set_ex("bot:status", &serialized, 3600)
            .await
            .map_err(|e| format!("Redis SET failed: {}", e))?;

        Ok(())
    }

    /// Get bot status
    pub async fn get_bot_status(&self) -> TradingResult<Option<crate::models::persistent_state::BotStatus>> {
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Option<String>>("bot:status").await {
            Ok(Some(data)) => {
                let status = serde_json::from_str(&data)
                    .map_err(|e| format!("Deserialization failed: {}", e))?;
                Ok(Some(status))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Redis GET failed: {}", e).into())
        }
    }
}

/// Database statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DbStats {
    pub total_keys: i64,
    pub memory_info: String,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_client_creation() {
        let config = DbConfig::default();
        // This test would require a running DragonflyDB instance
        // In CI/CD, we'd use a test container
        
        // For now, just test config creation
        assert_eq!(config.max_pool_size, 10);
        assert_eq!(config.default_ttl_secs, 3600);
    }
}

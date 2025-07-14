use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions, PgConnectOptions};
use sqlx::{ConnectOptions, Connection, PgConnection};
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub connect_timeout_seconds: u64,
    pub enable_logging: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://admin:welcome@localhost:5432/healthcare_emergency".to_string(),
            max_connections: 100,
            min_connections: 5,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600, // 10 minutes
            max_lifetime_seconds: 1800, // 30 minutes
            connect_timeout_seconds: 10,
            enable_logging: false, // Set to true for development
        }
    }
}

impl DatabaseConfig {
    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable is required")?;

        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .context("Invalid DB_MAX_CONNECTIONS value")?;

        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("Invalid DB_MIN_CONNECTIONS value")?;

        let acquire_timeout_seconds = std::env::var("DB_ACQUIRE_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .context("Invalid DB_ACQUIRE_TIMEOUT value")?;

        let idle_timeout_seconds = std::env::var("DB_IDLE_TIMEOUT")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .context("Invalid DB_IDLE_TIMEOUT value")?;

        let max_lifetime_seconds = std::env::var("DB_MAX_LIFETIME")
            .unwrap_or_else(|_| "1800".to_string())
            .parse()
            .context("Invalid DB_MAX_LIFETIME value")?;

        let connect_timeout_seconds = std::env::var("DB_CONNECT_TIMEOUT")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("Invalid DB_CONNECT_TIMEOUT value")?;

        let enable_logging = std::env::var("DB_ENABLE_LOGGING")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Ok(Self {
            url,
            max_connections,
            min_connections,
            acquire_timeout_seconds,
            idle_timeout_seconds,
            max_lifetime_seconds,
            connect_timeout_seconds,
            enable_logging,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            anyhow::bail!("Database URL cannot be empty");
        }

        if !self.url.starts_with("postgresql://") && !self.url.starts_with("postgres://") {
            anyhow::bail!("Database URL must be a PostgreSQL connection string");
        }

        if self.max_connections == 0 {
            anyhow::bail!("max_connections must be greater than 0");
        }

        if self.min_connections > self.max_connections {
            anyhow::bail!("min_connections cannot be greater than max_connections");
        }

        if self.acquire_timeout_seconds == 0 {
            anyhow::bail!("acquire_timeout_seconds must be greater than 0");
        }

        Ok(())
    }

    /// Create connection pool
    pub async fn create_pool(&self) -> Result<PgPool> {
        self.validate()
            .context("Database configuration validation failed")?;

        info!("Creating database connection pool with {} max connections", self.max_connections);

        let mut options = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(Duration::from_secs(self.acquire_timeout_seconds))
            .idle_timeout(Duration::from_secs(self.idle_timeout_seconds))
            .max_lifetime(Duration::from_secs(self.max_lifetime_seconds));

        // Configure connection options
        let mut connect_options: PgConnectOptions = self.url.parse()
            .context("Failed to parse database URL")?;

        if self.enable_logging {
            connect_options = connect_options.log_statements(tracing::log::LevelFilter::Info);
        }

        connect_options = connect_options
            .statement_cache_capacity(100)
            .application_name("dubai-healthcare-emergency");

        // Create pool with retry logic
        let pool = self.create_pool_with_retry(options, connect_options).await?;

        info!("Database connection pool created successfully");
        Ok(pool)
    }

    /// Create pool with retry logic
    async fn create_pool_with_retry(
        &self,
        options: PgPoolOptions,
        connect_options: PgConnectOptions,
    ) -> Result<PgPool> {
        let max_retries = 5;
        let mut retry_count = 0;

        loop {
            match options.clone().connect_with(connect_options.clone()).await {
                Ok(pool) => return Ok(pool),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(anyhow::anyhow!(
                            "Failed to create database pool after {} retries: {}",
                            max_retries,
                            e
                        ));
                    }

                    warn!(
                        "Failed to create database pool (attempt {}/{}): {}. Retrying in 2 seconds...",
                        retry_count, max_retries, e
                    );

                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        info!("Testing database connection...");

        let connect_options: PgConnectOptions = self.url.parse()
            .context("Failed to parse database URL")?;

        let mut connection = PgConnection::connect_with(&connect_options)
            .await
            .context("Failed to connect to database")?;

        // Test with a simple query
        sqlx::query("SELECT 1")
            .execute(&mut connection)
            .await
            .context("Failed to execute test query")?;

        connection.close().await?;

        info!("Database connection test successful");
        Ok(())
    }

    /// Get database name from URL
    pub fn database_name(&self) -> Option<&str> {
        self.url.split('/').last()?.split('?').next()
    }

    /// Get host from URL
    pub fn host(&self) -> Option<String> {
        let url = &self.url;
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(at_pos) = after_protocol.find('@') {
                let after_auth = &after_protocol[at_pos + 1..];
                if let Some(end) = after_auth.find('/') {
                    return Some(after_auth[..end].to_string());
                } else if let Some(end) = after_auth.find('?') {
                    return Some(after_auth[..end].to_string());
                } else {
                    return Some(after_auth.to_string());
                }
            }
        }
        None
    }
}

/// Database health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub database_name: Option<String>,
    pub host: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl DatabaseHealth {
    /// Perform database health check
    pub async fn check(pool: &PgPool, config: &DatabaseConfig) -> Self {
        let start_time = std::time::Instant::now();
        let timestamp = chrono::Utc::now();

        let status = match Self::perform_health_check(pool).await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                if response_time > 5000 {
                    HealthStatus::Degraded // Slow response
                } else {
                    HealthStatus::Healthy
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        };

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        // Get pool statistics
        let total_connections = pool.size();
        let idle_connections = pool.num_idle() as u32;
        let active_connections = total_connections - idle_connections as u32;

        Self {
            status,
            response_time_ms,
            active_connections,
            idle_connections,
            total_connections,
            database_name: config.database_name().map(|s| s.to_string()),
            host: config.host(),
            timestamp,
        }
    }

    /// Perform actual health check query
    async fn perform_health_check(pool: &PgPool) -> Result<()> {
        sqlx::query("SELECT 1 as health_check")
            .execute(pool)
            .await
            .context("Health check query failed")?;
        Ok(())
    }

    /// Check if database is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    /// Check if database is available (healthy or degraded)
    pub fn is_available(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert!(config.url.contains("postgresql://"));
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 5);
    }

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::default();
        
        // Valid config
        assert!(config.validate().is_ok());
        
        // Invalid URL
        config.url = "invalid_url".to_string();
        assert!(config.validate().is_err());
        
        // Reset URL
        config.url = "postgresql://localhost/test".to_string();
        assert!(config.validate().is_ok());
        
        // Invalid max connections
        config.max_connections = 0;
        assert!(config.validate().is_err());
        
        // Reset max connections
        config.max_connections = 10;
        
        // Invalid min > max
        config.min_connections = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_name_extraction() {
        let config = DatabaseConfig {
            url: "postgresql://user:pass@localhost:5432/healthcare_emergency?sslmode=require".to_string(),
            ..Default::default()
        };
        
        assert_eq!(config.database_name(), Some("healthcare_emergency"));
    }

    #[test]
    fn test_host_extraction() {
        let config = DatabaseConfig {
            url: "postgresql://user:pass@localhost:5432/healthcare_emergency".to_string(),
            ..Default::default()
        };
        
        assert_eq!(config.host(), Some("localhost:5432".to_string()));
    }

    #[test]
    fn test_config_from_env_missing_url() {
        // Clear any existing DATABASE_URL
        std::env::remove_var("DATABASE_URL");
        
        let result = DatabaseConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DATABASE_URL"));
    }

    #[test]
fn test_health_status_checks() {
    let health = DatabaseHealth {
        status: HealthStatus::Healthy,
        response_time_ms: 100,
        active_connections: 5,
        idle_connections: 10,
        total_connections: 15,
        database_name: Some("test".to_string()),
        host: Some("localhost".to_string()),
        timestamp: chrono::Utc::now(),
    };

    assert!(health.is_healthy());
    assert!(health.is_available());

    let unhealthy = DatabaseHealth {
        status: HealthStatus::Unhealthy,
        ..health.clone() // Clone health to avoid moving
    };

    assert!(!unhealthy.is_healthy());
    assert!(!unhealthy.is_available());

    let degraded = DatabaseHealth {
        status: HealthStatus::Degraded,
        ..health // Now safe to use health since we cloned it earlier
    };

    assert!(!degraded.is_healthy());
    assert!(degraded.is_available());
}
}
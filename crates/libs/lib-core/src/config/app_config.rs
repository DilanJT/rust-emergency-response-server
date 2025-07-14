use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

use super::database::DatabaseConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub redis: RedisConfig,
    pub logging: LoggingConfig,
    pub healthcare: HealthcareConfig,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout_seconds: u64,
    pub max_request_size_mb: usize,
    pub enable_metrics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_seconds: i64,
    pub refresh_expiration_seconds: i64,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub enable_console: bool,
    pub enable_file: bool,
    pub file_path: Option<String>,
    pub enable_request_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthcareConfig {
    pub hospital_name: String,
    pub hospital_id: String,
    pub dha_integration_enabled: bool,
    pub dha_api_url: Option<String>,
    pub dha_api_key: Option<String>,
    pub emergency_contact_required: bool,
    pub max_patient_age: u16,
    pub default_session_timeout_minutes: u32,
    pub enable_triage_ai: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            jwt: JwtConfig::default(),
            redis: RedisConfig::default(),
            logging: LoggingConfig::default(),
            healthcare: HealthcareConfig::default(),
            environment: Environment::Development,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            cors_origins: vec!["http://localhost:3000".to_string()],
            request_timeout_seconds: 30,
            max_request_size_mb: 10,
            enable_metrics: true,
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-super-secret-key-change-this-in-production".to_string(),
            expiration_seconds: 3600,        // 1 hour
            refresh_expiration_seconds: 86400, // 24 hours
            issuer: "dubai-healthcare-emergency".to_string(),
            audience: "healthcare-staff".to_string(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 50,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 5,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            enable_console: true,
            enable_file: false,
            file_path: None,
            enable_request_logging: true,
        }
    }
}

impl Default for HealthcareConfig {
    fn default() -> Self {
        Self {
            hospital_name: "Dubai Hospital".to_string(),
            hospital_id: "DHA-001".to_string(),
            dha_integration_enabled: false, // Disabled by default for development
            dha_api_url: None,
            dha_api_key: None,
            emergency_contact_required: true,
            max_patient_age: 150,
            default_session_timeout_minutes: 480, // 8 hours
            enable_triage_ai: false, // Disabled by default
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if it exists

        let environment = Self::detect_environment();
        
        let config = Self {
            server: ServerConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            jwt: JwtConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            logging: LoggingConfig::from_env(&environment)?,
            healthcare: HealthcareConfig::from_env()?,
            environment,
        };

        config.validate()?;
        Ok(config)
    }

    /// Detect environment from ENV variable
    fn detect_environment() -> Environment {
        match env::var("ENVIRONMENT")
            .or_else(|_| env::var("APP_ENV"))
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            "testing" | "test" => Environment::Testing,
            _ => Environment::Development,
        }
    }

    /// Validate the complete configuration
    pub fn validate(&self) -> Result<()> {
        self.server.validate()?;
        self.database.validate()?;
        self.jwt.validate()?;
        self.redis.validate()?;
        self.logging.validate()?;
        self.healthcare.validate()?;
        Ok(())
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    /// Get configuration as JSON (with secrets redacted)
    pub fn to_json_redacted(&self) -> Result<String> {
        let mut config = self.clone();
        config.jwt.secret = "[REDACTED]".to_string();
        if let Some(ref mut api_key) = config.healthcare.dha_api_key {
            *api_key = "[REDACTED]".to_string();
        }
        serde_json::to_string_pretty(&config).context("Failed to serialize config")
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("Invalid SERVER_PORT")?,
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            request_timeout_seconds: env::var("REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid REQUEST_TIMEOUT")?,
            max_request_size_mb: env::var("MAX_REQUEST_SIZE_MB")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid MAX_REQUEST_SIZE_MB")?,
            enable_metrics: env::var("ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            anyhow::bail!("Server host cannot be empty");
        }
        if self.port == 0 {
            anyhow::bail!("Server port must be greater than 0");
        }
        if self.request_timeout_seconds == 0 {
            anyhow::bail!("Request timeout must be greater than 0");
        }
        Ok(())
    }
}

impl JwtConfig {
    fn from_env() -> Result<Self> {
        let secret = env::var("JWT_SECRET")
            .context("JWT_SECRET environment variable is required")?;

        if secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters long");
        }

        Ok(Self {
            secret,
            expiration_seconds: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .context("Invalid JWT_EXPIRATION")?,
            refresh_expiration_seconds: env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .context("Invalid JWT_REFRESH_EXPIRATION")?,
            issuer: env::var("JWT_ISSUER")
                .unwrap_or_else(|_| "dubai-healthcare-emergency".to_string()),
            audience: env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "healthcare-staff".to_string()),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.secret.len() < 32 {
            anyhow::bail!("JWT secret must be at least 32 characters");
        }
        if self.expiration_seconds <= 0 {
            anyhow::bail!("JWT expiration must be positive");
        }
        Ok(())
    }
}

impl RedisConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            max_connections: env::var("REDIS_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .context("Invalid REDIS_MAX_CONNECTIONS")?,
            connection_timeout_seconds: env::var("REDIS_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid REDIS_CONNECTION_TIMEOUT")?,
            command_timeout_seconds: env::var("REDIS_COMMAND_TIMEOUT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid REDIS_COMMAND_TIMEOUT")?,
        })
    }

    fn validate(&self) -> Result<()> {
        if !self.url.starts_with("redis://") && !self.url.starts_with("rediss://") {
            anyhow::bail!("Redis URL must start with redis:// or rediss://");
        }
        Ok(())
    }
}

impl LoggingConfig {
    fn from_env(environment: &Environment) -> Result<Self> {
        let default_level = match environment {
            Environment::Production => "warn",
            Environment::Staging => "info",
            _ => "debug",
        };

        let format = match env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            _ => LogFormat::Pretty,
        };

        Ok(Self {
            level: env::var("RUST_LOG").unwrap_or_else(|_| default_level.to_string()),
            format,
            enable_console: env::var("LOG_CONSOLE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_file: env::var("LOG_FILE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            file_path: env::var("LOG_FILE_PATH").ok(),
            enable_request_logging: env::var("LOG_REQUESTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.enable_file && self.file_path.is_none() {
            anyhow::bail!("LOG_FILE_PATH must be set when LOG_FILE is true");
        }
        Ok(())
    }
}

impl HealthcareConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            hospital_name: env::var("HOSPITAL_NAME")
                .unwrap_or_else(|_| "Dubai Hospital".to_string()),
            hospital_id: env::var("HOSPITAL_ID")
                .unwrap_or_else(|_| "DHA-001".to_string()),
            dha_integration_enabled: env::var("DHA_INTEGRATION_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            dha_api_url: env::var("DHA_API_URL").ok(),
            dha_api_key: env::var("DHA_API_KEY").ok(),
            emergency_contact_required: env::var("EMERGENCY_CONTACT_REQUIRED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_patient_age: env::var("MAX_PATIENT_AGE")
                .unwrap_or_else(|_| "150".to_string())
                .parse()
                .context("Invalid MAX_PATIENT_AGE")?,
            default_session_timeout_minutes: env::var("SESSION_TIMEOUT_MINUTES")
                .unwrap_or_else(|_| "480".to_string())
                .parse()
                .context("Invalid SESSION_TIMEOUT_MINUTES")?,
            enable_triage_ai: env::var("ENABLE_TRIAGE_AI")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }

    fn validate(&self) -> Result<()> {
        if self.hospital_name.is_empty() {
            anyhow::bail!("Hospital name cannot be empty");
        }
        if self.hospital_id.is_empty() {
            anyhow::bail!("Hospital ID cannot be empty");
        }
        if self.dha_integration_enabled && self.dha_api_url.is_none() {
            anyhow::bail!("DHA_API_URL is required when DHA integration is enabled");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.environment, Environment::Development);
        assert!(config.is_development());
        assert!(!config.is_production());
    }

    #[test]
    fn test_environment_detection() {
        // Test production detection
        env::set_var("ENVIRONMENT", "production");
        assert_eq!(AppConfig::detect_environment(), Environment::Production);
        
        // Test staging detection
        env::set_var("ENVIRONMENT", "staging");
        assert_eq!(AppConfig::detect_environment(), Environment::Staging);
        
        // Test default
        env::remove_var("ENVIRONMENT");
        env::remove_var("APP_ENV");
        assert_eq!(AppConfig::detect_environment(), Environment::Development);
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();
        assert!(config.validate().is_ok());
        
        config.host = "".to_string();
        assert!(config.validate().is_err());
        
        config.host = "localhost".to_string();
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_jwt_config_validation() {
        let mut config = JwtConfig::default();
        
        // Short secret should fail
        config.secret = "short".to_string();
        assert!(config.validate().is_err());
        
        // Long enough secret should pass
        config.secret = "this-is-a-long-enough-secret-key-for-jwt".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_redis_config_validation() {
        let mut config = RedisConfig::default();
        assert!(config.validate().is_ok());
        
        config.url = "invalid-url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_healthcare_config_validation() {
        let mut config = HealthcareConfig::default();
        assert!(config.validate().is_ok());
        
        // Enable DHA integration without URL should fail
        config.dha_integration_enabled = true;
        config.dha_api_url = None;
        assert!(config.validate().is_err());
        
        // Add URL should pass
        config.dha_api_url = Some("https://api.dha.gov.ae".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_logging_config_validation() {
        let env = Environment::Development;
        let mut config = LoggingConfig::from_env(&env).unwrap();
        assert!(config.validate().is_ok());
        
        // Enable file logging without path should fail
        config.enable_file = true;
        config.file_path = None;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_redaction() {
        let mut config = AppConfig::default();
        config.jwt.secret = "super-secret-key-that-should-be-redacted".to_string();
        config.healthcare.dha_api_key = Some("secret-api-key".to_string());
        
        let json = config.to_json_redacted().unwrap();
        assert!(!json.contains("super-secret-key"));
        assert!(!json.contains("secret-api-key"));
        assert!(json.contains("[REDACTED]"));
    }
}
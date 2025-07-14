use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{AuthError, PatientError, HospitalError};

#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Patient management error: {0}")]
    Patient(#[from] PatientError),

    #[error("Hospital management error: {0}")]
    Hospital(#[from] HospitalError),

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Rate limit exceeded - try again in {retry_after} seconds")]
    RateLimit { retry_after: u64 },

    #[error("Internal server error - please try again later")]
    Internal,

    #[error("Service temporarily unavailable")]
    ServiceUnavailable,

    #[error("Request timeout")]
    Timeout,

    #[error("Invalid request format: {message}")]
    BadRequest { message: String },

    #[error("Resource conflict: {message}")]
    Conflict { message: String },

    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },

    #[error("System maintenance in progress")]
    Maintenance,
}

impl AppError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Auth(auth_error) => auth_error.status_code(),
            AppError::Patient(patient_error) => patient_error.status_code(),
            AppError::Hospital(hospital_error) => hospital_error.status_code(),
            AppError::Database { .. } => 500,
            AppError::Validation { .. } => 400,
            AppError::Configuration { .. } => 500,
            AppError::ExternalService { .. } => 502, // Bad Gateway
            AppError::RateLimit { .. } => 429,       // Too Many Requests
            AppError::Internal => 500,
            AppError::ServiceUnavailable => 503,
            AppError::Timeout => 504, // Gateway Timeout
            AppError::BadRequest { .. } => 400,
            AppError::Conflict { .. } => 409,
            AppError::NotImplemented { .. } => 501,
            AppError::Maintenance => 503,
        }
    }

    /// Get error code for client identification
    pub fn error_code(&self) -> String {
        match self {
            AppError::Auth(auth_error) => auth_error.error_code().to_string(),
            AppError::Patient(patient_error) => patient_error.error_code().to_string(),
            AppError::Hospital(hospital_error) => hospital_error.error_code().to_string(),
            AppError::Database { .. } => "DATABASE_ERROR".to_string(),
            AppError::Validation { .. } => "VALIDATION_ERROR".to_string(),
            AppError::Configuration { .. } => "CONFIGURATION_ERROR".to_string(),
            AppError::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR".to_string(),
            AppError::RateLimit { .. } => "RATE_LIMIT_EXCEEDED".to_string(),
            AppError::Internal => "INTERNAL_SERVER_ERROR".to_string(),
            AppError::ServiceUnavailable => "SERVICE_UNAVAILABLE".to_string(),
            AppError::Timeout => "REQUEST_TIMEOUT".to_string(),
            AppError::BadRequest { .. } => "BAD_REQUEST".to_string(),
            AppError::Conflict { .. } => "RESOURCE_CONFLICT".to_string(),
            AppError::NotImplemented { .. } => "NOT_IMPLEMENTED".to_string(),
            AppError::Maintenance => "SYSTEM_MAINTENANCE".to_string(),
        }
    }

    /// Check if this error should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::ServiceUnavailable
                | AppError::Timeout
                | AppError::ExternalService { .. }
                | AppError::Database { .. }
        )
    }

    /// Check if this error should be logged at error level
    pub fn should_log_error(&self) -> bool {
        matches!(
            self,
            AppError::Database { .. }
                | AppError::Configuration { .. }
                | AppError::Internal
                | AppError::ExternalService { .. }
        )
    }

    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Auth(auth_error) => auth_error.user_message(),
            AppError::Patient(patient_error) => patient_error.user_message(),
            AppError::Hospital(hospital_error) => hospital_error.user_message(),
            AppError::Validation { field, message } => {
                format!("Invalid {}: {}", field, message)
            }
            AppError::RateLimit { retry_after } => {
                format!("Too many requests. Please try again in {} seconds", retry_after)
            }
            AppError::ServiceUnavailable => {
                "Service is temporarily unavailable. Please try again later".to_string()
            }
            AppError::Timeout => "Request timed out. Please try again".to_string(),
            AppError::Maintenance => {
                "System is under maintenance. Please try again later".to_string()
            }
            AppError::Internal => {
                "An unexpected error occurred. Please contact support if the problem persists"
                    .to_string()
            }
            _ => self.to_string(),
        }
    }

    /// Create validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create database error
    pub fn database_error(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create external service error
    pub fn external_service_error(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }
}

/// API Error Response structure for JSON responses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub error_code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
}

impl ApiErrorResponse {
    /// Create from AppError
    pub fn from_app_error(error: &AppError) -> Self {
        Self {
            error: error.to_string(),
            error_code: error.error_code(),
            message: error.user_message(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add additional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_status_codes() {
        let auth_error = AppError::Auth(AuthError::InvalidCredentials);
        assert_eq!(auth_error.status_code(), 401);

        let validation_error = AppError::validation_error("username", "too short");
        assert_eq!(validation_error.status_code(), 400);

        assert_eq!(AppError::RateLimit { retry_after: 60 }.status_code(), 429);
    }

    #[test]
    fn test_error_code_generation() {
        let auth_error = AppError::Auth(AuthError::TokenExpired);
        assert_eq!(auth_error.error_code(), "AUTH_TOKEN_EXPIRED");

        let validation_error = AppError::validation_error("email", "invalid format");
        assert_eq!(validation_error.error_code(), "VALIDATION_ERROR");
    }

    #[test]
    fn test_retryable_errors() {
        assert!(AppError::ServiceUnavailable.is_retryable());
        assert!(AppError::Timeout.is_retryable());
        assert!(!AppError::Auth(AuthError::InvalidCredentials).is_retryable());
    }

    #[test]
    fn test_error_logging() {
        assert!(AppError::Internal.should_log_error());
        assert!(AppError::database_error("connection failed").should_log_error());
        assert!(!AppError::validation_error("name", "required").should_log_error());
    }

    #[test]
    fn test_user_messages() {
        let rate_limit_error = AppError::RateLimit { retry_after: 30 };
        assert!(rate_limit_error.user_message().contains("30 seconds"));

        let validation_error = AppError::validation_error("email", "invalid format");
        assert!(validation_error.user_message().contains("Invalid email"));
    }

    #[test]
    fn test_api_error_response() {
        let error = AppError::validation_error("username", "too short");
        let response = ApiErrorResponse::from_app_error(&error);

        assert_eq!(response.error_code, "VALIDATION_ERROR");
        assert!(response.message.contains("Invalid username"));
        assert!(!response.timestamp.is_empty());
    }

    #[test]
    fn test_error_conversion() {
        let auth_error = AuthError::InvalidCredentials;
        let app_error: AppError = auth_error.into();
        
        match app_error {
            AppError::Auth(AuthError::InvalidCredentials) => {},
            _ => panic!("Expected Auth error"),
        }
    }

    #[test]
    fn test_serialization() {
        let error = AppError::external_service_error("DHA Registry", "timeout");
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: AppError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }
}
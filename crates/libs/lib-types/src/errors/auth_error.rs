use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("User account is disabled: {username}")]
    AccountDisabled { username: String },

    #[error("User not found: {username}")]
    UserNotFound { username: String },

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Missing authentication token")]
    MissingToken,

    #[error("Insufficient permissions for this operation")]
    InsufficientPermissions,

    #[error("User is not assigned to hospital: {hospital_id}")]
    HospitalAccessDenied { hospital_id: Uuid },

    #[error("Password does not meet security requirements: {reason}")]
    WeakPassword { reason: String },

    #[error("Account is locked due to too many failed attempts")]
    AccountLocked,

    #[error("Session has been terminated")]
    SessionTerminated,

    #[error("Multi-factor authentication required")]
    MfaRequired,

    #[error("Invalid multi-factor authentication code")]
    InvalidMfaCode,

    #[error("Password reset required")]
    PasswordResetRequired,
}

impl AuthError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AuthError::InvalidCredentials => 401,
            AuthError::AccountDisabled { .. } => 403,
            AuthError::UserNotFound { .. } => 401, // Don't reveal user existence
            AuthError::InvalidToken => 401,
            AuthError::TokenExpired => 401,
            AuthError::MissingToken => 401,
            AuthError::InsufficientPermissions => 403,
            AuthError::HospitalAccessDenied { .. } => 403,
            AuthError::WeakPassword { .. } => 400,
            AuthError::AccountLocked => 423, // Locked
            AuthError::SessionTerminated => 401,
            AuthError::MfaRequired => 428, // Precondition Required
            AuthError::InvalidMfaCode => 400,
            AuthError::PasswordResetRequired => 428,
        }
    }

    /// Get error code for client identification
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthError::InvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            AuthError::AccountDisabled { .. } => "AUTH_ACCOUNT_DISABLED",
            AuthError::UserNotFound { .. } => "AUTH_INVALID_CREDENTIALS", // Same as invalid creds
            AuthError::InvalidToken => "AUTH_INVALID_TOKEN",
            AuthError::TokenExpired => "AUTH_TOKEN_EXPIRED",
            AuthError::MissingToken => "AUTH_MISSING_TOKEN",
            AuthError::InsufficientPermissions => "AUTH_INSUFFICIENT_PERMISSIONS",
            AuthError::HospitalAccessDenied { .. } => "AUTH_HOSPITAL_ACCESS_DENIED",
            AuthError::WeakPassword { .. } => "AUTH_WEAK_PASSWORD",
            AuthError::AccountLocked => "AUTH_ACCOUNT_LOCKED",
            AuthError::SessionTerminated => "AUTH_SESSION_TERMINATED",
            AuthError::MfaRequired => "AUTH_MFA_REQUIRED",
            AuthError::InvalidMfaCode => "AUTH_INVALID_MFA_CODE",
            AuthError::PasswordResetRequired => "AUTH_PASSWORD_RESET_REQUIRED",
        }
    }

    /// Check if this is a security-sensitive error that should be logged
    pub fn is_security_sensitive(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidCredentials
                | AuthError::AccountLocked
                | AuthError::InsufficientPermissions
                | AuthError::HospitalAccessDenied { .. }
                | AuthError::InvalidToken
        )
    }

    /// Get user-friendly message (safe for display)
    pub fn user_message(&self) -> String {
        match self {
            AuthError::InvalidCredentials | AuthError::UserNotFound { .. } => {
                "Invalid username or password".to_string()
            }
            AuthError::AccountDisabled { .. } => {
                "Your account has been disabled. Please contact your administrator".to_string()
            }
            AuthError::TokenExpired => "Your session has expired. Please log in again".to_string(),
            AuthError::AccountLocked => {
                "Account is temporarily locked due to multiple failed login attempts".to_string()
            }
            AuthError::WeakPassword { reason } => {
                format!("Password requirements not met: {}", reason)
            }
            AuthError::MfaRequired => {
                "Multi-factor authentication is required to continue".to_string()
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_status_codes() {
        assert_eq!(AuthError::InvalidCredentials.status_code(), 401);
        assert_eq!(AuthError::InsufficientPermissions.status_code(), 403);
        assert_eq!(AuthError::AccountLocked.status_code(), 423);
        assert_eq!(AuthError::MfaRequired.status_code(), 428);
    }

    #[test]
    fn test_auth_error_codes() {
        assert_eq!(
            AuthError::InvalidCredentials.error_code(),
            "AUTH_INVALID_CREDENTIALS"
        );
        assert_eq!(
            AuthError::TokenExpired.error_code(),
            "AUTH_TOKEN_EXPIRED"
        );
    }

    #[test]
    fn test_security_sensitivity() {
        assert!(AuthError::InvalidCredentials.is_security_sensitive());
        assert!(AuthError::AccountLocked.is_security_sensitive());
        assert!(!AuthError::WeakPassword { reason: "test".to_string() }.is_security_sensitive());
    }

    #[test]
    fn test_user_messages() {
        let error = AuthError::UserNotFound {
            username: "testuser".to_string(),
        };
        assert_eq!(error.user_message(), "Invalid username or password");

        let error = AuthError::WeakPassword {
            reason: "Must contain special characters".to_string(),
        };
        assert!(error.user_message().contains("special characters"));
    }

    #[test]
    fn test_serialization() {
        let error = AuthError::HospitalAccessDenied {
            hospital_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: AuthError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }
}
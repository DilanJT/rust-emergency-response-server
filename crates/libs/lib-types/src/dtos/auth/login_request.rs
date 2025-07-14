use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    /// Create new login request
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Validate login request
    pub fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username is required".to_string());
        }

        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }

        if self.password.is_empty() {
            return Err("Password is required".to_string());
        }

        if self.password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }

        Ok(())
    }

    /// Sanitize username (trim whitespace, lowercase)
    pub fn sanitized_username(&self) -> String {
        self.username.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_creation() {
        let request = LoginRequest::new("ahmed.director".to_string(), "password123".to_string());
        assert_eq!(request.username, "ahmed.director");
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_valid_login_request() {
        let request = LoginRequest::new("ahmed.director".to_string(), "password123".to_string());
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_username() {
        let request = LoginRequest::new("ab".to_string(), "password123".to_string());
        assert!(request.validate().is_err());
        assert!(request.validate().unwrap_err().contains("at least 3 characters"));
    }

    #[test]
    fn test_invalid_password() {
        let request = LoginRequest::new("ahmed.director".to_string(), "123".to_string());
        assert!(request.validate().is_err());
        assert!(request.validate().unwrap_err().contains("at least 6 characters"));
    }

    #[test]
    fn test_empty_fields() {
        let request = LoginRequest::new("".to_string(), "".to_string());
        let error = request.validate().unwrap_err();
        assert!(error.contains("Username is required"));
    }

    #[test]
    fn test_username_sanitization() {
        let request = LoginRequest::new("  Ahmed.Director  ".to_string(), "password".to_string());
        assert_eq!(request.sanitized_username(), "ahmed.director");
    }

    #[test]
    fn test_serialization() {
        let request = LoginRequest::new("ahmed".to_string(), "password".to_string());
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: LoginRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }
}
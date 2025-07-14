use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::UserRole;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64, // Seconds until expiration
    pub user_profile: UserProfileDto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfileDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub hospital_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl LoginResponse {
    /// Create new login response
    pub fn new(
        access_token: String,
        expires_in: i64,
        user_profile: UserProfileDto,
    ) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user_profile,
        }
    }

    /// Check if token is about to expire (within 5 minutes)
    pub fn is_near_expiry(&self) -> bool {
        self.expires_in < 300 // 5 minutes
    }
}

impl UserProfileDto {
    /// Create from domain User entity
    pub fn from_user(user: &crate::entities::User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role,
            hospital_id: user.hospital_id,
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            phone_number: user.phone_number.clone(),
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Get role display name
    pub fn role_display(&self) -> &'static str {
        self.role.display_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::User;

    fn create_test_user() -> User {
        User::new(
            "ahmed.director".to_string(),
            "ahmed@dubaihospital.ae".to_string(),
            "hashed_password".to_string(),
            UserRole::ErDirector,
            Uuid::new_v4(),
            "Ahmed".to_string(),
            "Al-Mansoori".to_string(),
            Some("+971501234567".to_string()),
        )
    }

    #[test]
    fn test_login_response_creation() {
        let user = create_test_user();
        let user_profile = UserProfileDto::from_user(&user);
        let response = LoginResponse::new(
            "jwt_token_here".to_string(),
            3600,
            user_profile,
        );

        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
        assert!(!response.is_near_expiry());
    }

    #[test]
    fn test_near_expiry() {
        let user = create_test_user();
        let user_profile = UserProfileDto::from_user(&user);
        let response = LoginResponse::new(
            "jwt_token_here".to_string(),
            200, // 200 seconds = ~3 minutes
            user_profile,
        );

        assert!(response.is_near_expiry());
    }

    #[test]
    fn test_user_profile_dto() {
        let user = create_test_user();
        let profile = UserProfileDto::from_user(&user);

        assert_eq!(profile.username, user.username);
        assert_eq!(profile.role, user.role);
        assert_eq!(profile.full_name(), "Ahmed Al-Mansoori");
        assert_eq!(profile.role_display(), "ER Director");
    }

    #[test]
    fn test_serialization() {
        let user = create_test_user();
        let user_profile = UserProfileDto::from_user(&user);
        let response = LoginResponse::new("token".to_string(), 3600, user_profile);
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: LoginResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
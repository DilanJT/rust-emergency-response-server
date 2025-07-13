use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::enums::UserRole;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub hospital_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user (for creation, before database insert)
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
        hospital_id: Uuid,
        first_name: String,
        last_name: String,
        phone_number: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            role,
            hospital_id,
            first_name,
            last_name,
            phone_number,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    pub fn can_access_patients(&self) -> bool {
        self.role.can_access_patients()
    }

    pub fn same_hospital(&self, hospital_id: Uuid) -> bool {
        self.hospital_id == hospital_id
    }

    pub fn role_display(&self) -> &'static str {
        self.role.display_name()
    }
}

// User without sensitive data (for API responses)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
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

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            hospital_id: user.hospital_id,
            first_name: user.first_name,
            last_name: user.last_name,
            phone_number: user.phone_number,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_user_creation() {
        let user = create_test_user();
        assert_eq!(user.username, "ahmed.director");
        assert_eq!(user.role, UserRole::ErDirector);
        assert!(user.is_active);
        assert_eq!(user.full_name(), "Ahmed Al-Mansoori");
    }

    #[test]
    fn test_admin_privileges() {
        let er_director = User::new(
            "director".to_string(),
            "director@hospital.ae".to_string(),
            "hash".to_string(),
            UserRole::ErDirector,
            Uuid::new_v4(),
            "Dr".to_string(),
            "Director".to_string(),
            None,
        );
        
        let paramedic = User::new(
            "paramedic".to_string(),
            "paramedic@hospital.ae".to_string(),
            "hash".to_string(),
            UserRole::Paramedic,
            Uuid::new_v4(),
            "John".to_string(),
            "Paramedic".to_string(),
            None,
        );

        assert!(er_director.is_admin());
        assert!(!paramedic.is_admin());
    }

    #[test]
    fn test_patient_access() {
        let admin = User::new(
            "admin".to_string(),
            "admin@system.ae".to_string(),
            "hash".to_string(),
            UserRole::Admin,
            Uuid::new_v4(),
            "System".to_string(),
            "Admin".to_string(),
            None,
        );

        let nurse = User::new(
            "nurse".to_string(),
            "nurse@hospital.ae".to_string(),
            "hash".to_string(),
            UserRole::Nurse,
            Uuid::new_v4(),
            "Sarah".to_string(),
            "Nurse".to_string(),
            None,
        );

        assert!(!admin.can_access_patients()); // System admin doesn't need patient access
        assert!(nurse.can_access_patients());
    }

    #[test]
    fn test_hospital_affiliation() {
        let hospital_id = Uuid::new_v4();
        let other_hospital_id = Uuid::new_v4();
        
        let user = User::new(
            "test".to_string(),
            "test@hospital.ae".to_string(),
            "hash".to_string(),
            UserRole::Nurse,
            hospital_id,
            "Test".to_string(),
            "User".to_string(),
            None,
        );

        assert!(user.same_hospital(hospital_id));
        assert!(!user.same_hospital(other_hospital_id));
    }

    #[test]
    fn test_user_profile_conversion() {
        let user = create_test_user();
        let profile: UserProfile = user.clone().into();
        
        assert_eq!(profile.id, user.id);
        assert_eq!(profile.username, user.username);
        assert_eq!(profile.role, user.role);
        // password_hash should not be in profile
    }

    #[test]
    fn test_serialization() {
        let user = create_test_user();
        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();
        assert_eq!(user, deserialized);
    }
}
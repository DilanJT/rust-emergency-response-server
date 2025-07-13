use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    ErDirector,
    Paramedic,
    Nurse,
    Specialist,
    Admin
}

impl UserRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            UserRole::ErDirector => "ER Director",
            UserRole::Paramedic => "Paramedic",
            UserRole::Nurse => "Nurse",
            UserRole::Specialist => "Specialist",
            UserRole::Admin => "Admin",
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::ErDirector | UserRole::Admin)
    }

    pub fn can_access_patients(&self) -> bool {
        matches!(self, UserRole::ErDirector | UserRole::Paramedic | UserRole::Nurse | UserRole::Specialist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let role = UserRole::ErDirector;
        println!("UserRole: {:?}", role);
        let json = serde_json::to_string(&role).unwrap();
        println!("Serialized UserRole: {}", json);
        assert_eq!(json, "\"er_director\"");

        let deserialized: UserRole = serde_json::from_str(&json).unwrap();
        println!("left side: {:?}, right side: {:?}", deserialized, role);
        assert_eq!(deserialized, role);
    }
}
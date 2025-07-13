use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[sqlx(type_name = "triage_level", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TriageLevel {
    Critical = 1,
    High = 2,
    Medium = 3,
    Low = 4,
}

impl TriageLevel {
    pub fn display_name(&self) -> &'static str {
        match self {
            TriageLevel::Critical => "Critical",
            TriageLevel::High => "High",
            TriageLevel::Medium => "Medium",
            TriageLevel::Low => "Low",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            TriageLevel::Critical => "#e74c3c", // Red
            TriageLevel::High => "#f39c12",     // Orange
            TriageLevel::Medium => "#f1c40f",   // Yellow
            TriageLevel::Low => "#2ecc71",      // Green
        }
    }

    pub fn priority(&self) -> u8 {
        *self as u8
    }

    pub fn is_emergency(&self) -> bool {
        matches!(self, TriageLevel::Critical | TriageLevel::High)
    }

    pub fn all_in_priority_order() -> Vec<TriageLevel> {
        vec![
            TriageLevel::Critical,
            TriageLevel::High,
            TriageLevel::Medium,
            TriageLevel::Low,
        ]
    }
}

impl std::fmt::Display for TriageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let level = TriageLevel::Critical;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"critical\"");
        
        let deserialized: TriageLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, level);
    }
}
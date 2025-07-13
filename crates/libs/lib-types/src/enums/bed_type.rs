use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "bed_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BedType {
    General,
    Icu,
    Emergency,
    Isolation,
    Pediatric,
}

impl BedType {
    /// Get display name for bed type
    pub fn display_name(&self) -> &'static str {
        match self {
            BedType::General => "General",
            BedType::Icu => "ICU",
            BedType::Emergency => "Emergency",
            BedType::Isolation => "Isolation",
            BedType::Pediatric => "Pediatric",
        }
    }

    /// Check if bed type is suitable for triage level
    pub fn is_suitable_for_triage(&self, triage_level: crate::triage_level::TriageLevel) -> bool {
        use crate::triage_level::TriageLevel;
        
        match (self, triage_level) {
            (BedType::Icu, TriageLevel::Critical) => true,
            (BedType::Emergency, TriageLevel::Critical | TriageLevel::High) => true,
            (BedType::General, TriageLevel::Medium | TriageLevel::Low) => true,
            (BedType::Isolation, _) => true, // Isolation beds can take any patient if needed
            (BedType::Pediatric, _) => false, // Pediatric beds need age check, not just triage
            _ => false,
        }
    }

    /// Get priority for bed assignment (lower number = higher priority)
    pub fn assignment_priority(&self) -> u8 {
        match self {
            BedType::Icu => 1,
            BedType::Emergency => 2,
            BedType::Isolation => 3,
            BedType::Pediatric => 4,
            BedType::General => 5,
        }
    }

    /// Check if bed type requires special equipment
    pub fn requires_special_equipment(&self) -> bool {
        matches!(self, BedType::Icu | BedType::Isolation)
    }

    /// Get all bed types ordered by priority
    pub fn all_by_priority() -> Vec<BedType> {
        vec![
            BedType::Icu,
            BedType::Emergency,
            BedType::Isolation,
            BedType::Pediatric,
            BedType::General,
        ]
    }
}

impl std::fmt::Display for BedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triage_level::TriageLevel;

    #[test]
    fn test_triage_suitability() {
        assert!(BedType::Icu.is_suitable_for_triage(TriageLevel::Critical));
        assert!(BedType::Emergency.is_suitable_for_triage(TriageLevel::High));
        assert!(BedType::General.is_suitable_for_triage(TriageLevel::Low));
        assert!(!BedType::General.is_suitable_for_triage(TriageLevel::Critical));
    }

    #[test]
    fn test_special_equipment() {
        assert!(BedType::Icu.requires_special_equipment());
        assert!(BedType::Isolation.requires_special_equipment());
        assert!(!BedType::General.requires_special_equipment());
        assert!(!BedType::Emergency.requires_special_equipment());
    }

    #[test]
    fn test_assignment_priority() {
        assert!(BedType::Icu.assignment_priority() < BedType::Emergency.assignment_priority());
        assert!(BedType::Emergency.assignment_priority() < BedType::General.assignment_priority());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", BedType::Icu), "ICU");
        assert_eq!(format!("{}", BedType::Pediatric), "Pediatric");
    }
}
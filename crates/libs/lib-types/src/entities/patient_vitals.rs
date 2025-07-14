use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::enums::TriageLevel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct PatientVitals {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub recorded_by: Uuid, // User ID who recorded the vitals
    pub systolic_bp: Option<i32>,
    pub diastolic_bp: Option<i32>,
    pub heart_rate: Option<i32>,
    pub oxygen_saturation: Option<i32>,
    pub temperature: Option<f32>, // Celsius
    pub respiratory_rate: Option<i32>,
    pub weight: Option<f32>, // Kilograms
    pub device_id: Option<String>,
    pub additional_measurements: serde_json::Value, // JSON for other measurements
    pub notes: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl PatientVitals {
    /// Create new vital signs record
    pub fn new(
        patient_id: Uuid,
        recorded_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_id,
            recorded_by,
            systolic_bp: None,
            diastolic_bp: None,
            heart_rate: None,
            oxygen_saturation: None,
            temperature: None,
            respiratory_rate: None,
            weight: None,
            device_id: None,
            additional_measurements: serde_json::Value::Object(serde_json::Map::new()),
            notes: None,
            recorded_at: now,
            created_at: now,
        }
    }

    /// Set blood pressure
    pub fn set_blood_pressure(&mut self, systolic: i32, diastolic: i32) {
        self.systolic_bp = Some(systolic);
        self.diastolic_bp = Some(diastolic);
    }

    /// Get blood pressure as tuple
    pub fn blood_pressure(&self) -> Option<(i32, i32)> {
        match (self.systolic_bp, self.diastolic_bp) {
            (Some(sys), Some(dia)) => Some((sys, dia)),
            _ => None,
        }
    }

    /// Assess blood pressure status
    pub fn bp_assessment(&self) -> VitalStatus {
        match self.blood_pressure() {
            Some((sys, dia)) => {
                if sys >= 180 || dia >= 120 {
                    VitalStatus::Critical
                } else if sys >= 140 || dia >= 90 {
                    VitalStatus::High
                } else if sys < 90 || dia < 60 {
                    VitalStatus::Low
                } else {
                    VitalStatus::Normal
                }
            }
            None => VitalStatus::Unknown,
        }
    }

    /// Assess heart rate status
    pub fn hr_assessment(&self) -> VitalStatus {
        match self.heart_rate {
            Some(hr) => {
                if hr < 50 || hr > 120 {
                    VitalStatus::Critical
                } else if hr < 60 || hr > 100 {
                    VitalStatus::High
                } else {
                    VitalStatus::Normal
                }
            }
            None => VitalStatus::Unknown,
        }
    }

    /// Assess oxygen saturation status
    pub fn o2_assessment(&self) -> VitalStatus {
        match self.oxygen_saturation {
            Some(o2) => {
                if o2 < 90 {
                    VitalStatus::Critical
                } else if o2 < 95 {
                    VitalStatus::High
                } else {
                    VitalStatus::Normal
                }
            }
            None => VitalStatus::Unknown,
        }
    }

    /// Assess temperature status
    pub fn temp_assessment(&self) -> VitalStatus {
        match self.temperature {
            Some(temp) => {
                if temp < 35.0 || temp > 40.0 {
                    VitalStatus::Critical
                } else if temp < 36.0 || temp > 38.5 {
                    VitalStatus::High
                } else {
                    VitalStatus::Normal
                }
            }
            None => VitalStatus::Unknown,
        }
    }

    /// Get overall vital status (worst of all vitals)
    pub fn overall_assessment(&self) -> VitalStatus {
        let assessments = [
            self.bp_assessment(),
            self.hr_assessment(),
            self.o2_assessment(),
            self.temp_assessment(),
        ];

        if assessments.iter().any(|&s| s == VitalStatus::Critical) {
            VitalStatus::Critical
        } else if assessments.iter().any(|&s| s == VitalStatus::High) {
            VitalStatus::High
        } else if assessments.iter().any(|&s| s == VitalStatus::Low) {
            VitalStatus::Low
        } else if assessments.iter().all(|&s| s == VitalStatus::Normal) {
            VitalStatus::Normal
        } else {
            VitalStatus::Unknown
        }
    }

    /// Suggest triage level based on vitals
    pub fn suggested_triage(&self) -> Option<TriageLevel> {
        match self.overall_assessment() {
            VitalStatus::Critical => Some(TriageLevel::Critical),
            VitalStatus::High => Some(TriageLevel::High),
            VitalStatus::Low => Some(TriageLevel::Medium),
            VitalStatus::Normal => Some(TriageLevel::Low),
            VitalStatus::Unknown => None,
        }
    }

    /// Check if vitals indicate emergency
    pub fn is_emergency(&self) -> bool {
        matches!(
            self.overall_assessment(),
            VitalStatus::Critical | VitalStatus::High
        )
    }

    /// Get formatted blood pressure string
    pub fn bp_string(&self) -> String {
        match self.blood_pressure() {
            Some((sys, dia)) => format!("{}/{}", sys, dia),
            None => "N/A".to_string(),
        }
    }

    /// Get formatted temperature string
    pub fn temp_string(&self) -> String {
        match self.temperature {
            Some(temp) => format!("{:.1}°C", temp),
            None => "N/A".to_string(),
        }
    }

    /// Check if vitals are complete (all major vitals recorded)
    pub fn is_complete(&self) -> bool {
        self.systolic_bp.is_some()
            && self.diastolic_bp.is_some()
            && self.heart_rate.is_some()
            && self.oxygen_saturation.is_some()
            && self.temperature.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VitalStatus {
    Critical,
    High,
    Low,
    Normal,
    Unknown,
}

impl VitalStatus {
    pub fn color(&self) -> &'static str {
        match self {
            VitalStatus::Critical => "#e74c3c", // Red
            VitalStatus::High => "#f39c12",     // Orange
            VitalStatus::Low => "#f1c40f",      // Yellow
            VitalStatus::Normal => "#2ecc71",   // Green
            VitalStatus::Unknown => "#95a5a6",  // Gray
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vitals() -> PatientVitals {
        let mut vitals = PatientVitals::new(Uuid::new_v4(), Uuid::new_v4());
        vitals.set_blood_pressure(120, 80);
        vitals.heart_rate = Some(75);
        vitals.oxygen_saturation = Some(98);
        vitals.temperature = Some(37.0);
        vitals.respiratory_rate = Some(16);
        vitals
    }

    #[test]
    fn test_vitals_creation() {
        let vitals = create_test_vitals();
        assert_eq!(vitals.blood_pressure(), Some((120, 80)));
        assert_eq!(vitals.heart_rate, Some(75));
        assert!(vitals.is_complete());
    }

    #[test]
    fn test_blood_pressure_assessment() {
        let mut vitals = create_test_vitals();
        
        // Normal BP
        vitals.set_blood_pressure(120, 80);
        assert_eq!(vitals.bp_assessment(), VitalStatus::Normal);
        
        // High BP
        vitals.set_blood_pressure(150, 95);
        assert_eq!(vitals.bp_assessment(), VitalStatus::High);
        
        // Critical BP
        vitals.set_blood_pressure(190, 125);
        assert_eq!(vitals.bp_assessment(), VitalStatus::Critical);
        
        // Low BP
        vitals.set_blood_pressure(85, 55);
        assert_eq!(vitals.bp_assessment(), VitalStatus::Low);
    }

    #[test]
    fn test_heart_rate_assessment() {
        let mut vitals = create_test_vitals();
        
        // Normal HR
        vitals.heart_rate = Some(75);
        assert_eq!(vitals.hr_assessment(), VitalStatus::Normal);
        
        // High HR
        vitals.heart_rate = Some(110);
        assert_eq!(vitals.hr_assessment(), VitalStatus::High);
        
        // Critical HR
        vitals.heart_rate = Some(45);
        assert_eq!(vitals.hr_assessment(), VitalStatus::Critical);
    }

    #[test]
    fn test_oxygen_assessment() {
        let mut vitals = create_test_vitals();
        
        // Normal O2
        vitals.oxygen_saturation = Some(98);
        assert_eq!(vitals.o2_assessment(), VitalStatus::Normal);
        
        // High concern O2
        vitals.oxygen_saturation = Some(92);
        assert_eq!(vitals.o2_assessment(), VitalStatus::High);
        
        // Critical O2
        vitals.oxygen_saturation = Some(85);
        assert_eq!(vitals.o2_assessment(), VitalStatus::Critical);
    }

    #[test]
    fn test_temperature_assessment() {
        let mut vitals = create_test_vitals();
        
        // Normal temp
        vitals.temperature = Some(37.0);
        assert_eq!(vitals.temp_assessment(), VitalStatus::Normal);
        
        // High temp (fever)
        vitals.temperature = Some(39.5);
        assert_eq!(vitals.temp_assessment(), VitalStatus::High);
        
        // Critical temp
        vitals.temperature = Some(41.0);
        assert_eq!(vitals.temp_assessment(), VitalStatus::Critical);
    }

    #[test]
    fn test_overall_assessment() {
        let mut vitals = create_test_vitals();
        
        // All normal
        assert_eq!(vitals.overall_assessment(), VitalStatus::Normal);
        
        // One critical makes overall critical
        vitals.heart_rate = Some(40); // Critical
        assert_eq!(vitals.overall_assessment(), VitalStatus::Critical);
        
        // One high makes overall high (if no critical)
        vitals.heart_rate = Some(75); // Back to normal
        vitals.temperature = Some(39.0); // High
        assert_eq!(vitals.overall_assessment(), VitalStatus::High);
    }

    #[test]
    fn test_triage_suggestion() {
        let mut vitals = create_test_vitals();
        
        // Normal vitals suggest low triage
        assert_eq!(vitals.suggested_triage(), Some(TriageLevel::Low));
        
        // Critical vitals suggest critical triage
        vitals.oxygen_saturation = Some(85);
        assert_eq!(vitals.suggested_triage(), Some(TriageLevel::Critical));
        assert!(vitals.is_emergency());
    }

    #[test]
    fn test_formatting() {
        let vitals = create_test_vitals();
        assert_eq!(vitals.bp_string(), "120/80");
        assert_eq!(vitals.temp_string(), "37.0°C");
    }

    #[test]
    fn test_completeness() {
        let mut vitals = PatientVitals::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(!vitals.is_complete());
        
        vitals.set_blood_pressure(120, 80);
        vitals.heart_rate = Some(75);
        vitals.oxygen_saturation = Some(98);
        vitals.temperature = Some(37.0);
        assert!(vitals.is_complete());
    }

    #[test]
    fn test_serialization() {
        let vitals = create_test_vitals();
        let json = serde_json::to_string(&vitals).unwrap();
        let deserialized: PatientVitals = serde_json::from_str(&json).unwrap();
        assert_eq!(vitals, deserialized);
    }
}
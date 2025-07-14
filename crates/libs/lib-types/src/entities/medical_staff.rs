use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::enums::AvailabilityStatus;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct MedicalStaff {
    pub id: Uuid,
    pub user_id: Uuid, // Foreign key to User table
    pub hospital_id: Uuid,
    pub staff_id: String, // Hospital-specific staff ID
    pub specialty: String,
    pub availability_status: AvailabilityStatus,
    pub license_number: String,
    pub certifications: serde_json::Value, // JSON array of certifications
    pub shift_schedule: serde_json::Value, // JSON object with shift information
    pub department: String,
    pub seniority_level: String, // "Junior", "Senior", "Consultant", "Director"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MedicalStaff {
    /// Create new medical staff record
    pub fn new(
        user_id: Uuid,
        hospital_id: Uuid,
        staff_id: String,
        specialty: String,
        license_number: String,
        department: String,
        seniority_level: String,
        certifications: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            hospital_id,
            staff_id,
            specialty,
            availability_status: AvailabilityStatus::Available,
            license_number,
            certifications: serde_json::to_value(certifications).unwrap_or(serde_json::Value::Array(vec![])),
            shift_schedule: serde_json::Value::Object(serde_json::Map::new()),
            department,
            seniority_level,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if staff can take new assignments
    pub fn can_take_assignment(&self) -> bool {
        self.availability_status.can_take_assignment()
    }

    /// Check if staff is currently working
    pub fn is_working(&self) -> bool {
        self.availability_status.is_working()
    }

    /// Update availability status
    pub fn update_availability(&mut self, status: AvailabilityStatus) {
        self.availability_status = status;
        self.updated_at = Utc::now();
    }

    /// Get assignment priority (lower is better)
    pub fn assignment_priority(&self) -> u8 {
        let availability_priority = self.availability_status.assignment_priority();
        let seniority_bonus = match self.seniority_level.as_str() {
            "Director" => 0,
            "Consultant" => 1,
            "Senior" => 2,
            "Junior" => 3,
            _ => 4,
        };
         (availability_priority * 10) + seniority_bonus
    }

    /// Check if staff has specific specialty
    pub fn has_specialty(&self, specialty: &str) -> bool {
        self.specialty.eq_ignore_ascii_case(specialty)
    }

    /// Get certifications as vector
    pub fn get_certifications(&self) -> Vec<String> {
        self.certifications
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if staff has specific certification
    pub fn has_certification(&self, certification: &str) -> bool {
        self.get_certifications()
            .iter()
            .any(|c| c.eq_ignore_ascii_case(certification))
    }

    /// Add certification
    pub fn add_certification(&mut self, certification: String) {
        if let serde_json::Value::Array(ref mut certs) = self.certifications {
            if !certs.iter().any(|c| c.as_str() == Some(&certification)) {
                certs.push(serde_json::Value::String(certification));
                self.updated_at = Utc::now();
            }
        }
    }

    /// Check if staff is senior level or above
    pub fn is_senior(&self) -> bool {
        matches!(
            self.seniority_level.as_str(),
            "Senior" | "Consultant" | "Director"
        )
    }

    /// Check if staff can supervise others
    pub fn can_supervise(&self) -> bool {
        matches!(
            self.seniority_level.as_str(),
            "Consultant" | "Director"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_staff() -> MedicalStaff {
        MedicalStaff::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "STAFF-001".to_string(),
            "Emergency Medicine".to_string(),
            "LIC-EM-12345".to_string(),
            "Emergency Department".to_string(),
            "Senior".to_string(),
            vec!["ACLS".to_string(), "PALS".to_string()],
        )
    }

    #[test]
    fn test_staff_creation() {
        let staff = create_test_staff();
        assert_eq!(staff.staff_id, "STAFF-001");
        assert_eq!(staff.specialty, "Emergency Medicine");
        assert_eq!(staff.availability_status, AvailabilityStatus::Available);
        assert!(staff.can_take_assignment());
    }

    #[test]
    fn test_availability_management() {
        let mut staff = create_test_staff();
        
        // Initially available
        assert!(staff.can_take_assignment());
        assert!(staff.is_working());
        
        // Set to busy
        staff.update_availability(AvailabilityStatus::Busy);
        assert!(!staff.can_take_assignment());
        assert!(staff.is_working());
        
        // Set to off duty
        staff.update_availability(AvailabilityStatus::OffDuty);
        assert!(!staff.can_take_assignment());
        assert!(!staff.is_working());
        
        // Set to on call
        staff.update_availability(AvailabilityStatus::OnCall);
        assert!(staff.can_take_assignment());
        assert!(staff.is_working());
    }

    #[test]
    fn test_assignment_priority() {
        let mut director = create_test_staff();
        director.seniority_level = "Director".to_string();
        director.availability_status = AvailabilityStatus::Available;
        
        let mut junior = create_test_staff();
        junior.seniority_level = "Junior".to_string();
        junior.availability_status = AvailabilityStatus::Available;
        
        assert!(director.assignment_priority() < junior.assignment_priority());
        
        // Busy director vs available junior
        director.availability_status = AvailabilityStatus::Busy;
        assert!(director.assignment_priority() > junior.assignment_priority());
    }

    #[test]
    fn test_specialty_matching() {
        let staff = create_test_staff();
        
        assert!(staff.has_specialty("Emergency Medicine"));
        assert!(staff.has_specialty("emergency medicine")); // Case insensitive
        assert!(!staff.has_specialty("Cardiology"));
    }

    #[test]
    fn test_certifications() {
        let mut staff = create_test_staff();
        let certs = staff.get_certifications();
        
        assert!(certs.contains(&"ACLS".to_string()));
        assert!(certs.contains(&"PALS".to_string()));
        assert!(staff.has_certification("ACLS"));
        assert!(staff.has_certification("acls")); // Case insensitive
        
        // Add new certification
        staff.add_certification("BLS".to_string());
        assert!(staff.has_certification("BLS"));
        
        // Don't add duplicate
        let cert_count = staff.get_certifications().len();
        staff.add_certification("BLS".to_string());
        assert_eq!(staff.get_certifications().len(), cert_count);
    }

    #[test]
    fn test_seniority_levels() {
        let mut staff = create_test_staff();
        
        // Senior
        staff.seniority_level = "Senior".to_string();
        assert!(staff.is_senior());
        assert!(!staff.can_supervise());
        
        // Consultant
        staff.seniority_level = "Consultant".to_string();
        assert!(staff.is_senior());
        assert!(staff.can_supervise());
        
        // Director
        staff.seniority_level = "Director".to_string();
        assert!(staff.is_senior());
        assert!(staff.can_supervise());
        
        // Junior
        staff.seniority_level = "Junior".to_string();
        assert!(!staff.is_senior());
        assert!(!staff.can_supervise());
    }

    #[test]
    fn test_serialization() {
        let staff = create_test_staff();
        let json = serde_json::to_string(&staff).unwrap();
        let deserialized: MedicalStaff = serde_json::from_str(&json).unwrap();
        assert_eq!(staff, deserialized);
    }
}
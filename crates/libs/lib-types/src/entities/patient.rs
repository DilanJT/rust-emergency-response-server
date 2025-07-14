use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::enums::{PatientStatus, TriageLevel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Patient {
    pub id: Uuid,
    pub patient_number: String,
    pub national_id: Option<String>, // Emirates ID or other national ID
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub gender: String, // "Male", "Female", "Other"
    pub chief_complaint: String,
    pub triage_level: TriageLevel,
    pub status: PatientStatus,
    pub hospital_id: Uuid,
    pub assigned_staff_id: Option<Uuid>,
    pub ambulance_id: Option<Uuid>,
    pub bed_id: Option<Uuid>,
    pub emergency_contacts: serde_json::Value, // JSON object with contact info
    pub medical_history: serde_json::Value,    // JSON object with medical history
    pub allergies: serde_json::Value,          // JSON array of allergies
    pub insurance_info: serde_json::Value,     // JSON object with insurance details
    pub incident_location: Option<String>,     // Location where incident occurred
    pub incident_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Patient {
    /// Create a new patient
    pub fn new(
        patient_number: String,
        national_id: Option<String>,
        first_name: String,
        last_name: String,
        age: i32,
        gender: String,
        chief_complaint: String,
        triage_level: TriageLevel,
        hospital_id: Uuid,
        incident_location: Option<String>,
        incident_time: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            patient_number,
            national_id,
            first_name,
            last_name,
            age,
            gender,
            chief_complaint,
            triage_level,
            status: PatientStatus::Dispatched,
            hospital_id,
            assigned_staff_id: None,
            ambulance_id: None,
            bed_id: None,
            emergency_contacts: serde_json::Value::Object(serde_json::Map::new()),
            medical_history: serde_json::Value::Object(serde_json::Map::new()),
            allergies: serde_json::Value::Array(vec![]),
            insurance_info: serde_json::Value::Object(serde_json::Map::new()),
            incident_location,
            incident_time,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if patient is currently active (not discharged)
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Check if patient is in transport
    pub fn is_in_transport(&self) -> bool {
        self.status.is_in_transport()
    }

    /// Check if patient has arrived at hospital
    pub fn is_at_hospital(&self) -> bool {
        self.status.is_at_hospital()
    }

    /// Check if patient is emergency level
    pub fn is_emergency(&self) -> bool {
        self.triage_level.is_emergency()
    }

    /// Get patient priority for sorting
    pub fn priority(&self) -> u8 {
        self.triage_level.priority()
    }

    /// Update patient status
    pub fn update_status(&mut self, new_status: PatientStatus) {
        let next_statuses = self.status.next_statuses();
        if next_statuses.contains(&new_status) || next_statuses.is_empty() {
            self.status = new_status;
            self.updated_at = Utc::now();
        }
    }

    /// Assign to medical staff
    pub fn assign_staff(&mut self, staff_id: Uuid) {
        self.assigned_staff_id = Some(staff_id);
        self.updated_at = Utc::now();
    }

    /// Assign to ambulance
    pub fn assign_ambulance(&mut self, ambulance_id: Uuid) {
        self.ambulance_id = Some(ambulance_id);
        self.updated_at = Utc::now();
    }

    /// Assign to bed
    pub fn assign_bed(&mut self, bed_id: Uuid) {
        self.bed_id = Some(bed_id);
        self.updated_at = Utc::now();
    }

    /// Check if patient is anonymous (no national ID)
    pub fn is_anonymous(&self) -> bool {
        self.national_id.is_none() || self.national_id.as_ref().unwrap().is_empty()
    }

    /// Get allergies as vector
    pub fn get_allergies(&self) -> Vec<String> {
        self.allergies
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add allergy
    pub fn add_allergy(&mut self, allergy: String) {
        if let serde_json::Value::Array(ref mut allergies) = self.allergies {
            if !allergies.iter().any(|a| a.as_str() == Some(&allergy)) {
                allergies.push(serde_json::Value::String(allergy));
                self.updated_at = Utc::now();
            }
        }
    }

    /// Get display name for UI (handles anonymous patients)
    pub fn display_name(&self) -> String {
        if self.is_anonymous() {
            format!("Anonymous Patient ({})", self.patient_number)
        } else {
            self.full_name()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_patient() -> Patient {
        Patient::new(
            "PAT-001".to_string(),
            Some("784-1990-1234567-1".to_string()),
            "Ahmed".to_string(),
            "Al-Rashid".to_string(),
            45,
            "Male".to_string(),
            "Chest Pain".to_string(),
            TriageLevel::Critical,
            Uuid::new_v4(),
            Some("Sheikh Zayed Road, Dubai".to_string()),
            Some(Utc::now()),
        )
    }

    #[test]
    fn test_patient_creation() {
        let patient = create_test_patient();
        assert_eq!(patient.patient_number, "PAT-001");
        assert_eq!(patient.full_name(), "Ahmed Al-Rashid");
        assert_eq!(patient.status, PatientStatus::Dispatched);
        assert!(patient.is_active());
        assert!(patient.is_emergency());
    }

    #[test]
    fn test_anonymous_patient() {
        let mut patient = create_test_patient();
        patient.national_id = None;
        
        assert!(patient.is_anonymous());
        assert_eq!(patient.display_name(), "Anonymous Patient (PAT-001)");
    }

    #[test]
    fn test_status_updates() {
        let mut patient = create_test_patient();
        
        // Valid status progression
        patient.update_status(PatientStatus::EnRoute);
        assert_eq!(patient.status, PatientStatus::EnRoute);
        
        // Invalid status jump should not work
        let original_status = patient.status;
        patient.update_status(PatientStatus::Discharged); // Can't go directly from EnRoute to Discharged
        assert_eq!(patient.status, original_status);
    }

    #[test]
    fn test_assignments() {
        let mut patient = create_test_patient();
        let staff_id = Uuid::new_v4();
        let ambulance_id = Uuid::new_v4();
        let bed_id = Uuid::new_v4();
        
        patient.assign_staff(staff_id);
        assert_eq!(patient.assigned_staff_id, Some(staff_id));
        
        patient.assign_ambulance(ambulance_id);
        assert_eq!(patient.ambulance_id, Some(ambulance_id));
        
        patient.assign_bed(bed_id);
        assert_eq!(patient.bed_id, Some(bed_id));
    }

    #[test]
    fn test_allergies() {
        let mut patient = create_test_patient();
        
        patient.add_allergy("Penicillin".to_string());
        patient.add_allergy("Nuts".to_string());
        patient.add_allergy("Penicillin".to_string()); // Duplicate should be ignored
        
        let allergies = patient.get_allergies();
        assert_eq!(allergies.len(), 2);
        assert!(allergies.contains(&"Penicillin".to_string()));
        assert!(allergies.contains(&"Nuts".to_string()));
    }

    #[test]
    fn test_patient_states() {
        let mut patient = create_test_patient();
        
        // Initially dispatched
        assert!(patient.is_in_transport());
        assert!(!patient.is_at_hospital());
        
        // En route
        patient.update_status(PatientStatus::EnRoute);
        assert!(patient.is_in_transport());
        assert!(!patient.is_at_hospital());
        
        // Arrived
        patient.update_status(PatientStatus::Arrived);
        assert!(!patient.is_in_transport());
        assert!(patient.is_at_hospital());
        
        // Admitted
        patient.update_status(PatientStatus::Admitted);
        assert!(patient.is_at_hospital());
        assert!(patient.is_active());
        
        // Discharged
        patient.update_status(PatientStatus::Discharged);
        assert!(!patient.is_active());
    }

    #[test]
    fn test_priority_ordering() {
        let critical = Patient::new(
            "PAT-001".to_string(), None, "Test".to_string(), "Critical".to_string(),
            30, "Male".to_string(), "Critical".to_string(), TriageLevel::Critical,
            Uuid::new_v4(), None, None
        );
        
        let low = Patient::new(
            "PAT-002".to_string(), None, "Test".to_string(), "Low".to_string(),
            30, "Male".to_string(), "Low".to_string(), TriageLevel::Low,
            Uuid::new_v4(), None, None
        );
        
        assert!(critical.priority() < low.priority());
    }

    #[test]
    fn test_serialization() {
        let patient = create_test_patient();
        let json = serde_json::to_string(&patient).unwrap();
        let deserialized: Patient = serde_json::from_str(&json).unwrap();
        assert_eq!(patient, deserialized);
    }
}
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{PatientStatus, TriageLevel};
use crate::entities::{Patient, PatientVitals};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatientResponse {
    pub id: Uuid,
    pub patient_number: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub gender: String,
    pub chief_complaint: String,
    pub triage_level: TriageLevel,
    pub status: PatientStatus,
    pub hospital_id: Uuid,
    pub hospital_name: Option<String>,
    pub assigned_staff_id: Option<Uuid>,
    pub assigned_staff_name: Option<String>,
    pub ambulance_id: Option<String>,
    pub bed_id: Option<Uuid>,
    pub bed_number: Option<String>,
    pub incident_location: Option<String>,
    pub incident_time: Option<DateTime<Utc>>,
    pub latest_vitals: Option<VitalsDto>,
    pub allergies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VitalsDto {
    pub id: Uuid,
    pub systolic_bp: Option<i32>,
    pub diastolic_bp: Option<i32>,
    pub heart_rate: Option<i32>,
    pub oxygen_saturation: Option<i32>,
    pub temperature: Option<f32>,
    pub respiratory_rate: Option<i32>,
    pub recorded_by: Uuid,
    pub recorded_by_name: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatientListResponse {
    pub patients: Vec<PatientSummary>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatientSummary {
    pub id: Uuid,
    pub patient_number: String,
    pub display_name: String,
    pub age: i32,
    pub gender: String,
    pub chief_complaint: String,
    pub triage_level: TriageLevel,
    pub status: PatientStatus,
    pub assigned_staff_name: Option<String>,
    pub ambulance_id: Option<String>,
    pub eta_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl PatientResponse {
    /// Create from Patient entity
    pub fn from_patient(patient: &Patient) -> Self {
        Self {
            id: patient.id,
            patient_number: patient.patient_number.clone(),
            first_name: patient.first_name.clone(),
            last_name: patient.last_name.clone(),
            age: patient.age,
            gender: patient.gender.clone(),
            chief_complaint: patient.chief_complaint.clone(),
            triage_level: patient.triage_level,
            status: patient.status,
            hospital_id: patient.hospital_id,
            hospital_name: None, // Set by service layer
            assigned_staff_id: patient.assigned_staff_id,
            assigned_staff_name: None, // Set by service layer
            ambulance_id: patient.ambulance_id.map(|id| id.to_string()),
            bed_id: patient.bed_id,
            bed_number: None, // Set by service layer
            incident_location: patient.incident_location.clone(),
            incident_time: patient.incident_time,
            latest_vitals: None, // Set by service layer
            allergies: patient.get_allergies(),
            created_at: patient.created_at,
            updated_at: patient.updated_at,
        }
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Get display name (handles anonymous patients)
    pub fn display_name(&self) -> String {
        if self.first_name.is_empty() || self.last_name.is_empty() {
            format!("Anonymous Patient ({})", self.patient_number)
        } else {
            self.full_name()
        }
    }

    /// Check if patient is emergency level
    pub fn is_emergency(&self) -> bool {
        self.triage_level.is_emergency()
    }

    /// Check if patient is currently active
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Get blood pressure string from latest vitals
    pub fn blood_pressure_display(&self) -> String {
        match &self.latest_vitals {
            Some(vitals) => match (vitals.systolic_bp, vitals.diastolic_bp) {
                (Some(sys), Some(dia)) => format!("{}/{}", sys, dia),
                _ => "N/A".to_string(),
            },
            None => "N/A".to_string(),
        }
    }
}

impl VitalsDto {
    /// Create from PatientVitals entity
    pub fn from_vitals(vitals: &PatientVitals) -> Self {
        Self {
            id: vitals.id,
            systolic_bp: vitals.systolic_bp,
            diastolic_bp: vitals.diastolic_bp,
            heart_rate: vitals.heart_rate,
            oxygen_saturation: vitals.oxygen_saturation,
            temperature: vitals.temperature,
            respiratory_rate: vitals.respiratory_rate,
            recorded_by: vitals.recorded_by,
            recorded_by_name: None, // Set by service layer
            recorded_at: vitals.recorded_at,
        }
    }

    /// Get blood pressure as formatted string
    pub fn blood_pressure_display(&self) -> String {
        match (self.systolic_bp, self.diastolic_bp) {
            (Some(sys), Some(dia)) => format!("{}/{}", sys, dia),
            _ => "N/A".to_string(),
        }
    }

    /// Check if vitals are complete
    pub fn is_complete(&self) -> bool {
        self.systolic_bp.is_some()
            && self.diastolic_bp.is_some()
            && self.heart_rate.is_some()
            && self.oxygen_saturation.is_some()
            && self.temperature.is_some()
    }
}

impl PatientSummary {
    /// Create from Patient entity for list views
    pub fn from_patient(patient: &Patient) -> Self {
        Self {
            id: patient.id,
            patient_number: patient.patient_number.clone(),
            display_name: patient.display_name(),
            age: patient.age,
            gender: patient.gender.clone(),
            chief_complaint: patient.chief_complaint.clone(),
            triage_level: patient.triage_level,
            status: patient.status,
            assigned_staff_name: None, // Set by service layer
            ambulance_id: patient.ambulance_id.map(|id| id.to_string()),
            eta_minutes: None, // Calculated by service layer
            created_at: patient.created_at,
        }
    }
}

impl PatientListResponse {
    /// Create paginated response
    pub fn new(patients: Vec<PatientSummary>, total_count: i64, page: i32, page_size: i32) -> Self {
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as i32;
        Self {
            patients,
            total_count,
            page,
            page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Patient;

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
            Some("Sheikh Zayed Road".to_string()),
            Some(Utc::now()),
        )
    }

    #[test]
    fn test_patient_response_creation() {
        let patient = create_test_patient();
        let response = PatientResponse::from_patient(&patient);
        
        assert_eq!(response.id, patient.id);
        assert_eq!(response.patient_number, patient.patient_number);
        assert_eq!(response.full_name(), "Ahmed Al-Rashid");
        assert!(response.is_emergency());
        assert!(response.is_active());
    }

    #[test]
    fn test_anonymous_patient_display() {
        let mut patient = create_test_patient();
        patient.first_name = "".to_string();
        patient.last_name = "".to_string();
        
        let response = PatientResponse::from_patient(&patient);
        assert_eq!(response.display_name(), "Anonymous Patient (PAT-001)");
    }

    #[test]
    fn test_patient_summary() {
        let patient = create_test_patient();
        let summary = PatientSummary::from_patient(&patient);
        
        assert_eq!(summary.id, patient.id);
        assert_eq!(summary.display_name, "Ahmed Al-Rashid");
        assert_eq!(summary.triage_level, TriageLevel::Critical);
    }

    #[test]
    fn test_patient_list_response() {
        let patient = create_test_patient();
        let summaries = vec![PatientSummary::from_patient(&patient)];
        let response = PatientListResponse::new(summaries, 25, 1, 10);
        
        assert_eq!(response.total_count, 25);
        assert_eq!(response.page, 1);
        assert_eq!(response.page_size, 10);
        assert_eq!(response.total_pages, 3); // ceil(25/10) = 3
    }

    #[test]
    fn test_vitals_dto() {
        let vitals = PatientVitals::new(Uuid::new_v4(), Uuid::new_v4());
        let dto = VitalsDto::from_vitals(&vitals);
        
        assert_eq!(dto.id, vitals.id);
        assert_eq!(dto.recorded_by, vitals.recorded_by);
        assert!(!dto.is_complete()); // No vitals set yet
    }

    #[test]
    fn test_serialization() {
        let patient = create_test_patient();
        let response = PatientResponse::from_patient(&patient);
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PatientResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
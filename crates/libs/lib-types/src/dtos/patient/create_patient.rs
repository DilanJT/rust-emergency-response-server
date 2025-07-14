use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::TriageLevel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePatientRequest {
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub gender: String, // "Male", "Female", "Other"
    pub national_id: Option<String>, // Emirates ID
    pub chief_complaint: String,
    pub triage_level: TriageLevel,
    pub hospital_id: Uuid,
    pub incident_location: Option<String>,
    pub incident_time: Option<DateTime<Utc>>,
    pub emergency_contacts: Option<EmergencyContact>,
    pub allergies: Option<Vec<String>>,
    pub medical_history: Option<String>,
    pub insurance_info: Option<InsuranceInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub name: String,
    pub relationship: String,
    pub phone_number: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsuranceInfo {
    pub provider: String,
    pub policy_number: String,
    pub group_number: Option<String>,
    pub member_id: String,
}

impl CreatePatientRequest {
    /// Validate the create patient request
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Required field validations
        if self.first_name.trim().is_empty() {
            errors.push("First name is required".to_string());
        }

        if self.last_name.trim().is_empty() {
            errors.push("Last name is required".to_string());
        }

        if self.age < 0 || self.age > 150 {
            errors.push("Age must be between 0 and 150".to_string());
        }

        if !matches!(self.gender.as_str(), "Male" | "Female" | "Other") {
            errors.push("Gender must be Male, Female, or Other".to_string());
        }

        if self.chief_complaint.trim().is_empty() {
            errors.push("Chief complaint is required".to_string());
        }

        // Emirates ID validation (if provided)
        if let Some(ref national_id) = self.national_id {
            if !national_id.is_empty() && !Self::is_valid_emirates_id(national_id) {
                errors.push("Invalid Emirates ID format".to_string());
            }
        }

        // Emergency contact validation (if provided)
        if let Some(ref contact) = self.emergency_contacts {
            if contact.name.trim().is_empty() {
                errors.push("Emergency contact name is required".to_string());
            }
            if contact.phone_number.trim().is_empty() {
                errors.push("Emergency contact phone is required".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Basic Emirates ID validation (simplified)
    fn is_valid_emirates_id(id: &str) -> bool {
        // Emirates ID format: XXX-YYYY-XXXXXXX-X (15 digits with dashes)
        let clean_id = id.replace("-", "");
        clean_id.len() == 15 && clean_id.chars().all(|c| c.is_ascii_digit())
    }

    /// Get sanitized first name
    pub fn sanitized_first_name(&self) -> String {
        self.first_name.trim().to_string()
    }

    /// Get sanitized last name
    pub fn sanitized_last_name(&self) -> String {
        self.last_name.trim().to_string()
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.sanitized_first_name(), self.sanitized_last_name())
    }

    /// Check if patient is a minor (under 18)
    pub fn is_minor(&self) -> bool {
        self.age < 18
    }

    /// Check if patient is elderly (over 65)
    pub fn is_elderly(&self) -> bool {
        self.age > 65
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_request() -> CreatePatientRequest {
        CreatePatientRequest {
            first_name: "Ahmed".to_string(),
            last_name: "Al-Rashid".to_string(),
            age: 45,
            gender: "Male".to_string(),
            national_id: Some("784-1990-1234567-1".to_string()),
            chief_complaint: "Chest Pain".to_string(),
            triage_level: TriageLevel::High,
            hospital_id: Uuid::new_v4(),
            incident_location: Some("Sheikh Zayed Road".to_string()),
            incident_time: Some(Utc::now()),
            emergency_contacts: Some(EmergencyContact {
                name: "Fatima Al-Rashid".to_string(),
                relationship: "Wife".to_string(),
                phone_number: "+971501234567".to_string(),
                email: Some("fatima@email.com".to_string()),
            }),
            allergies: Some(vec!["Penicillin".to_string()]),
            medical_history: Some("Hypertension".to_string()),
            insurance_info: Some(InsuranceInfo {
                provider: "Dubai Health Insurance".to_string(),
                policy_number: "DH123456".to_string(),
                group_number: None,
                member_id: "MEM789".to_string(),
            }),
        }
    }

    #[test]
    fn test_valid_patient_request() {
        let request = create_valid_request();
        assert!(request.validate().is_ok());
        assert_eq!(request.full_name(), "Ahmed Al-Rashid");
        assert!(!request.is_minor());
        assert!(!request.is_elderly());
    }

    #[test]
    fn test_invalid_patient_request() {
        let mut request = create_valid_request();
        request.first_name = "".to_string();
        request.age = -5;
        request.gender = "Invalid".to_string();
        
        let errors = request.validate().unwrap_err();
        assert!(errors.len() >= 3);
        assert!(errors.iter().any(|e| e.contains("First name")));
        assert!(errors.iter().any(|e| e.contains("Age must be")));
        assert!(errors.iter().any(|e| e.contains("Gender must be")));
    }

    #[test]
    fn test_emirates_id_validation() {
        let mut request = create_valid_request();
        
        // Valid Emirates ID
        request.national_id = Some("784-1990-1234567-1".to_string());
        assert!(request.validate().is_ok());
        
        // Invalid Emirates ID
        request.national_id = Some("invalid-id".to_string());
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_age_categories() {
        let mut request = create_valid_request();
        
        // Minor
        request.age = 15;
        assert!(request.is_minor());
        assert!(!request.is_elderly());
        
        // Adult
        request.age = 35;
        assert!(!request.is_minor());
        assert!(!request.is_elderly());
        
        // Elderly
        request.age = 70;
        assert!(!request.is_minor());
        assert!(request.is_elderly());
    }

    #[test]
    fn test_emergency_contact_validation() {
        let mut request = create_valid_request();
        request.emergency_contacts = Some(EmergencyContact {
            name: "".to_string(), // Invalid empty name
            relationship: "Wife".to_string(),
            phone_number: "".to_string(), // Invalid empty phone
            email: None,
        });
        
        let errors = request.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("contact name")));
        assert!(errors.iter().any(|e| e.contains("contact phone")));
    }

    #[test]
    fn test_serialization() {
        let request = create_valid_request();
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreatePatientRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);
    }
}
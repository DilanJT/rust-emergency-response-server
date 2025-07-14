use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::enums::{PatientStatus, TriageLevel};

#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatientError {
    #[error("Patient not found: {patient_id}")]
    NotFound { patient_id: Uuid },

    #[error("Patient already exists with ID: {national_id}")]
    AlreadyExists { national_id: String },

    #[error("Invalid patient data: {field} - {reason}")]
    InvalidData { field: String, reason: String },

    #[error("Cannot update patient status from {current} to {requested}")]
    InvalidStatusTransition {
        current: PatientStatus,
        requested: PatientStatus,
    },

    #[error("Patient is not assigned to this hospital: {hospital_id}")]
    HospitalMismatch { hospital_id: Uuid },

    #[error("Patient is already assigned to staff member: {staff_id}")]
    AlreadyAssigned { staff_id: Uuid },

    #[error("Cannot assign patient - staff member not available: {staff_id}")]
    StaffNotAvailable { staff_id: Uuid },

    #[error("Patient bed assignment failed - bed not available: {bed_id}")]
    BedNotAvailable { bed_id: Uuid },

    #[error("Triage level change not permitted: {from} to {to} - requires senior staff approval")]
    TriageChangeNotPermitted { from: TriageLevel, to: TriageLevel },

    #[error("Patient is in critical condition - cannot discharge")]
    CriticalConditionDischarge,

    #[error("Patient has unpaid bills - cannot discharge")]
    UnpaidBillsDischarge,

    #[error("Patient vital signs are incomplete or invalid")]
    InvalidVitalSigns,

    #[error("Patient is a minor - guardian consent required")]
    MinorConsentRequired,

    #[error("Patient has allergies to prescribed medication: {medication}")]
    AllergyConflict { medication: String },

    #[error("Patient medical history is incomplete for this procedure")]
    IncompleteHistory,

    #[error("Patient transfer failed - {reason}")]
    TransferFailed { reason: String },

    #[error("Emergency contact information is required for critical patients")]
    EmergencyContactRequired,
}

impl PatientError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            PatientError::NotFound { .. } => 404,
            PatientError::AlreadyExists { .. } => 409, // Conflict
            PatientError::InvalidData { .. } => 400,
            PatientError::InvalidStatusTransition { .. } => 422, // Unprocessable Entity
            PatientError::HospitalMismatch { .. } => 403,
            PatientError::AlreadyAssigned { .. } => 409,
            PatientError::StaffNotAvailable { .. } => 409,
            PatientError::BedNotAvailable { .. } => 409,
            PatientError::TriageChangeNotPermitted { .. } => 403,
            PatientError::CriticalConditionDischarge => 422,
            PatientError::UnpaidBillsDischarge => 402, // Payment Required
            PatientError::InvalidVitalSigns => 400,
            PatientError::MinorConsentRequired => 422,
            PatientError::AllergyConflict { .. } => 422,
            PatientError::IncompleteHistory => 422,
            PatientError::TransferFailed { .. } => 422,
            PatientError::EmergencyContactRequired => 422,
        }
    }

    /// Get error code for client identification
    pub fn error_code(&self) -> &'static str {
        match self {
            PatientError::NotFound { .. } => "PATIENT_NOT_FOUND",
            PatientError::AlreadyExists { .. } => "PATIENT_ALREADY_EXISTS",
            PatientError::InvalidData { .. } => "PATIENT_INVALID_DATA",
            PatientError::InvalidStatusTransition { .. } => "PATIENT_INVALID_STATUS_TRANSITION",
            PatientError::HospitalMismatch { .. } => "PATIENT_HOSPITAL_MISMATCH",
            PatientError::AlreadyAssigned { .. } => "PATIENT_ALREADY_ASSIGNED",
            PatientError::StaffNotAvailable { .. } => "STAFF_NOT_AVAILABLE",
            PatientError::BedNotAvailable { .. } => "BED_NOT_AVAILABLE",
            PatientError::TriageChangeNotPermitted { .. } => "TRIAGE_CHANGE_NOT_PERMITTED",
            PatientError::CriticalConditionDischarge => "CRITICAL_CONDITION_DISCHARGE",
            PatientError::UnpaidBillsDischarge => "UNPAID_BILLS_DISCHARGE",
            PatientError::InvalidVitalSigns => "INVALID_VITAL_SIGNS",
            PatientError::MinorConsentRequired => "MINOR_CONSENT_REQUIRED",
            PatientError::AllergyConflict { .. } => "ALLERGY_CONFLICT",
            PatientError::IncompleteHistory => "INCOMPLETE_HISTORY",
            PatientError::TransferFailed { .. } => "TRANSFER_FAILED",
            PatientError::EmergencyContactRequired => "EMERGENCY_CONTACT_REQUIRED",
        }
    }

    /// Check if this error requires immediate attention
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            PatientError::CriticalConditionDischarge
                | PatientError::AllergyConflict { .. }
                | PatientError::EmergencyContactRequired
        )
    }

    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            PatientError::NotFound { .. } => "Patient record not found".to_string(),
            PatientError::InvalidStatusTransition { current, requested } => {
                format!(
                    "Cannot change patient status from {} to {}",
                    current.display_name(),
                    requested.display_name()
                )
            }
            PatientError::StaffNotAvailable { .. } => {
                "The selected staff member is not available for assignment".to_string()
            }
            PatientError::BedNotAvailable { .. } => {
                "The selected bed is not available".to_string()
            }
            PatientError::AllergyConflict { medication } => {
                format!("Patient is allergic to {}", medication)
            }
            PatientError::MinorConsentRequired => {
                "Guardian consent is required for patients under 18".to_string()
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_error_status_codes() {
        assert_eq!(
            PatientError::NotFound {
                patient_id: Uuid::new_v4()
            }
            .status_code(),
            404
        );
        assert_eq!(
            PatientError::AlreadyExists {
                national_id: "123".to_string()
            }
            .status_code(),
            409
        );
        assert_eq!(PatientError::UnpaidBillsDischarge.status_code(), 402);
    }

    #[test]
    fn test_patient_error_codes() {
        assert_eq!(
            PatientError::NotFound {
                patient_id: Uuid::new_v4()
            }
            .error_code(),
            "PATIENT_NOT_FOUND"
        );
        assert_eq!(
            PatientError::InvalidStatusTransition {
                current: PatientStatus::Dispatched,
                requested: PatientStatus::Discharged
            }
            .error_code(),
            "PATIENT_INVALID_STATUS_TRANSITION"
        );
    }

    #[test]
    fn test_critical_errors() {
        assert!(PatientError::CriticalConditionDischarge.is_critical());
        assert!(PatientError::AllergyConflict {
            medication: "Penicillin".to_string()
        }
        .is_critical());
        assert!(!PatientError::NotFound {
            patient_id: Uuid::new_v4()
        }
        .is_critical());
    }

    #[test]
    fn test_user_messages() {
        let error = PatientError::InvalidStatusTransition {
            current: PatientStatus::EnRoute,
            requested: PatientStatus::Discharged,
        };
        assert!(error.user_message().contains("En Route"));
        assert!(error.user_message().contains("Discharged"));
    }

    #[test]
    fn test_serialization() {
        let error = PatientError::AllergyConflict {
            medication: "Aspirin".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: PatientError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }
}
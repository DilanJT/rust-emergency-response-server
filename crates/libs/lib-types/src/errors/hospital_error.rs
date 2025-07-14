use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum HospitalError {
    #[error("Hospital not found: {hospital_id}")]
    NotFound { hospital_id: Uuid },

    #[error("Hospital is at full capacity - no beds available")]
    AtCapacity,

    #[error("Hospital is not accepting patients - status: {status}")]
    NotAcceptingPatients { status: String },

    #[error("Hospital does not have required specialty: {specialty}")]
    SpecialtyNotAvailable { specialty: String },

    #[error("Bed not found: {bed_id}")]
    BedNotFound { bed_id: Uuid },

    #[error("Bed is already occupied by patient: {patient_id}")]
    BedOccupied { patient_id: Uuid },

    #[error("Invalid bed type for patient triage level")]
    IncompatibleBedType,

    #[error("Equipment not available: {equipment_type}")]
    EquipmentNotAvailable { equipment_type: String },

    #[error("Hospital network communication failed: {reason}")]
    NetworkCommunicationFailed { reason: String },

    #[error("Hospital capacity data is stale - last update: {last_update}")]
    StaleCapacityData { last_update: String },

    #[error("Cannot update hospital capacity - invalid bed count: {requested}")]
    InvalidCapacityUpdate { requested: i32 },

    #[error("Hospital is under maintenance - emergency only")]
    UnderMaintenance,

    #[error("Hospital transfer protocol violation: {reason}")]
    TransferProtocolViolation { reason: String },

    #[error("Hospital license validation failed")]
    LicenseValidationFailed,

    #[error("Hospital regional restrictions apply")]
    RegionalRestrictions,
}

impl HospitalError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            HospitalError::NotFound { .. } => 404,
            HospitalError::AtCapacity => 503, // Service Unavailable
            HospitalError::NotAcceptingPatients { .. } => 503,
            HospitalError::SpecialtyNotAvailable { .. } => 422,
            HospitalError::BedNotFound { .. } => 404,
            HospitalError::BedOccupied { .. } => 409, // Conflict
            HospitalError::IncompatibleBedType => 422,
            HospitalError::EquipmentNotAvailable { .. } => 503,
            HospitalError::NetworkCommunicationFailed { .. } => 502, // Bad Gateway
            HospitalError::StaleCapacityData { .. } => 409,
            HospitalError::InvalidCapacityUpdate { .. } => 400,
            HospitalError::UnderMaintenance => 503,
            HospitalError::TransferProtocolViolation { .. } => 422,
            HospitalError::LicenseValidationFailed => 403,
            HospitalError::RegionalRestrictions => 403,
        }
    }

    /// Get error code for client identification
    pub fn error_code(&self) -> &'static str {
        match self {
            HospitalError::NotFound { .. } => "HOSPITAL_NOT_FOUND",
            HospitalError::AtCapacity => "HOSPITAL_AT_CAPACITY",
            HospitalError::NotAcceptingPatients { .. } => "HOSPITAL_NOT_ACCEPTING_PATIENTS",
            HospitalError::SpecialtyNotAvailable { .. } => "SPECIALTY_NOT_AVAILABLE",
            HospitalError::BedNotFound { .. } => "BED_NOT_FOUND",
            HospitalError::BedOccupied { .. } => "BED_OCCUPIED",
            HospitalError::IncompatibleBedType => "INCOMPATIBLE_BED_TYPE",
            HospitalError::EquipmentNotAvailable { .. } => "EQUIPMENT_NOT_AVAILABLE",
            HospitalError::NetworkCommunicationFailed { .. } => "NETWORK_COMMUNICATION_FAILED",
            HospitalError::StaleCapacityData { .. } => "STALE_CAPACITY_DATA",
            HospitalError::InvalidCapacityUpdate { .. } => "INVALID_CAPACITY_UPDATE",
            HospitalError::UnderMaintenance => "HOSPITAL_UNDER_MAINTENANCE",
            HospitalError::TransferProtocolViolation { .. } => "TRANSFER_PROTOCOL_VIOLATION",
            HospitalError::LicenseValidationFailed => "LICENSE_VALIDATION_FAILED",
            HospitalError::RegionalRestrictions => "REGIONAL_RESTRICTIONS",
        }
    }

    /// Check if this is a capacity-related error
    pub fn is_capacity_issue(&self) -> bool {
        matches!(
            self,
            HospitalError::AtCapacity
                | HospitalError::BedOccupied { .. }
                | HospitalError::BedNotFound { .. }
                | HospitalError::StaleCapacityData { .. }
        )
    }

    /// Check if hospital is temporarily unavailable
    pub fn is_temporary_unavailable(&self) -> bool {
        matches!(
            self,
            HospitalError::AtCapacity
                | HospitalError::NotAcceptingPatients { .. }
                | HospitalError::UnderMaintenance
                | HospitalError::EquipmentNotAvailable { .. }
        )
    }

    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            HospitalError::NotFound { .. } => "Hospital not found".to_string(),
            HospitalError::AtCapacity => {
                "Hospital is at full capacity. Please try another hospital".to_string()
            }
            HospitalError::NotAcceptingPatients { status } => {
                format!("Hospital is not currently accepting patients ({})", status)
            }
            HospitalError::SpecialtyNotAvailable { specialty } => {
                format!("This hospital does not have {} services", specialty)
            }
            HospitalError::BedOccupied { .. } => "The selected bed is already occupied".to_string(),
            HospitalError::UnderMaintenance => {
                "Hospital is under maintenance - emergency cases only".to_string()
            }
            HospitalError::EquipmentNotAvailable { equipment_type } => {
                format!("{} is not currently available", equipment_type)
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hospital_error_status_codes() {
        assert_eq!(
            HospitalError::NotFound {
                hospital_id: Uuid::new_v4()
            }
            .status_code(),
            404
        );
        assert_eq!(HospitalError::AtCapacity.status_code(), 503);
        assert_eq!(
            HospitalError::NetworkCommunicationFailed {
                reason: "timeout".to_string()
            }
            .status_code(),
            502
        );
    }

    #[test]
    fn test_hospital_error_codes() {
        assert_eq!(HospitalError::AtCapacity.error_code(), "HOSPITAL_AT_CAPACITY");
        assert_eq!(
            HospitalError::SpecialtyNotAvailable {
                specialty: "Cardiology".to_string()
            }
            .error_code(),
            "SPECIALTY_NOT_AVAILABLE"
        );
    }

    #[test]
    fn test_capacity_issues() {
        assert!(HospitalError::AtCapacity.is_capacity_issue());
        assert!(HospitalError::BedOccupied {
            patient_id: Uuid::new_v4()
        }
        .is_capacity_issue());
        assert!(!HospitalError::LicenseValidationFailed.is_capacity_issue());
    }

    #[test]
    fn test_temporary_unavailable() {
        assert!(HospitalError::AtCapacity.is_temporary_unavailable());
        assert!(HospitalError::UnderMaintenance.is_temporary_unavailable());
        assert!(!HospitalError::LicenseValidationFailed.is_temporary_unavailable());
    }

    #[test]
    fn test_user_messages() {
        let error = HospitalError::SpecialtyNotAvailable {
            specialty: "Neurology".to_string(),
        };
        assert!(error.user_message().contains("Neurology"));
    }

    #[test]
    fn test_serialization() {
        let error = HospitalError::BedOccupied {
            patient_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: HospitalError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }
}
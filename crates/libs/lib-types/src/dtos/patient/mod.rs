//! Patient DTOs

pub mod create_patient;
pub mod patient_response;

pub use create_patient::{CreatePatientRequest, EmergencyContact, InsuranceInfo};
pub use patient_response::{PatientResponse, PatientSummary, PatientListResponse, VitalsDto};
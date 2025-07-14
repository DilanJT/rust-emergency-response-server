// pub mod errors;
pub mod auth_error;
pub mod patient_error;
pub mod hospital_error;
pub mod app_error;

// Re-exports for convenience
pub use auth_error::AuthError;
pub use patient_error::PatientError;
pub use hospital_error::HospitalError;
pub use app_error::{AppError, ApiErrorResponse};
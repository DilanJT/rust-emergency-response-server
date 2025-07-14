// pub mod entities;

pub mod user;
pub mod hospital;
pub mod patient;
pub mod medical_staff;
pub mod patient_vitals;

pub use user::{User, UserProfile};
pub use hospital::Hospital;
pub use patient::Patient;
pub use medical_staff::MedicalStaff;
pub use patient_vitals::{PatientVitals, VitalStatus};

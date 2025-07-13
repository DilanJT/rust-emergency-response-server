// pub mod enums;

pub mod user_role;
pub mod triage_level;
pub mod patient_status;
pub mod availability_status;
pub mod bed_type;

pub use user_role::UserRole;
pub use triage_level::TriageLevel;
pub use patient_status::PatientStatus;
pub use availability_status::AvailabilityStatus;
pub use bed_type::BedType;
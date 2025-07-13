use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "patient_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PatientStatus {
    Dispatched,
    EnRoute,
    Arrived,
    Admitted,
    Discharged,
}

impl PatientStatus {
    /// Get display name for patient status
    pub fn display_name(&self) -> &'static str {
        match self {
            PatientStatus::Dispatched => "Dispatched",
            PatientStatus::EnRoute => "En Route",
            PatientStatus::Arrived => "Arrived",
            PatientStatus::Admitted => "Admitted",
            PatientStatus::Discharged => "Discharged",
        }
    }

    /// Get next possible statuses from current status
    pub fn next_statuses(&self) -> Vec<PatientStatus> {
        match self {
            PatientStatus::Dispatched => vec![PatientStatus::EnRoute],
            PatientStatus::EnRoute => vec![PatientStatus::Arrived],
            PatientStatus::Arrived => vec![PatientStatus::Admitted],
            PatientStatus::Admitted => vec![PatientStatus::Discharged],
            PatientStatus::Discharged => vec![], // Terminal status
        }
    }

    /// Check if status indicates patient is in transport
    pub fn is_in_transport(&self) -> bool {
        matches!(self, PatientStatus::Dispatched | PatientStatus::EnRoute)
    }

    /// Check if status indicates patient is at hospital
    pub fn is_at_hospital(&self) -> bool {
        matches!(
            self,
            PatientStatus::Arrived | PatientStatus::Admitted | PatientStatus::Discharged
        )
    }

    /// Check if patient is currently receiving care
    pub fn is_active(&self) -> bool {
        !matches!(self, PatientStatus::Discharged)
    }

    /// Get status workflow order
    pub fn workflow_order(&self) -> u8 {
        match self {
            PatientStatus::Dispatched => 1,
            PatientStatus::EnRoute => 2,
            PatientStatus::Arrived => 3,
            PatientStatus::Admitted => 4,
            PatientStatus::Discharged => 5,
        }
    }
}

impl std::fmt::Display for PatientStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_workflow() {
        assert_eq!(PatientStatus::Dispatched.next_statuses(), vec![PatientStatus::EnRoute]);
        assert_eq!(PatientStatus::EnRoute.next_statuses(), vec![PatientStatus::Arrived]);
        assert_eq!(PatientStatus::Admitted.next_statuses(), vec![PatientStatus::Discharged]);
        assert!(PatientStatus::Discharged.next_statuses().is_empty());
    }

    #[test]
    fn test_transport_status() {
        assert!(PatientStatus::Dispatched.is_in_transport());
        assert!(PatientStatus::EnRoute.is_in_transport());
        assert!(!PatientStatus::Arrived.is_in_transport());
        assert!(!PatientStatus::Admitted.is_in_transport());
    }

    #[test]
    fn test_hospital_status() {
        assert!(!PatientStatus::Dispatched.is_at_hospital());
        assert!(!PatientStatus::EnRoute.is_at_hospital());
        assert!(PatientStatus::Arrived.is_at_hospital());
        assert!(PatientStatus::Admitted.is_at_hospital());
        assert!(PatientStatus::Discharged.is_at_hospital());
    }

    #[test]
    fn test_active_status() {
        assert!(PatientStatus::Dispatched.is_active());
        assert!(PatientStatus::Admitted.is_active());
        assert!(!PatientStatus::Discharged.is_active());
    }

    #[test]
    fn test_workflow_order() {
        assert!(PatientStatus::Dispatched.workflow_order() < PatientStatus::EnRoute.workflow_order());
        assert!(PatientStatus::Arrived.workflow_order() < PatientStatus::Admitted.workflow_order());
    }
}
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "availability_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    Available,
    Busy,
    OffDuty,
    OnCall,
}

impl AvailabilityStatus {
    /// Get display name for availability status
    pub fn display_name(&self) -> &'static str {
        match self {
            AvailabilityStatus::Available => "Available",
            AvailabilityStatus::Busy => "Busy",
            AvailabilityStatus::OffDuty => "Off Duty",
            AvailabilityStatus::OnCall => "On Call",
        }
    }

    /// Check if staff member can take new assignments
    pub fn can_take_assignment(&self) -> bool {
        matches!(self, AvailabilityStatus::Available | AvailabilityStatus::OnCall)
    }

    /// Check if staff member is currently working
    pub fn is_working(&self) -> bool {
        matches!(
            self,
            AvailabilityStatus::Available | AvailabilityStatus::Busy | AvailabilityStatus::OnCall
        )
    }

    /// Get priority for assignment (lower number = higher priority)
    pub fn assignment_priority(&self) -> u8 {
        match self {
            AvailabilityStatus::Available => 1,
            AvailabilityStatus::OnCall => 2,
            AvailabilityStatus::Busy => 3,
            AvailabilityStatus::OffDuty => 4,
        }
    }

    /// Get status indicator color for UI
    pub fn status_color(&self) -> &'static str {
        match self {
            AvailabilityStatus::Available => "#2ecc71", // Green
            AvailabilityStatus::Busy => "#e74c3c",      // Red
            AvailabilityStatus::OffDuty => "#95a5a6",   // Gray
            AvailabilityStatus::OnCall => "#f39c12",    // Orange
        }
    }
}

impl std::fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_capability() {
        assert!(AvailabilityStatus::Available.can_take_assignment());
        assert!(AvailabilityStatus::OnCall.can_take_assignment());
        assert!(!AvailabilityStatus::Busy.can_take_assignment());
        assert!(!AvailabilityStatus::OffDuty.can_take_assignment());
    }

    #[test]
    fn test_working_status() {
        assert!(AvailabilityStatus::Available.is_working());
        assert!(AvailabilityStatus::Busy.is_working());
        assert!(AvailabilityStatus::OnCall.is_working());
        assert!(!AvailabilityStatus::OffDuty.is_working());
    }

    #[test]
    fn test_assignment_priority() {
        assert!(AvailabilityStatus::Available.assignment_priority() < AvailabilityStatus::OnCall.assignment_priority());
        assert!(AvailabilityStatus::OnCall.assignment_priority() < AvailabilityStatus::Busy.assignment_priority());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", AvailabilityStatus::OffDuty), "Off Duty");
        assert_eq!(format!("{}", AvailabilityStatus::OnCall), "On Call");
    }
}
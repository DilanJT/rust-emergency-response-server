use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::Hospital;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HospitalResponse {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub phone_number: String,
    pub email: String,
    pub total_beds: i32,
    pub available_beds: i32,
    pub specialties: Vec<String>,
    pub hospital_type: String,
    pub status: String,
    pub capacity_status: CapacityStatus,
    pub distance_km: Option<f64>, // Distance from user's location
    pub eta_minutes: Option<i32>, // Estimated time of arrival
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapacityStatus {
    pub occupancy_percentage: f64,
    pub status_text: String,
    pub status_color: String,
    pub is_accepting_patients: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HospitalListResponse {
    pub hospitals: Vec<HospitalSummary>,
    pub total_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HospitalSummary {
    pub id: Uuid,
    pub name: String,
    pub available_beds: i32,
    pub total_beds: i32,
    pub occupancy_percentage: f64,
    pub status: String,
    pub distance_km: Option<f64>,
    pub eta_minutes: Option<i32>,
    pub has_specialty: Option<bool>, // If filtering by specialty
}

impl HospitalResponse {
    /// Create from Hospital entity
    pub fn from_hospital(hospital: &Hospital) -> Self {
        let capacity_status = CapacityStatus {
            occupancy_percentage: hospital.occupancy_percentage(),
            status_text: hospital.capacity_status().to_string(),
            status_color: hospital.capacity_color().to_string(),
            is_accepting_patients: hospital.has_available_beds() && hospital.status == "Active",
        };

        Self {
            id: hospital.id,
            name: hospital.name.clone(),
            address: hospital.address.clone(),
            phone_number: hospital.phone_number.clone(),
            email: hospital.email.clone(),
            total_beds: hospital.total_beds,
            available_beds: hospital.available_beds,
            specialties: hospital.get_specialties(),
            hospital_type: hospital.hospital_type.clone(),
            status: hospital.status.clone(),
            capacity_status,
            distance_km: None, // Set by service layer
            eta_minutes: None, // Set by service layer
            created_at: hospital.created_at,
        }
    }

    /// Check if hospital can accept new patients
    pub fn can_accept_patients(&self) -> bool {
        self.capacity_status.is_accepting_patients
    }

    /// Check if hospital has specific specialty
    pub fn has_specialty(&self, specialty: &str) -> bool {
        self.specialties
            .iter()
            .any(|s| s.eq_ignore_ascii_case(specialty))
    }

    /// Get capacity indicator for UI
    pub fn capacity_indicator(&self) -> &str {
        if self.available_beds == 0 {
            "🔴" // Full
        } else if self.capacity_status.occupancy_percentage > 90.0 {
            "🟡" // Nearly full
        } else {
            "🟢" // Available
        }
    }
}

impl HospitalSummary {
    /// Create from Hospital entity for list views
    pub fn from_hospital(hospital: &Hospital) -> Self {
        Self {
            id: hospital.id,
            name: hospital.name.clone(),
            available_beds: hospital.available_beds,
            total_beds: hospital.total_beds,
            occupancy_percentage: hospital.occupancy_percentage(),
            status: hospital.status.clone(),
            distance_km: None, // Set by service layer
            eta_minutes: None, // Set by service layer
            has_specialty: None, // Set when filtering
        }
    }

    /// Get capacity indicator
    pub fn capacity_indicator(&self) -> &str {
        if self.available_beds == 0 {
            "🔴"
        } else if self.occupancy_percentage > 90.0 {
            "🟡"
        } else {
            "🟢"
        }
    }
}

impl HospitalListResponse {
    /// Create list response
    pub fn new(hospitals: Vec<HospitalSummary>) -> Self {
        let total_count = hospitals.len() as i64;
        Self {
            hospitals,
            total_count,
        }
    }

    /// Sort hospitals by availability (available beds first)
    pub fn sort_by_availability(mut self) -> Self {
        self.hospitals.sort_by(|a, b| {
            // First by available beds (descending), then by name
            b.available_beds
                .cmp(&a.available_beds)
                .then_with(|| a.name.cmp(&b.name))
        });
        self
    }

    /// Sort hospitals by distance (if available)
    pub fn sort_by_distance(mut self) -> Self {
        self.hospitals.sort_by(|a, b| {
            match (a.distance_km, b.distance_km) {
                (Some(dist_a), Some(dist_b)) => dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });
        self
    }

    /// Filter hospitals with available beds
    pub fn with_available_beds(mut self) -> Self {
        self.hospitals.retain(|h| h.available_beds > 0);
        self.total_count = self.hospitals.len() as i64;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hospital() -> Hospital {
        Hospital::new(
            "Dubai Hospital".to_string(),
            "DHA-001".to_string(),
            "25.2697,55.3094".to_string(),
            "Oud Metha, Dubai, UAE".to_string(),
            "+97143193000".to_string(),
            "info@dubaihospital.ae".to_string(),
            100,
            vec!["Emergency Medicine".to_string(), "Cardiology".to_string()],
            "Public".to_string(),
        )
    }

    #[test]
    fn test_hospital_response_creation() {
        let hospital = create_test_hospital();
        let response = HospitalResponse::from_hospital(&hospital);
        
        assert_eq!(response.id, hospital.id);
        assert_eq!(response.name, hospital.name);
        assert!(response.can_accept_patients());
        assert!(response.has_specialty("Emergency Medicine"));
        assert_eq!(response.capacity_indicator(), "🟢");
    }

    #[test]
    fn test_capacity_status() {
        let mut hospital = create_test_hospital();
        
        // Nearly full hospital
        hospital.update_available_beds(5); // 95% occupancy
        let response = HospitalResponse::from_hospital(&hospital);
        assert_eq!(response.capacity_indicator(), "🟡");
        
        // Full hospital
        hospital.update_available_beds(0);
        let response = HospitalResponse::from_hospital(&hospital);
        assert_eq!(response.capacity_indicator(), "🔴");
        assert!(!response.can_accept_patients());
    }

    #[test]
    fn test_hospital_summary() {
        let hospital = create_test_hospital();
        let summary = HospitalSummary::from_hospital(&hospital);
        
        assert_eq!(summary.id, hospital.id);
        assert_eq!(summary.name, hospital.name);
        assert_eq!(summary.available_beds, 100);
        assert_eq!(summary.capacity_indicator(), "🟢");
    }

    #[test]
    fn test_hospital_list_operations() {
        let mut hospital1 = create_test_hospital();
        hospital1.name = "Hospital A".to_string();
        hospital1.update_available_beds(50);
        
        let mut hospital2 = create_test_hospital();
        hospital2.name = "Hospital B".to_string();
        hospital2.update_available_beds(0);
        
        let mut hospital3 = create_test_hospital();
        hospital3.name = "Hospital C".to_string();
        hospital3.update_available_beds(75);
        
        let summaries = vec![
            HospitalSummary::from_hospital(&hospital1),
            HospitalSummary::from_hospital(&hospital2),
            HospitalSummary::from_hospital(&hospital3),
        ];
        
        let response = HospitalListResponse::new(summaries);
        assert_eq!(response.total_count, 3);
        
        // Test filtering by available beds
        let available_only = response.with_available_beds();
        assert_eq!(available_only.total_count, 2); // Excludes hospital B (0 beds)
        
        // Test sorting by availability
        let sorted = HospitalListResponse::new(vec![
            HospitalSummary::from_hospital(&hospital1),
            HospitalSummary::from_hospital(&hospital2),
            HospitalSummary::from_hospital(&hospital3),
        ]).sort_by_availability();
        
        // Should be ordered: Hospital C (75), Hospital A (50), Hospital B (0)
        assert_eq!(sorted.hospitals[0].name, "Hospital C");
        assert_eq!(sorted.hospitals[1].name, "Hospital A");
        assert_eq!(sorted.hospitals[2].name, "Hospital B");
    }

    #[test]
    fn test_serialization() {
        let hospital = create_test_hospital();
        let response = HospitalResponse::from_hospital(&hospital);
        
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: HospitalResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}
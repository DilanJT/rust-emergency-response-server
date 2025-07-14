use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Hospital {
    pub id: Uuid,
    pub name: String,
    pub license_number: String,
    // Note: Using string for location for now, have to upgrate to PostGIS later
    pub location: String,
    pub address: String,
    pub phone_number: String,
    pub email: String,
    pub total_beds: i32,
    pub available_beds: i32,
    pub specialties: serde_json::Value, // JSON arrray of specialties
    pub hospital_type: String, // e.g. "Public", "Specialized", "Private"
    pub status: String, // Active, Maintenance, Emergency Only
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Hospital {
    pub fn new(
        name: String,
        license_number: String,
        location: String,
        address: String,
        phone_number: String,
        email: String,
        total_beds: i32,
        specialties: Vec<String>,
        hospital_type: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            license_number,
            location,
            address,
            phone_number,
            email,
            total_beds,
            available_beds: total_beds,
            specialties: serde_json::to_value(specialties).unwrap_or(serde_json::Value::Array(vec![])),
            hospital_type,
            status: "Active".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn occupancy_percentage(&self) -> f64 {
        if self.total_beds == 0 {
            return 0.0;
        }
        let occupied_beds = self.total_beds - self.available_beds;
        (occupied_beds as f64 / self.total_beds as f64) * 100.0
    }

    pub fn has_available_beds(&self) -> bool {
        self.available_beds > 0
    }

    pub fn is_at_capacity(&self) -> bool {
        self.available_beds <= 0
    }

    pub fn is_nearly_full(&self) -> bool {
        self.occupancy_percentage() > 90.0
    }

    pub fn capacity_status(&self) -> &'static str {
        if self.is_at_capacity() {
            "Full"
        } else if self.is_nearly_full() {
            "Nearly Full"
        } else if self.occupancy_percentage() > 70.0 {
            "Busy"
        } else {
            "Available"
        }
    }

    pub fn get_specialties(&self) -> Vec<String> {
        self.specialties
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn has_specialty(&self, specialty: &str) -> bool {
        self.get_specialties()
            .iter()
            .any(|s| s.eq_ignore_ascii_case(specialty))
    }

    pub fn update_available_beds(&mut self, available_beds: i32) {
        self.available_beds = available_beds.max(0).min(self.total_beds);
        self.updated_at = Utc::now();
    }

    pub fn capacity_color(&self) -> &'static str {
        if self.is_at_capacity() {
            "red"
        } else if self.is_nearly_full() {
            "yellow"
        } else {
            "green"
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hospital() -> Hospital {
        Hospital::new(
            "Dubai Hospital".to_string(),
            "DHA-001".to_string(),
            "25.2697,55.3094".to_string(), // Dubai coordinates
            "Oud Metha, Dubai, UAE".to_string(),
            "+97143193000".to_string(),
            "info@dubaihospital.ae".to_string(),
            100,
            vec!["Emergency Medicine".to_string(), "Cardiology".to_string()],
            "Public".to_string(),
        )
    }

    #[test]
    fn test_hospital_creation() {
        let hospital = create_test_hospital();
        assert_eq!(hospital.name, "Dubai Hospital");
        assert_eq!(hospital.total_beds, 100);
        assert_eq!(hospital.available_beds, 100);
        assert!(hospital.has_available_beds());
    }

    #[test]
    fn test_occupancy_calculation() {
        let mut hospital = create_test_hospital();
        assert_eq!(hospital.occupancy_percentage(), 0.0);

        hospital.update_available_beds(50);
        assert_eq!(hospital.occupancy_percentage(), 50.0);

        hospital.update_available_beds(0);
        assert_eq!(hospital.occupancy_percentage(), 100.0);
    }

    #[test]
    fn test_capacity_status() {
        let mut hospital = create_test_hospital();
        
        // Available
        hospital.update_available_beds(50);
        assert_eq!(hospital.capacity_status(), "Available");
        
        // Busy
        hospital.update_available_beds(25);
        assert_eq!(hospital.capacity_status(), "Busy");
        
        // Nearly Full
        hospital.update_available_beds(5);
        assert_eq!(hospital.capacity_status(), "Nearly Full");
        
        // Full
        hospital.update_available_beds(0);
        assert_eq!(hospital.capacity_status(), "Full");
        assert!(hospital.is_at_capacity());
    }

    #[test]
    fn test_specialties() {
        let hospital = create_test_hospital();
        let specialties = hospital.get_specialties();
        
        assert!(specialties.contains(&"Emergency Medicine".to_string()));
        assert!(specialties.contains(&"Cardiology".to_string()));
        
        assert!(hospital.has_specialty("Emergency Medicine"));
        assert!(hospital.has_specialty("cardiology")); // Case insensitive
        assert!(!hospital.has_specialty("Neurology"));
    }

    #[test]
    fn test_bed_updates() {
        let mut hospital = create_test_hospital();
        
        // Normal update
        hospital.update_available_beds(75);
        assert_eq!(hospital.available_beds, 75);
        
        // Can't go below 0
        hospital.update_available_beds(-10);
        assert_eq!(hospital.available_beds, 0);
        
        // Can't exceed total beds
        hospital.update_available_beds(150);
        assert_eq!(hospital.available_beds, 100);
    }

    #[test]
    fn test_serialization() {
        let hospital = create_test_hospital();
        let json = serde_json::to_string(&hospital).unwrap();
        let deserialized: Hospital = serde_json::from_str(&json).unwrap();
        assert_eq!(hospital, deserialized);
    }
}
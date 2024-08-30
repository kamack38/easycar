use std::fmt;

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize, Debug, Clone)]
pub enum LicenseCategory {
    A,
    A1,
    A2,
    AM,
    B,
    B1,
    BE,
    C,
    C1,
    CE,
    C1E,
    D,
    D1,
    DE,
    D1E,
    T,
    PT,
}

impl fmt::Display for LicenseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self {
            LicenseCategory::A => "A",
            LicenseCategory::A1 => "A1",
            LicenseCategory::A2 => "A2",
            LicenseCategory::AM => "AM",
            LicenseCategory::B => "B",
            LicenseCategory::B1 => "B1",
            LicenseCategory::BE => "BE",
            LicenseCategory::C => "C",
            LicenseCategory::C1 => "C1",
            LicenseCategory::CE => "CE",
            LicenseCategory::C1E => "C1E",
            LicenseCategory::D => "D",
            LicenseCategory::D1 => "D1",
            LicenseCategory::DE => "DE",
            LicenseCategory::D1E => "D1E",
            LicenseCategory::T => "T",
            LicenseCategory::PT => "PT",
        };
        write!(f, "{}", category)
    }
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct ReservationStatus {
    pub status: String,    // TODO: Convert to a type
    pub timestamp: String, // TODO: Convert to date type
    pub message: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TheoryOrPracticeExam {
    pub exam_id: String,
    pub additional_info: String,
    pub date: String, // TODO: Convert to date type
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub organization_unit_id: i32,
    pub organization_unit_name: String,
    pub theory: Option<TheoryOrPracticeExam>,
    pub practice: Option<TheoryOrPracticeExam>,
    pub category: LicenseCategory, // TODO: Convert to a type
    pub address: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Candidate {
    pub firstname: String,
    pub lastname: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Reservation {
    pub awaiting_reschedule: bool,
    pub status: ReservationStatus,
    pub exam: Exam,
    pub candidate: Candidate,
    pub id: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Reservations {
    pub items: Vec<Reservation>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub count: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_pages: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_count: i32,
}

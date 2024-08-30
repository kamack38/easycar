use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

use super::reservations::LicenseCategory;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    pub id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub places: i32, // Free places
    pub date: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: i32, // Price
    pub additional_info: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Hour {
    pub time: String,
    pub theory_exams: Vec<Exam>,
    pub practice_exams: Vec<Exam>,
    pub linked_exams_dto: Vec<Exam>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub day: String,
    pub scheduled_hours: Vec<Hour>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub scheduled_days: Vec<Day>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExamSchedule {
    pub organization_id: String,
    pub is_osk_vehicle_reservation_enabled: bool,
    pub is_reschedule_reservation: bool,
    pub category: LicenseCategory,
    pub schedule: Schedule,
}

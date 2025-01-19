use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::num::NonZeroU32;

use crate::types::LicenseCategory;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    /// Exam id
    pub id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// Number of free places for the exam
    pub places: i32,
    /// Date of the exam
    pub date: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// Exam price
    pub amount: i32,
    pub additional_info: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hour {
    pub time: String,
    pub theory_exams: Vec<Exam>,
    pub practice_exams: Vec<Exam>,
    pub linked_exams_dto: Vec<Exam>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub day: String,
    pub scheduled_hours: Vec<Hour>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub scheduled_days: Vec<Day>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSchedule {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// WORD center ID
    pub organization_id: NonZeroU32,
    /// Can one reserve his own vehicle
    pub is_osk_vehicle_reservation_enabled: bool,
    /// Is it possible to reschedule
    pub is_reschedule_reservation: bool,
    /// Driving license category
    pub category: LicenseCategory,
    pub schedule: Schedule,
}

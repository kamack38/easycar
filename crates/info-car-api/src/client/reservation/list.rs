use std::num::NonZeroU32;

use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

use super::{LicenseCategory, Status};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TheoryOrPracticeExam {
    pub exam_id: String,
    pub additional_info: String,
    pub date: String,
}

// TODO: Convert theory or pracitce to a type
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub organization_unit_id: NonZeroU32,
    pub organization_unit_name: String,
    pub theory: Option<TheoryOrPracticeExam>,
    pub practice: Option<TheoryOrPracticeExam>,
    pub category: LicenseCategory,
    pub address: String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Candidate {
    pub firstname: String,
    pub lastname: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationListItem {
    pub awaiting_reschedule: bool,
    pub status: Status,
    pub exam: Exam,
    pub candidate: Candidate,
    pub id: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationList {
    pub items: Vec<ReservationListItem>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub count: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_pages: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_count: i32,
}

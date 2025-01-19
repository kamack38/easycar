use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::num::NonZeroU32;

use crate::types::{LicenseCategory, Status, TheoryOrPracticeExam};

// TODO: Convert theory or pracitce to a type
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationExam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub organization_unit_id: NonZeroU32,
    pub organization_unit_name: String,
    pub theory: Option<TheoryOrPracticeExam>,
    pub practice: Option<TheoryOrPracticeExam>,
    pub category: LicenseCategory,
    pub address: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct Candidate {
    pub firstname: String,
    pub lastname: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationListItem {
    pub awaiting_reschedule: bool,
    pub status: Status,
    pub exam: ReservationExam,
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

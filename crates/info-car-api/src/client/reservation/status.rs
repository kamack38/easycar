use super::{LicenseCategory, Status};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationStatus {
    pub id: String,
    pub user_id: String,
    pub word_reservation_id: String,
    pub status: Status,
    pub updated_at: String,
    pub candidate: ReservationCandidate,
    pub exam: ReservationExam,
    pub is_reminder_sent: Option<String>,
    pub is_first_reminder_sent: Option<String>,
    pub invoice: Option<ReservationInvoice>,
    pub cancellation_message: Option<String>,
    pub active_payment: Option<String>,
    pub awaiting_reschedule: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationCandidate {
    pub pkk: String,
    pub firstname: String,
    pub lastname: String,
    pub pesel: String,
    pub birth_date: Option<String>,
    pub phone_number: String,
    pub email: String,
    pub language: String,
}

// TODO: Convert to an exam enum (thory or pracitce)
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationExam {
    pub organization_unit_id: String,
    pub organization_unit_name: String,
    pub latitude: String,
    pub longitude: String,
    pub address: String,
    pub province: String,
    pub confirming_operator: Option<String>,
    pub confirmation_record_number: Option<String>,
    pub category: LicenseCategory,
    pub theory: Option<TheoryOrPracticeExam>,
    pub pracitce: Option<TheoryOrPracticeExam>,
    pub osk_vehicle_number: Option<String>,
    pub sign_language: String,
    pub exam_date: String,
    pub start_date: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TheoryOrPracticeExam {
    pub exam_id: String,
    pub date: String,
    pub additional_info: String,
    pub room: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationInvoice {
    pub account_balance: i32,
    pub exam_price: i32,
    pub surcharge: i32,
    pub provision: i32,
}

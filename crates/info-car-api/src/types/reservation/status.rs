use core::fmt;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::num::NonZeroU32;

use crate::types::{LicenseCategory, TheoryOrPracticeExam};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Status {
    pub status: PossibleStatuses,
    pub timestamp: String, // TODO: Convert to date type
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PossibleStatuses {
    Created,
    Draft,
    PaymentRejected,
    PlaceReserved,
    SignupConfirmed,
    CancellationRequest,
    Cancelled,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for PossibleStatuses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            PossibleStatuses::Created => "Created",
            PossibleStatuses::Draft => "Draft",
            PossibleStatuses::PaymentRejected => "Payment Rejected",
            PossibleStatuses::PlaceReserved => "Place Reserved",
            PossibleStatuses::SignupConfirmed => "Signup Confirmed",
            PossibleStatuses::CancellationRequest => "Cancellation Request",
            PossibleStatuses::Cancelled => "Cancelled",
            PossibleStatuses::Unknown => "Unknown",
        };
        write!(f, "{}", status_str)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationStatus {
    pub id: String,
    pub user_id: String,
    pub word_reservation_id: String,
    pub status: Status,
    pub updated_at: String,
    pub candidate: ReservationCandidate,
    pub exam: DetailedReservationExam,
    pub is_reminder_sent: Option<String>,
    pub is_first_reminder_sent: Option<String>,
    pub invoice: Option<ReservationInvoice>,
    pub cancellation_message: Option<String>,
    pub active_payment: Option<String>,
    pub awaiting_reschedule: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationCandidate {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub phone_number: String,
    pub birth_date: Option<String>,
    pub pesel: String,
    pub language: String,
    pub pkk: String,
}

// TODO: Convert to an exam enum (thory or pracitce)
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedReservationExam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub organization_unit_id: NonZeroU32,
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationInvoice {
    pub account_balance: u32,
    pub exam_price: u32,
    pub surcharge: u32,
    pub provision: u32,
}

use core::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod list;
pub mod new;
pub mod payment;
pub mod status;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl Default for LicenseCategory {
    fn default() -> Self {
        LicenseCategory::B
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Status {
    pub status: PossibleStatuses,
    pub timestamp: String, // TODO: Convert to date type
    pub message: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EndpointResponse<T> {
    #[serde(rename = "errors")]
    Errors(Vec<GenericError>),
    #[serde(untagged)]
    Success(T),
}

#[derive(Error, Debug)]
#[error("{}", .0.iter().map(|v| format!("{} ({}). ", v.user_message, v.code)).collect::<String>())]
pub struct GenericEndpointError(Vec<GenericError>);

impl<T> EndpointResponse<T> {
    pub fn ok(self) -> Result<T, GenericEndpointError> {
        use EndpointResponse::*;
        match self {
            Success(v) => Ok(v),
            Errors(errs) => Err(GenericEndpointError(errs)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericError {
    pub code: String,
    pub path: Option<String>,
    pub user_message: String,
    pub timestamp: String,
}

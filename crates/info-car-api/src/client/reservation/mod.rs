use core::fmt;

use serde::{Deserialize, Serialize};

pub mod list;
pub mod new;
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
    pub status: PossibleStatuses, // TODO: Convert to a type
    pub timestamp: String,        // TODO: Convert to date type
    pub message: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum PossibleStatuses {
    Created,
    PlaceReserveD,
    Cancelled,
    SignupConfirmed,
    #[serde(other)]
    Unknown,
}
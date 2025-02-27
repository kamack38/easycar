use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub enum LicenseCategory {
    A,
    A1,
    A2,
    AM,
    #[default]
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

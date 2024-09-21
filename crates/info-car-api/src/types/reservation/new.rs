use std::num::NonZeroU32;

use crate::client::UserInfo;

use crate::types::LicenseCategory;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileIdType {
    /// Profil kierowcy zawodowego
    PKZ(String),
    /// Profil kandydata na kierowcę
    PKK(String),
}

impl Default for ProfileIdType {
    fn default() -> Self {
        ProfileIdType::PKK(String::default())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReservationCandidate {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub phone_number: String,
    pub pesel: String,
    pub category: LicenseCategory,
    #[serde(flatten)]
    pub driver_profile: ProfileIdType,
}

impl NewReservationCandidate {
    pub fn new_from_userinfo(
        userinfo: UserInfo,
        pesel: String,
        phone_number: String,
        driver_profile: ProfileIdType,
    ) -> Self {
        Self {
            category: LicenseCategory::default(),
            email: userinfo.email,
            firstname: userinfo.given_name,
            lastname: userinfo.family_name,
            pesel,
            phone_number,
            driver_profile,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExamId {
    PracticeId(String),
    TheoryId(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReservationExam {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    organization_unit_id: NonZeroU32,
    #[serde(flatten)]
    exam_id: ExamId,
}

impl NewReservationExam {
    pub fn new_theory_exam(organization_unit_id: NonZeroU32, exam_id: String) -> Self {
        NewReservationExam {
            organization_unit_id,
            exam_id: ExamId::TheoryId(exam_id),
        }
    }

    pub fn new_practice_exam(organization_unit_id: NonZeroU32, exam_id: String) -> Self {
        NewReservationExam {
            organization_unit_id,
            exam_id: ExamId::PracticeId(exam_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationLanguageAndOsk {
    language: String, // eg. POLISH
    osk_vehicle_reservation: Option<String>,
    sign_language: String, // eg. NONE
}

impl Default for ReservationLanguageAndOsk {
    fn default() -> Self {
        Self {
            language: String::from("POLISH"),
            osk_vehicle_reservation: None,
            sign_language: String::from("NONE"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReservation {
    candidate: NewReservationCandidate,
    exam: NewReservationExam,
    language_and_osk: ReservationLanguageAndOsk,
}

impl NewReservation {
    pub fn new(
        candidate: NewReservationCandidate,
        exam: NewReservationExam,
        language_and_osk: ReservationLanguageAndOsk,
    ) -> Self {
        Self {
            candidate,
            exam,
            language_and_osk,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NewReservationSuccess {
    pub id: String,
}

use crate::client::UserInfo;

use super::LicenseCategory;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReservationCandidate {
    pub category: LicenseCategory,
    pub email: String,
    pub firstname: String,
    pub lastname: String,
    pub pesel: String,
    pub phone_number: String,
    #[serde(flatten)]
    pub driver_profile: ProfileIdType,
}

impl ReservationCandidate {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ExamId {
    PracticeExam(String),
    TheoryExam(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservationExam {
    organization_unit_id: String,
    #[serde(flatten)]
    exam_id: ExamId,
}

impl ReservationExam {
    pub fn new_theory_exam(organization_unit_id: String, exam_id: String) -> Self {
        ReservationExam {
            organization_unit_id,
            exam_id: ExamId::TheoryExam(exam_id),
        }
    }

    pub fn new_practice_exam(organization_unit_id: String, exam_id: String) -> Self {
        ReservationExam {
            organization_unit_id,
            exam_id: ExamId::PracticeExam(exam_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewReservation {
    candidate: ReservationCandidate,
    exam: ReservationExam,
    language_and_osk: ReservationLanguageAndOsk,
}

impl NewReservation {
    pub fn new(
        candidate: ReservationCandidate,
        exam: ReservationExam,
        language_and_osk: ReservationLanguageAndOsk,
    ) -> Self {
        Self {
            candidate,
            exam,
            language_and_osk,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewReservationSuccess {
    pub id: String,
}

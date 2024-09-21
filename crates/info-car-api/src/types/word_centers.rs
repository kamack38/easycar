use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::num::NonZeroU32;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Province {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: NonZeroU32,
    pub name: String,
    pub latitude: String,
    pub longitude: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub zoom: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WordRescheduleEnabled {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// Word center ID
    pub organization_id: NonZeroU32,
    pub reschedule_enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: NonZeroU32,
    pub name: String,
    pub address: String,
    pub latitude: String,
    pub longitude: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub province_id: NonZeroU32,
    pub offline: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub code: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignLanguage {
    pub code: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WordCenters {
    pub provinces: Vec<Province>,
    pub words: Vec<Word>,
    pub languages_enums: Vec<Language>,
    pub sign_language_enums: Vec<SignLanguage>,
}

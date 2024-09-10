use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Province {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i32,
    pub name: String,
    pub latitude: String,
    pub longitude: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub zoom: i32,
}
#[derive(Deserialize, Debug, Clone)]
pub struct WordRescheduleEnabled {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i32,
    pub reschedule_enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i32,
    pub name: String,
    pub address: String,
    pub latitude: String,
    pub longitude: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub province_id: i32,
    pub offline: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LanguageEnum {
    pub code: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignLanguageEnum {
    pub code: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WordCenters {
    pub provinces: Vec<Province>,
    pub words: Vec<Word>,
    pub languages_enums: Vec<LanguageEnum>,
    pub sign_language_enums: Vec<SignLanguageEnum>,
}

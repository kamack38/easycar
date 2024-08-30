pub mod error;
pub mod exam_schedule;
pub mod reservations;
pub mod word_centers;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};
use serde::Deserialize;
use thiserror::Error;

use self::{
    error::{handle_response, JWTError},
    exam_schedule::ExamSchedule,
    reservations::{LicenseCategory, Reservations},
    word_centers::WordCenters,
};

pub struct InfoCarClient {
    client: Client,
    token: Option<String>,
}

#[derive(Error, Debug)]
pub enum GenericError {
    #[error("Bearer token not provided")]
    NoBearer,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    JWTError(#[from] JWTError),
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

impl InfoCarClient {
    pub fn new() -> Self {
        InfoCarClient {
            client: ClientBuilder::new().use_rustls_tls().build().unwrap(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    async fn get_csrf_token(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client.get(url).send().await?;

        let fragment = Html::parse_fragment(&response.text().await?);
        let csrf_selector = Selector::parse("input[type=\"hidden\"][name=\"_csrf\"]")?;

        let csrf_element = fragment.select(&csrf_selector).next().unwrap();
        Ok(csrf_element.value().attr("value").unwrap().to_owned())
    }

    pub async fn login(&mut self, username: &str, password: &str) {
        let csrf_token = self
            .get_csrf_token("https://info-car.pl/oauth2/login")
            .await
            .unwrap();

        println!("csrf_token: {csrf_token:#?}");
        let form_params = [
            ("username", username),
            ("_csrf", &csrf_token),
            ("password", password),
            ("_csrf", &csrf_token),
        ];

        let response = self
            .client
            .post("https://info-car.pl/oauth2/login")
            .form(&form_params)
            .send()
            .await
            .unwrap();
        println!("{:#?}", response);
    }

    pub async fn user_info(&self) -> Result<UserInfo, GenericError> {
        Ok(self
            .client
            .get("https://info-car.pl/oauth2/userinfo")
            .bearer_auth(self.token.as_ref().ok_or(GenericError::NoBearer)?)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn my_reservations(&self) -> Result<Reservations, GenericError> {
        let response = self
            .client
            .get("https://info-car.pl/api/word/reservations")
            .bearer_auth(self.token.as_ref().ok_or(GenericError::NoBearer)?)
            .send()
            .await?;
        Ok(handle_response(response)?.json().await?)
    }

    pub async fn word_centers(&self) -> Result<WordCenters, GenericError> {
        let response = self
            .client
            .get("https://info-car.pl/api/word/word-centers")
            .bearer_auth(self.token.as_ref().ok_or(GenericError::NoBearer)?)
            .send()
            .await?;
        Ok(handle_response(response)?.json().await?)
    }

    pub async fn exam_schedule(
        &self,
        word_id: String,
        end_date: DateTime<Utc>,
        start_date: DateTime<Utc>,
        category: LicenseCategory,
    ) -> Result<ExamSchedule, GenericError> {
        let mut map = HashMap::<&str, String>::new();
        map.insert("category", category.to_string());
        map.insert("endDate", end_date.to_string());
        map.insert("startDate", start_date.to_string());
        map.insert("wordId", word_id);

        let response = self
            .client
            .put("https://info-car.pl/api/word/word-centers/exam-schedule")
            .bearer_auth(self.token.as_ref().ok_or(GenericError::NoBearer)?)
            .json(&map)
            .send()
            .await?;
        Ok(handle_response(response)?.json().await?)
    }
}

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
use url;

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

#[derive(Error, Debug)]
pub enum LoginError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
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

#[derive(Error, Debug)]
pub enum RefreshTokenError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("No fragment was provided in the response URL")]
    NoFragmentProvided,
    #[error(transparent)]
    UrlFragmentParseError(#[from] serde_urlencoded::de::Error),
    #[error("The session_id was not set")]
    SessionIdNotSet,
}

#[derive(Error, Debug)]
pub enum CsrfTokenError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SelectorParseError(#[from] scraper::error::SelectorErrorKind<'static>),
    #[error("The element containing the CSRF token could not be found")]
    TokenNotFound,
    #[error("The CSRF token value could not be found on the input element")]
    TokenValueNotFound,
}

impl InfoCarClient {
    pub fn new() -> Self {
        InfoCarClient {
            client: ClientBuilder::new()
                .use_rustls_tls()
                .cookie_store(true)
                .build()
                .unwrap(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub async fn refresh_token(&mut self) -> Result<(), RefreshTokenError> {
        let response = self
            .client
            .get("https://info-car.pl/oauth2/authorize")
            .query(&[
                ("response_type", "id_token token"),
                ("client_id", "client"),
                (
                    "redirect_uri",
                    "https://info-car.pl/new/assets/refresh.html",
                ),
                ("scope", "openid profile email resource.read"),
                ("prompt", "none"),
            ])
            .send()
            .await?;

        let parsed_response: HashMap<String, String> = serde_urlencoded::from_str(
            response
                .url()
                .fragment()
                .ok_or(RefreshTokenError::NoFragmentProvided)?,
        )?;

        println!("{:#?}", parsed_response);

        let new_token = parsed_response.get("access_token").unwrap();

        self.set_token(new_token.to_owned());

        Ok(())
    }

    async fn get_csrf_token(&self, url: &str) -> Result<String, CsrfTokenError> {
        let response = self.client.get(url).send().await?;

        let fragment = Html::parse_fragment(&response.text().await?);
        let csrf_selector = Selector::parse("input[type=\"hidden\"][name=\"_csrf\"]")?;

        let csrf_element = fragment
            .select(&csrf_selector)
            .next()
            .ok_or(CsrfTokenError::TokenNotFound)?;
        Ok(csrf_element
            .value()
            .attr("value")
            .ok_or(CsrfTokenError::TokenValueNotFound)?
            .to_owned())
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), LoginError> {
        let csrf_token = self
            .get_csrf_token("https://info-car.pl/oauth2/login")
            .await
            .unwrap();

        let form_params = [
            ("username", username),
            ("_csrf", &csrf_token),
            ("password", password),
            ("_csrf", &csrf_token),
        ];

        self.client
            .post("https://info-car.pl/oauth2/login")
            .form(&form_params)
            .send()
            .await?;

        self.refresh_token().await.unwrap();

        Ok(())
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

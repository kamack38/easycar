pub mod exam_schedule;
pub mod reservations;
pub mod word_centers;

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};

use self::{
    exam_schedule::ExamSchedule,
    reservations::{LicenseCategory, Reservations},
    word_centers::WordCenters,
};
use crate::error::{
    handle_response, CsrfTokenError, GenericClientError, LoginError, RefreshTokenError,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

pub struct InfoCarClient {
    client: Client,
    token: Option<String>,
    pub token_expire_date: Option<DateTime<Utc>>,
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
            token_expire_date: None,
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

        let new_token = parsed_response
            .get("access_token")
            .ok_or(RefreshTokenError::AccessTokenNotProvided)?;

        let expire_time_unix: i64 = parsed_response
            .get("expires_in")
            .ok_or(RefreshTokenError::ExpireTimeNotProvided)?
            .parse()
            .or(Err(RefreshTokenError::ExpireTimeParseError))?;

        self.token_expire_date = Some(
            Utc::now()
                .checked_add_signed(Duration::seconds(expire_time_unix))
                .unwrap(),
        );

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

        self.refresh_token().await?;

        Ok(())
    }

    pub async fn user_info(&self) -> Result<UserInfo, GenericClientError> {
        Ok(self
            .client
            .get("https://info-car.pl/oauth2/userinfo")
            .bearer_auth(self.token.as_ref().ok_or(GenericClientError::NoBearer)?)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn my_reservations(&self) -> Result<Reservations, GenericClientError> {
        let response = self
            .client
            .get("https://info-car.pl/api/word/reservations")
            .bearer_auth(self.token.as_ref().ok_or(GenericClientError::NoBearer)?)
            .send()
            .await?;
        Ok(handle_response(response)?.json().await?)
    }

    pub async fn word_centers(&self) -> Result<WordCenters, GenericClientError> {
        let response = self
            .client
            .get("https://info-car.pl/api/word/word-centers")
            .bearer_auth(self.token.as_ref().ok_or(GenericClientError::NoBearer)?)
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
    ) -> Result<ExamSchedule, GenericClientError> {
        let mut map = HashMap::<&str, String>::new();
        map.insert("category", category.to_string());
        map.insert("endDate", end_date.to_string());
        map.insert("startDate", start_date.to_string());
        map.insert("wordId", word_id);

        let response = self
            .client
            .put("https://info-car.pl/api/word/word-centers/exam-schedule")
            .bearer_auth(self.token.as_ref().ok_or(GenericClientError::NoBearer)?)
            .json(&map)
            .send()
            .await?;
        Ok(handle_response(response)?.json().await?)
    }
}

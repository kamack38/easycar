use chrono::{DateTime, Days, Utc};
use info_car_api::{
    client::{
        exam_schedule::Exam,
        reservation::{list::ReservationList, LicenseCategory},
        Client,
    },
    error::{GenericClientError, LoginError},
    utils::find_n_practice_exams,
};
use thiserror::Error;

#[derive(Clone)]
pub struct UserData {
    pub username: String,
    pub password: String,
    pub preferred_osk: String,
}

impl UserData {
    pub fn new(username: String, password: String, preferred_osk: String) -> Self {
        UserData {
            username,
            password,
            preferred_osk,
        }
    }
}

#[derive(Error, Debug)]
pub enum GetExamsError {
    #[error(transparent)]
    GenericClientError(#[from] GenericClientError),
    #[error("No exams found")]
    NoExamsError,
}

pub struct InfoCarClient {
    client: Client,
    user_data: UserData,
}

impl InfoCarClient {
    pub fn new(user_data: UserData) -> Self {
        Self {
            client: Client::new(),
            user_data,
        }
    }

    pub async fn login(&mut self) -> Result<DateTime<Utc>, LoginError> {
        self.client
            .login(&self.user_data.username, &self.user_data.password)
            .await?;
        Ok(self
            .client
            .token_expire_date
            .expect("Expire date is not available"))
    }

    pub async fn refresh_token(&mut self) -> Result<DateTime<Utc>, LoginError> {
        if self.client.refresh_token().await.is_err() {
            self.login().await
        } else {
            Ok(self
                .client
                .token_expire_date
                .expect("Expire date is not set"))
        }
    }

    pub async fn get_nearest_exams(&mut self, number: usize) -> Result<Vec<Exam>, GetExamsError> {
        let schedule = self
            .client
            .exam_schedule(
                self.user_data.preferred_osk.clone(),
                Utc::now(),
                Utc::now().checked_add_days(Days::new(31)).unwrap(),
                LicenseCategory::B,
            )
            .await?;

        find_n_practice_exams(schedule, number).ok_or(GetExamsError::NoExamsError)
    }

    pub async fn get_reservations(&mut self) -> Result<ReservationList, GenericClientError> {
        self.client.my_reservations().await
    }
}

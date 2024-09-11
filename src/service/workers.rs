use crate::logger::BotLogger;

use crate::utils::date_from_string;
use chrono::{DateTime, Days, Duration as ChronoDuration, Utc};
use info_car_api::{
    client::{
        exam_schedule::ExamList,
        reservation::{list::ReservationList, LicenseCategory},
        Client,
    },
    error::{GenericClientError, LoginError},
    utils::find_n_practice_exams,
};
use teloxide::prelude::*;
use thiserror::Error;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::{sleep, Duration as TokioDuration},
};

#[derive(Clone)]
pub struct UserData {
    pub username: String,
    pub password: String,
    pub preferred_osk: String,
    pub chat_id: ChatId,
}

impl UserData {
    pub fn new(username: String, password: String, preferred_osk: String, chat_id: String) -> Self {
        UserData {
            username,
            password,
            preferred_osk,
            chat_id: ChatId(chat_id.parse().expect("Could not parse the given chat id")),
        }
    }
}

#[derive(Debug)]
pub enum ClientMessage {
    GetSchedule,
    GetNewExams,
    RefreshToken,
    GetReservations,
}

#[derive(Debug)]
pub enum BotMessage {
    SendSchedule(Option<ExamList>),
    SendReservations(Option<ReservationList>),
    // SendCurrentExam(Exam),
}

pub async fn refresh_token_worker(
    tc: mpsc::Sender<ClientMessage>,
    mut rc: mpsc::Receiver<DateTime<Utc>>,
) {
    loop {
        let expire_date = rc.recv().await.expect("Client transmitter not active");
        println!("Got token expire date: {expire_date}");
        let duration = expire_date - Utc::now() - ChronoDuration::minutes(5);
        println!("Token expires in: {}", duration.num_seconds());
        sleep(TokioDuration::from_secs(
            duration.num_seconds().try_into().unwrap(),
        ))
        .await;
        println!("Sending refresh token signal...");
        tc.send(ClientMessage::RefreshToken).await.unwrap();
    }
}

pub async fn scheduler(tc: Sender<ClientMessage>) {
    loop {
        tc.send(ClientMessage::GetNewExams)
            .await
            .expect("Client receiver does not exist");
        sleep(TokioDuration::from_secs(10)).await;
    }
}

pub struct InfoCarService {
    logger: BotLogger,
    client: Client,
    last_exam_id: String,
    t_to_refresher: Sender<DateTime<Utc>>,
    t_to_bot: Sender<BotMessage>,
    r_client: Receiver<ClientMessage>,
    user_data: UserData,
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    ClientLoginError(#[from] LoginError),
    #[error(transparent)]
    GenericClientError(#[from] GenericClientError),
    #[error("No exams found")]
    NoExamsError,
}

impl InfoCarService {
    pub fn new(
        user_data: UserData,
        teloxide_token: String,
        r_client: Receiver<ClientMessage>,
        t_to_refresher: Sender<DateTime<Utc>>,
        t_to_bot: Sender<BotMessage>,
    ) -> Self {
        Self {
            logger: BotLogger::new(teloxide_token, user_data.chat_id),
            client: Client::new(),
            last_exam_id: String::from(""),
            r_client,
            t_to_refresher,
            t_to_bot,
            user_data,
        }
    }

    pub async fn start(&mut self) -> Result<(), LoginError> {
        self.client
            .login(&self.user_data.username, &self.user_data.password)
            .await?;
        self.t_to_refresher
            .send(
                self.client
                    .token_expire_date
                    .expect("Expire date is not available"),
            )
            .await
            .unwrap();

        loop {
            if let Err(err) = self.handle_message().await {
                println!("Got an error: {err}");
            };
        }
    }

    async fn handle_message(&mut self) -> Result<(), ServiceError> {
        match self.r_client.recv().await.expect("No sender exists...") {
            ClientMessage::RefreshToken => {
                self.logger
                    .log("Got refresh token signal. Refreshing...")
                    .await;
                if let Err(err) = self.client.refresh_token().await {
                    self.logger
                        .log(&format!("Got: {err}. Logining again..."))
                        .await;
                    self.client
                        .login(&self.user_data.username, &self.user_data.password)
                        .await?;
                };
                self.t_to_refresher
                    .send(
                        self.client
                            .token_expire_date
                            .expect("Expire date is not set"),
                    )
                    .await
                    .unwrap();
            }
            ClientMessage::GetNewExams => {
                let schedule = self
                    .client
                    .exam_schedule(
                        self.user_data.preferred_osk.clone(),
                        Utc::now(),
                        Utc::now().checked_add_days(Days::new(31)).unwrap(),
                        LicenseCategory::B,
                    )
                    .await?;

                let closest_exam = find_n_practice_exams(schedule, 1)
                    .ok_or(ServiceError::NoExamsError)?
                    .pop()
                    .unwrap();

                if closest_exam.id == self.last_exam_id {
                    println!("No change...");
                    return Ok(());
                }

                self.last_exam_id = closest_exam.id.clone();

                let duration = date_from_string(&closest_exam.date)
                    .signed_duration_since(Utc::now())
                    .num_days();
                let exam_message = format!(
                    "New exam is available! The next exam date is {} (in {} days)",
                    closest_exam.date, duration
                );
                self.logger.log(&exam_message).await;
            }
            ClientMessage::GetSchedule => {
                let exam_schedule = self
                    .client
                    .exam_schedule(
                        self.user_data.preferred_osk.clone(),
                        Utc::now(),
                        Utc::now().checked_add_days(Days::new(31)).unwrap(),
                        LicenseCategory::B,
                    )
                    .await?;

                let flattened_schedule = find_n_practice_exams(exam_schedule, 5);
                self.t_to_bot
                    .send(BotMessage::SendSchedule(
                        flattened_schedule.map(|v| v.into()),
                    ))
                    .await
                    .expect("Bot receiver does not exist");
            }
            ClientMessage::GetReservations => {
                let reservations = self.client.my_reservations().await?;

                println!("{reservations:?}");

                self.t_to_bot
                    .send(BotMessage::SendReservations(Some(reservations)))
                    .await
                    .expect("Bot receiver does not exist");
            }
        };
        Ok(())
    }
}

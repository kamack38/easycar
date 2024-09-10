use crate::logger::BotLogger;

use crate::utils::date_from_string;
use chrono::{DateTime, Days, Duration as ChronoDuration, Utc};
use info_car_api::{
    client::{exam_schedule::ExamList, reservation::LicenseCategory, Client},
    error::LoginError,
    utils::find_n_practice_exams,
};
use teloxide::prelude::*;
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
}

#[derive(Debug)]
pub enum BotMessage {
    SendSchedule(ExamList),
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

pub async fn info_car_worker(
    user_data: UserData,
    teloxide_token: String,
    mut r_client: Receiver<ClientMessage>,
    t_to_refresher: Sender<DateTime<Utc>>,
    t_to_bot: Sender<BotMessage>,
) -> Result<(), LoginError> {
    let mut last_id: String = "".to_owned();

    let logger = BotLogger::new(teloxide_token, user_data.chat_id);

    let mut client = Client::new();
    client
        .login(&user_data.username, &user_data.password)
        .await?;
    t_to_refresher
        .send(
            client
                .token_expire_date
                .expect("Expire date is not available"),
        )
        .await
        .unwrap();

    loop {
        match r_client.recv().await.expect("No sender exists...") {
            // TODO: Refactor
            ClientMessage::RefreshToken => {
                logger.log("Got refresh token signal. Refreshing...").await;
                if let Err(err) = client.refresh_token().await {
                    logger.log(&format!("Got: {err}. Logining again...")).await;
                    if let Err(login_err) =
                        client.login(&user_data.username, &user_data.password).await
                    {
                        let cos = login_err.to_string();
                        logger
                            .log(&format!("While logining got an error: {}", cos))
                            .await;
                    };
                };
                t_to_refresher
                    .send(client.token_expire_date.expect("Expire date is not set"))
                    .await
                    .unwrap();
            }
            ClientMessage::GetNewExams => {
                let response = client
                    .exam_schedule(
                        user_data.preferred_osk.clone(),
                        Utc::now(),
                        Utc::now().checked_add_days(Days::new(31)).unwrap(),
                        LicenseCategory::B,
                    )
                    .await;

                match response {
                    Ok(schedule) => match find_n_practice_exams(schedule, 1) {
                        Some(exams) => {
                            if exams[0].id != last_id {
                                last_id = exams[0].id.clone();
                                println!("{:?}", exams[0]);

                                let duration = date_from_string(&exams[0].date)
                                    .signed_duration_since(Utc::now())
                                    .num_days();
                                let exam_message = format!(
                                    "New exam is available! The next exam date is {} (in {} days)",
                                    exams[0].date, duration
                                );
                                logger.log(&format!("{exam_message}")).await;
                            } else {
                                println!("No change...")
                            }
                        }
                        None => println!("No exams found"),
                    },
                    Err(err) => {
                        logger.log(&format!("Got an error! Error: {err}")).await;
                    }
                };
            }
            ClientMessage::GetSchedule => {
                println!("Getting schedule");
                let response = client
                    .exam_schedule(
                        user_data.preferred_osk.clone(),
                        Utc::now(),
                        Utc::now().checked_add_days(Days::new(31)).unwrap(),
                        LicenseCategory::B,
                    )
                    .await;

                match response {
                    Ok(schedule) => {
                        let flattened_schedule = find_n_practice_exams(schedule, 5);
                        t_to_bot
                            .send(BotMessage::SendSchedule(
                                flattened_schedule.expect("No exams found").into(),
                            ))
                            .await
                            .expect("Bot receiver does not exist");
                    }
                    Err(err) => {
                        logger.log(&format!("Got an error! Error: {err}")).await;
                    }
                };
            }
        }
    }
}

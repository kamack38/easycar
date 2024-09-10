use crate::logger::BotLogger;
use std::sync::Arc;

use crate::utils::{date_from_string, readable_time_delta};
use chrono::{DateTime, Days, Duration as ChronoDuration, Utc};
use info_car_api::{
    client::{exam_schedule::ExamSchedule, reservation::LicenseCategory, Client},
    error::LoginError,
    utils::find_first_non_empty_practice_exam,
};
use teloxide::prelude::*;
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    time::{sleep, Duration as TokioDuration},
};

pub struct EasyCarService {
    pub bot: Bot,
    pub teloxide_token: String,
    pub user_data: UserData,
}

impl EasyCarService {
    pub fn new(teloxide_token: String, user_data: UserData) -> Self {
        Self {
            bot: Bot::new(&teloxide_token),
            user_data,
            teloxide_token,
        }
    }
    pub async fn start(&self) -> Result<(), ()> {
        // Get a start date for the /uptime command
        let start_date = Utc::now();

        // Create a client channel (Transmit to Client and Receive from all)
        let (tc, ra) = mpsc::channel::<ClientMessage>(32);

        // Create a token refresher channel (Transmit to Refresher and Receive from Client)
        let (tr, rc) = mpsc::channel::<DateTime<Utc>>(8);

        // Create a token bot channel (Transmit to Bot and Receive from Client)
        let (tb, rbc) = mpsc::channel::<BotMessage>(10);

        tokio::spawn(info_car_worker(
            self.user_data.clone(),
            self.teloxide_token.clone(),
            ra,
            tr,
            tb,
        ));
        tokio::spawn(scheduler(tc.clone()));
        tokio::spawn(refresh_token_worker(tc.clone(), rc));

        let rx_from_thread = Arc::new(Mutex::new(rbc));

        teloxide::repl(self.bot.clone(), move |message: Message, bot: Bot| {
            let tc = tc.clone();
            let rbc: Arc<Mutex<Receiver<BotMessage>>> = Arc::clone(&rx_from_thread);
            async move {
                if let Some(text) = message.text() {
                    match text {
                        "/start" => {
                            let rem_keyboard = KeyboardRemove::default();
                            bot.send_message(message.chat.id, text)
                                .reply_markup(rem_keyboard)
                                .await?;
                            // Create a custom keyboard
                            let keyboard = KeyboardMarkup::default()
                                .append_row(vec![
                                    KeyboardButton::new("Current Exam"),
                                    KeyboardButton::new("Exams"),
                                ])
                                .append_row(vec![KeyboardButton::new("Enroll")]);

                            // Send the message with the keyboard
                            bot.send_message(message.chat.id, "Choose an option:")
                                .reply_markup(keyboard)
                                .await?;
                        }
                        "Current Exam" => {
                            // tc.send(ClientMessage::GetAvailableExams).await.unwrap();
                            bot.send_message(message.chat.id, "The current exam is: ")
                                .await?;
                        }
                        "Exams" => {
                            tc.send(ClientMessage::GetSchedule).await.unwrap();
                            let BotMessage::SendSchedule(schedule) = rbc
                                .lock()
                                .await
                                .recv()
                                .await
                                .expect("Client->Bot channel closed");

                            println!("{schedule:?}");
                            bot.send_message(
                                message.chat.id,
                                format!(
                                    "The available exams are: {:?}",
                                    schedule.schedule.scheduled_days[0]
                                ),
                            )
                            .await?;
                        }
                        "Enroll" => {
                            bot.send_message(message.chat.id, "Do you want to enroll to exam")
                                .await?;
                        }
                        "/uptime" => {
                            bot.send_message(
                                message.chat.id,
                                format!(
                                    "The uptime is: {}",
                                    readable_time_delta(Utc::now() - start_date)
                                ),
                            )
                            .await?;
                        }
                        _ => {
                            bot.send_message(message.chat.id, "Unknown command").await?;
                        }
                    }
                }
                Ok(())
            }
        })
        .await;

        Ok(())
    }
}

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
enum ClientMessage {
    GetSchedule,
    GetNewExams,
    RefreshToken,
}

#[derive(Debug)]
enum BotMessage {
    SendSchedule(ExamSchedule),
    // SendCurrentExam(Exam),
}

async fn refresh_token_worker(
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

async fn scheduler(tc: Sender<ClientMessage>) {
    loop {
        tc.send(ClientMessage::GetNewExams)
            .await
            .expect("Client receiver does not exist");
        sleep(TokioDuration::from_secs(10)).await;
    }
}

async fn info_car_worker(
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
                    Ok(schedule) => {
                        if let Some(exam) = find_first_non_empty_practice_exam(&schedule) {
                            if exam[0].id != last_id {
                                last_id = exam[0].id.clone();
                                println!("{:?}", exam);

                                let duration = date_from_string(&exam[0].date)
                                    .signed_duration_since(Utc::now())
                                    .num_days();
                                let exam_message = format!(
                                    "New exam is available! The next exam date is {} (in {} days)",
                                    exam[0].date, duration
                                );
                                logger.log(&format!("{exam_message}")).await;
                                // println!("{exam_message}");
                            } else {
                                println!("No change...")
                            }
                        }
                    }
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
                        t_to_bot
                            .send(BotMessage::SendSchedule(schedule))
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

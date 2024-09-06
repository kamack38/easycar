mod utils;

use chrono::{DateTime, Days, Duration as ChronoDuration, Utc};
use dotenvy::dotenv;
use info_car_api::{
    client::{reservation::LicenseCategory, Client},
    error::LoginError,
    utils::find_first_non_empty_practice_exam,
};
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::{sleep, Duration as TokioDuration},
};
use utils::{date_from_string, readable_time_delta};

struct UserData {
    pub username: String,
    pub password: String,
    pub preffered_osk: String,
    pub chat_id: ChatId,
}

impl UserData {
    pub fn new(username: String, password: String, preffered_osk: String, chat_id: ChatId) -> Self {
        UserData {
            username,
            password,
            preffered_osk,
            chat_id,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get a start date for the /uptime command
    let start_date = Utc::now();
    dotenv().expect(".env file not found");

    // Get UserData
    let username = dotenvy::var("USERNAME")?;
    let password = dotenvy::var("PASSWORD")?;
    let osk_id = "3";

    let chat_id = ChatId(dotenvy::var("TELEGRAM_CHAT_ID")?.parse().unwrap());
    let user_data = UserData::new(username, password, osk_id.to_string(), chat_id);
    // let pesel = dotenvy::var("PESEL")?;
    // let phone_number = dotenvy::var("PHONE_NUMBER")?;
    // let pkk = dotenvy::var("PKK")?;

    let bot = Bot::from_env();

    // Create a client channel (Transmit to Client and Recieve from all)
    let (tc, ra) = mpsc::channel::<ClientMessage>(32);

    // Create a token refresher channel (Transmit to Refresher and Recieve from Client)
    let (tr, rc) = mpsc::channel::<DateTime<Utc>>(8);

    // Create a token bot channel (Transmit to Bot and Recieve from Client)
    // let (tb, mut rc) = mpsc::channel::<bool>(10);

    tokio::spawn(info_car_worker(user_data, ra, tr));
    tokio::spawn(scheduler(tc.clone()));
    tokio::spawn(refresh_token_worker(tc, rc));

    sleep(TokioDuration::from_secs(30)).await;

    teloxide::repl(bot, move |message: Message, bot: Bot| async move {
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
                    bot.send_message(message.chat.id, "The current exam is")
                        .await?;
                }
                "Exams" => {
                    bot.send_message(message.chat.id, "The available exams are: ")
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
    })
    .await;

    Ok(())
}

struct ErrorLogger {
    chat_id: ChatId,
    bot: Bot,
}

impl ErrorLogger {
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            chat_id,
            bot: Bot::from_env(),
        }
    }

    pub async fn log(&self, error_message: &str) {
        println!("{error_message}");
        self.bot
            .send_message(self.chat_id, error_message)
            .await
            .unwrap();
    }
}

async fn refresh_token_worker(
    tc: mpsc::Sender<ClientMessage>,
    mut rc: mpsc::Receiver<DateTime<Utc>>,
) {
    loop {
        let expire_date = rc.recv().await.unwrap();
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
            .expect("Client reciever does not exist");
        sleep(TokioDuration::from_secs(10)).await;
    }
}

enum ClientMessage {
    GetAvailableExams,
    GetNewExams,
    RefreshToken,
}

async fn info_car_worker(
    user_data: UserData,
    mut r_client: Receiver<ClientMessage>,
    t_to_refresher: Sender<DateTime<Utc>>,
) -> Result<(), LoginError> {
    let mut last_id: String = "".to_owned();

    let logger = ErrorLogger::new(user_data.chat_id);

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
                        user_data.preffered_osk.clone(),
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
            ClientMessage::GetAvailableExams => todo!(),
        }
    }
}

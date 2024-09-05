use chrono::{DateTime, Days, Duration as ChronoDuration, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use dotenvy::dotenv;
use info_car_api::{
    client::{reservation::LicenseCategory, Client},
    utils::find_first_non_empty_practice_exam,
};
use teloxide::prelude::*;
use tokio::{
    sync::mpsc,
    time::{sleep, Duration as TokioDuration},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect(".env file not found");

    let username = dotenvy::var("USERNAME")?;
    let password = dotenvy::var("PASSWORD")?;
    let chat_id = dotenvy::var("TELEGRAM_CHAT_ID")?;
    // let pesel = dotenvy::var("PESEL")?;
    // let phone_number = dotenvy::var("PHONE_NUMBER")?;
    // let pkk = dotenvy::var("PKK")?;

    let bot = Bot::from_env();
    let logger = ErrorLogger::new(chat_id.clone(), &bot);
    let osk_id = "3";

    let mut client = Client::new();

    // Create a channel for sending the token expire date
    let (tx, rx) = mpsc::channel::<DateTime<Utc>>(10);

    // Create a channel for seding when to refresh the token
    let (ty, mut ry) = mpsc::channel::<bool>(10);

    client.login(&username, &password).await?;
    tx.send(
        client
            .token_expire_date
            .expect("Expire date is not available"),
    )
    .await
    .unwrap();

    tokio::spawn(refresh_token_worker(ty, rx));

    let mut last_id: String = "".to_owned();
    loop {
        // Handle token refreshing
        if ry.try_recv().is_ok() {
            logger.log("Got refresh token signal. Refreshing...").await;
            if let Err(err) = client.refresh_token().await {
                logger.log(&format!("Got: {err}. Logining again...")).await;
                if let Err(login_err) = client.login(&username, &password).await {
                    logger
                        .log(&format!("While logining got an error: {login_err}"))
                        .await;
                };
            };
            tx.send(client.token_expire_date.expect("Expire date is not set"))
                .await
                .unwrap();
        }

        let response = client
            .exam_schedule(
                osk_id.into(),
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

                        if let Err(err) = bot.send_message(chat_id.clone(), exam_message).await {
                            println!("{err:?}");
                        }
                    } else {
                        println!("No change...")
                    }
                }
            }
            Err(err) => {
                logger.log(&format!("Got an error! Error: {err}")).await;
            }
        };

        sleep(TokioDuration::from_secs(10)).await;
    }
}

struct ErrorLogger<'a> {
    chat_id: ChatId,
    bot: &'a Bot,
}

impl<'a> ErrorLogger<'a> {
    pub fn new(chat_id: String, bot: &'a Bot) -> Self {
        Self {
            chat_id: ChatId(chat_id.parse().unwrap()),
            bot,
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

fn date_from_string(timestamp: &str) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse timestamp");
    let datetime_cest: DateTime<chrono_tz::Tz> =
        Warsaw.from_local_datetime(&naive_datetime).unwrap();
    datetime_cest.with_timezone(&Utc)
}

async fn refresh_token_worker(ty: mpsc::Sender<bool>, mut rx: mpsc::Receiver<DateTime<Utc>>) {
    loop {
        let expire_date = rx.recv().await.unwrap();
        println!("Got token expire date: {expire_date}");
        let duration = expire_date - Utc::now() - ChronoDuration::minutes(5);
        println!("Token expires in: {}", duration.num_seconds());
        sleep(TokioDuration::from_secs(
            duration.num_seconds().try_into().unwrap(),
        ))
        .await;
        println!("Sending refresh token signal...");
        ty.send(true).await.unwrap();
    }
}

async fn bot_worker(bot: Bot) {
    teloxide::repl(bot, |message: Message, bot: Bot| async move {
        if let Some(text) = message.text() {
            // Reply to any incoming message
            bot.send_message(message.chat.id, format!("You said: {}", text))
                .await?;
        }
        Ok(())
    })
    .await;
}

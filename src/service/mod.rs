pub mod workers;

use std::sync::Arc;

use crate::utils::readable_time_delta;
use chrono::{DateTime, Utc};
use teloxide::prelude::*;
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use workers::*;

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

        let mut info_car_service = InfoCarService::new(
            self.user_data.clone(),
            self.teloxide_token.clone(),
            ra,
            tr.clone(),
            tb.clone(),
        );

        tokio::spawn(async move { info_car_service.start().await });
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

                            match schedule {
                                Some(schedule) => {
                                    bot.send_message(
                                        message.chat.id,
                                        format!("The available exams are:\n{}", schedule),
                                    )
                                    .await?;
                                }
                                None => {
                                    bot.send_message(message.chat.id, "No exams found").await?;
                                }
                            }
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

pub mod workers;

use std::ops::Not;
use std::sync::Arc;

use crate::utils::readable_time_delta;
use chrono::{DateTime, Utc};
use teloxide::payloads::SetChatMenuButtonSetters;
use teloxide::types::{MenuButton, MessageId};
use teloxide::RequestError;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use tokio::time::{sleep, Duration};
use workers::*;

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Display this text.
    #[command(aliases = ["h", "?"])]
    Help,
    /// Get the bot uptime
    #[command()]
    Uptime,
    /// Get exam dates
    #[command()]
    Exams,
    /// Get current exam
    #[command()]
    Exam,
    /// Show all reservations
    #[command()]
    Reservations,
    /// Enroll to the exam
    #[command()]
    Enroll(String),
}

pub struct EasyCarService {
    pub bot: Bot,
    pub teloxide_token: String,
    pub user_data: UserData,
}

async fn waiting_spinner(
    rbc: &Arc<Mutex<Receiver<BotMessage>>>,
    bot: &Bot,
    chat_id: ChatId,
) -> Result<MessageId, RequestError> {
    // Spinner logic
    let spinner = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut iter = spinner.iter().cycle();

    // Send an initial message and get its MessageId to edit later
    let sent_message = bot.send_message(chat_id, *iter.next().unwrap()).await?;

    // Simulate the spinner by continuously editing the message
    for frame in iter {
        bot.edit_message_text(chat_id, sent_message.id, *frame)
            .await?;
        sleep(Duration::from_millis(100)).await; // Adjust the speed here
        if rbc.lock().await.is_empty().not() {
            break;
        }
    }
    Ok(sent_message.id)
}

impl EasyCarService {
    pub fn new(teloxide_token: String, user_data: UserData) -> Self {
        Self {
            bot: Bot::new(&teloxide_token),
            user_data,
            teloxide_token,
        }
    }

    pub async fn start(self) -> Result<(), ()> {
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

        self.bot
            .set_my_commands(Command::bot_commands())
            .await
            .unwrap();
        self.bot
            .set_chat_menu_button()
            .menu_button(MenuButton::Commands)
            .await
            .unwrap();

        Command::repl(self.bot, move |bot: Bot, msg: Message, cmd: Command| {
            let tc = tc.clone();
            let rbc: Arc<Mutex<Receiver<BotMessage>>> = Arc::clone(&rx_from_thread);
            async move {
                match cmd {
                    Command::Help => {
                        bot.send_message(msg.chat.id, Command::descriptions().to_string())
                            .await?;
                    }
                    Command::Exam => {
                        // tc.send(ClientMessage::GetAvailableExams).await.unwrap();
                        bot.send_message(msg.chat.id, "The current exam is: ")
                            .await?;
                    }
                    Command::Exams => {
                        tc.send(ClientMessage::GetSchedule).await.unwrap();
                        let message_id = waiting_spinner(&rbc, &bot, msg.chat.id).await?;
                        if let BotMessage::SendSchedule(schedule) = rbc
                            .lock()
                            .await
                            .recv()
                            .await
                            .expect("Client->Bot channel closed")
                        {
                            match schedule {
                                Some(schedule) => {
                                    bot.edit_message_text(
                                        msg.chat.id,
                                        message_id,
                                        format!("The available exams are:\n{}", schedule),
                                    )
                                    .await?;
                                }
                                None => {
                                    bot.edit_message_text(
                                        msg.chat.id,
                                        message_id,
                                        "❌ No exams found",
                                    )
                                    .await?;
                                }
                            }
                        }
                    }
                    Command::Reservations => {
                        tc.send(ClientMessage::GetReservations).await.unwrap();
                        let message_id = waiting_spinner(&rbc, &bot, msg.chat.id).await?;
                        if let BotMessage::SendReservations(reservations) = rbc
                            .lock()
                            .await
                            .recv()
                            .await
                            .expect("Client->Bot channel closed")
                        {
                            match reservations {
                                Some(reservations) => {
                                    let text: String = reservations
                                        .items
                                        .iter()
                                        .map(|v| {
                                            format!(
                                                "• At {} in {} ({})",
                                                v.exam
                                                    .practice
                                                    .as_ref()
                                                    .or_else(|| v.exam.theory.as_ref())
                                                    .unwrap()
                                                    .date,
                                                v.exam.organization_unit_name,
                                                v.status.status
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n\n");
                                    bot.edit_message_text(msg.chat.id, message_id, text).await?;
                                }
                                None => {
                                    bot.edit_message_text(
                                        msg.chat.id,
                                        message_id,
                                        "❌ No reservations found",
                                    )
                                    .await?;
                                }
                            }
                        }
                    }
                    Command::Enroll(exam_id) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("Do you want to enroll to exam {exam_id}"),
                        )
                        .await?;
                    }
                    Command::Uptime => {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                "The uptime is: {}",
                                readable_time_delta(Utc::now() - start_date)
                            ),
                        )
                        .await?;
                    }
                };
                Ok(())
            }
        })
        .await;

        Ok(())
    }
}

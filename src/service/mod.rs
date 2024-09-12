pub mod workers;

use std::sync::Arc;

use crate::client::{GetExamsError, InfoCarClient, UserData};
use crate::utils::readable_time_delta;
use chrono::{DateTime, Utc};
use info_car_api::error::GenericClientError;
use teloxide::payloads::SetChatMenuButtonSetters;
use teloxide::types::{MenuButton, MessageId};
use teloxide::RequestError;
use teloxide::{prelude::*, utils::command::BotCommands};
use thiserror::Error;
use tokio::sync::{
    oneshot::{self, Receiver},
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

#[derive(Debug, Error)]
pub enum AnswerError {
    #[error(transparent)]
    GetExamsError(#[from] GetExamsError),
    #[error(transparent)]
    GenericClientError(#[from] GenericClientError),
    #[error(transparent)]
    TeloxideError(#[from] RequestError),
}

async fn waiting_spinner(
    mut rx: Receiver<()>,
    bot: Arc<Bot>,
    chat_id: ChatId,
) -> Result<MessageId, RequestError> {
    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut iter = spinner.iter().cycle();

    // Send an initial message and get its MessageId to edit later
    let sent_message = bot.send_message(chat_id, *iter.next().unwrap()).await?;

    // Simulate the spinner by continuously editing the message
    for frame in iter {
        if rx.try_recv().is_ok() {
            break; // Stop the spinner when the signal is recieved
        }
        bot.edit_message_text(chat_id, sent_message.id, *frame)
            .await?;
        sleep(Duration::from_millis(100)).await; // Adjust the speed here
    }
    Ok(sent_message.id)
}

async fn handle_spinner_cmd(
    cmd: Command,
    client: Arc<Mutex<InfoCarClient>>,
) -> Result<String, GetExamsError> {
    match cmd {
        Command::Exams => {
            let exams = client.lock().await.get_nearest_exams(5).await?;
            Ok(format!(
                "The available exams are:\n{}",
                exams
                    .iter()
                    .map(|exam| format!("Exam ({}): {}\n", exam.id, exam.date))
                    .collect::<String>()
            ))
        }
        Command::Reservations => {
            let reservations = client.lock().await.get_reservations().await?;

            let text: String = reservations
                .items
                .iter()
                .map(|v| {
                    format!(
                        "• At {} in {} ({})\n\n",
                        v.exam
                            .practice
                            .as_ref()
                            .or(v.exam.theory.as_ref())
                            .unwrap()
                            .date,
                        v.exam.organization_unit_name,
                        v.status.status
                    )
                })
                .collect::<String>();
            Ok(text)
        }
        _ => unreachable!(),
    }
}

async fn answer(
    bot: Arc<Bot>,
    msg: Message,
    cmd: Command,
    client: Arc<Mutex<InfoCarClient>>,
    start_date: DateTime<Utc>,
) -> Result<(), AnswerError> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
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
        Command::Exam => {
            bot.send_message(msg.chat.id, "The current exam is: ")
                .await?;
        }
        Command::Enroll(exam_id) => {
            bot.send_message(
                msg.chat.id,
                format!("Do you want to enroll to exam {exam_id}"),
            )
            .await?;
        }
        // Handle spinner for all commands that use it
        _ => {
            // Create a oneshot channel to signal when to stop the spinner
            let (tx, rx) = oneshot::channel();

            let handle = tokio::spawn(waiting_spinner(rx, Arc::clone(&bot), msg.chat.id));

            let resp = handle_spinner_cmd(cmd, client).await;

            // Stop the spinner
            tx.send(()).unwrap();
            let message_id = handle.await.unwrap().unwrap();

            match resp {
                Ok(message) => {
                    bot.edit_message_text(msg.chat.id, message_id, message)
                        .await?;
                }
                Err(err) => {
                    bot.edit_message_text(msg.chat.id, message_id, format!("❌ Error: {err}"))
                        .await?;
                    Err(err)?;
                }
            }
        }
    }
    Ok(())
}

pub struct EasyCarService {
    pub bot: Arc<Bot>,
    pub teloxide_token: String,
    pub client: Arc<Mutex<InfoCarClient>>,
    pub chat_id: ChatId,
}

impl EasyCarService {
    pub fn new(teloxide_token: String, user_data: UserData, chat_id: String) -> Self {
        Self {
            bot: Arc::new(Bot::new(&teloxide_token)),
            client: Arc::new(Mutex::new(InfoCarClient::new(user_data))),
            teloxide_token,
            chat_id: ChatId(chat_id.parse().unwrap()),
        }
    }

    pub async fn start(self) -> Result<(), ()> {
        // Get a start date for the /uptime command
        let start_date = Utc::now();

        tokio::spawn(session_worker(Arc::clone(&self.client)));
        tokio::spawn(scheduler(
            Arc::clone(&self.client),
            Arc::clone(&self.bot),
            self.chat_id,
        ));

        self.bot
            .set_my_commands(Command::bot_commands())
            .await
            .unwrap();
        self.bot
            .set_chat_menu_button()
            .menu_button(MenuButton::Commands)
            .await
            .unwrap();

        Command::repl(
            self.bot,
            move |bot: Arc<Bot>, msg: Message, cmd: Command| {
                let client = Arc::clone(&self.client);
                async move {
                    if let Err(err) = answer(bot, msg, cmd, client, start_date).await {
                        match err {
                            AnswerError::TeloxideError(err) => return Err(err),
                            _ => println!("{err}"),
                        }
                    }
                    Ok(())
                }
            },
        )
        .await;

        Ok(())
    }
}

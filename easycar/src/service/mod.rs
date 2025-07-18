pub mod workers;

use std::error::Error;
use std::fmt::Write;
use std::num::ParseIntError;
use std::sync::Arc;

use crate::client::{GetExamsError, InfoCarClient, NewClientError, UserData};
use crate::utils::{date_from_string, readable_date_from_string, readable_time_delta};
use chrono::{DateTime, Utc};
use info_car_api::error::{EnrollError, GenericClientError};
use info_car_api::types::ProfileIdType;
use teloxide::payloads::SetChatMenuButtonSetters;
use teloxide::types::{MenuButton, MessageId, ParseMode};
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
    /// Gets current chat id
    #[command()]
    ChatId,
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
    /// Show reservation status
    #[command()]
    Status(String),
    /// Pay for the exam using a blik code (takes a space sperated reservation id and blik code)
    #[command()]
    Pay(String),
    /// Cancel reservation
    #[command()]
    Cancel(String),
}

#[derive(Debug, Error)]
pub enum AnswerError {
    #[error(transparent)]
    GetExamsError(#[from] GetExamsError),
    #[error(transparent)]
    GenericClientError(#[from] GenericClientError),
    #[error(transparent)]
    EnrollToExamError(#[from] EnrollError),
    #[error(transparent)]
    TeloxideError(#[from] RequestError),
    #[error("Too few arguments! Expected: {0}, got {1}!")]
    TooFewArguments(u32, u32),
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
            break; // Stop the spinner when the signal is received
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
) -> Result<String, AnswerError> {
    match cmd {
        Command::Exams => {
            let exams = client.lock().await.get_nearest_exams(5).await?;
            Ok(format!(
                "The available exams are:\n{}",
                exams.into_iter().fold(String::new(), |mut output, exam| {
                    let exam_in = date_from_string(&exam.date)
                        .signed_duration_since(Utc::now())
                        .num_days();
                    let readable_date = readable_date_from_string(exam.date);
                    let _ = writeln!(
                        output,
                        "Exam (<code>{}</code>): {} (in <b>{}</b> days)",
                        exam.id, readable_date, exam_in,
                    );
                    output
                })
            ))
        }
        Command::Reservations => {
            let reservations = client.lock().await.get_reservations().await?;

            let text: String = reservations
                .items
                .iter()
                .fold(String::new(), |mut output, v| {
                    let _ = write!(
                        output,
                        "• At {} in {} ({})\n\n",
                        v.exam
                            .practice
                            .as_ref()
                            .or(v.exam.theory.as_ref())
                            .unwrap()
                            .date,
                        v.exam.organization_unit_name,
                        v.status.status
                    );
                    output
                });
            Ok(text)
        }
        Command::Enroll(exam_id) => {
            let reservation_id = client.lock().await.enroll(exam_id).await?;

            Ok(format!(
                "Enrolled to the exam! The reservation id is <code>{reservation_id}</code>\nCheck the status using <code>/status {reservation_id}</code>"
            ))
        }
        Command::Status(reservation_id) => {
            let status = client.lock().await.status(reservation_id).await?;

            Ok(format!(
                "ID: {}\nStatus: {}\nReason: {}\nWord: {}\nAddress: {}\nCategory: {}\nDate: {}",
                status.id,
                status.status.status,
                status.status.message.unwrap_or("None".to_string()),
                status.exam.organization_unit_name,
                status.exam.address,
                status.exam.category,
                status.exam.exam_date,
            ))
        }
        Command::Cancel(reservation_id) => {
            client.lock().await.cancel(reservation_id.clone()).await?;

            Ok(format!(
                "Successfully canceled reservation: {reservation_id}",
            ))
        }
        Command::Pay(commands) => {
            let mut commands = commands.split_whitespace();

            let reservation_id = commands
                .next()
                .ok_or(AnswerError::TooFewArguments(2, 0))?
                .to_string();
            let blik_code = commands
                .next()
                .ok_or(AnswerError::TooFewArguments(2, 1))?
                .to_string();

            let response = client
                .lock()
                .await
                .pay(reservation_id.clone(), blik_code)
                .await?;

            Ok(format!(
                "Paid for exam {reservation_id} with {:.2} PLN.\nStatus: {}",
                response.paid_amount as f64 / 100.0,
                response.payment_status
            ))
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
        Command::ChatId => {
            bot.send_message(
                msg.chat.id,
                format!("The current ChatId is: <code>{}</code>", msg.chat.id),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Command::Exam => {
            bot.send_message(msg.chat.id, "The current exam is: ")
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
            let message_id = handle.await.unwrap()?;

            match resp {
                Ok(message) => {
                    bot.edit_message_text(msg.chat.id, message_id, message)
                        .parse_mode(ParseMode::Html)
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

#[derive(Error, Debug)]
pub enum NewServiceError {
    #[error(transparent)]
    ClientError(#[from] NewClientError),
    #[error("Failed to parse chat_id ({})", 0)]
    ChatIdParseError(#[from] ParseIntError),
}

impl EasyCarService {
    pub async fn new(
        teloxide_token: String,
        user_data: UserData,
        chat_id: String,
        pesel: String,
        phone_number: String,
        driver_profile_id: ProfileIdType,
    ) -> Result<Self, NewServiceError> {
        Ok(Self {
            bot: Arc::new(Bot::new(&teloxide_token)),
            client: Arc::new(Mutex::new(
                InfoCarClient::new(user_data, pesel, phone_number, driver_profile_id).await?,
            )),
            teloxide_token,
            chat_id: ChatId(chat_id.parse()?),
        })
    }

    pub async fn start(self) -> Result<(), RequestError> {
        // Get a start date for the /uptime command
        let start_date = Utc::now();

        tokio::spawn(session_worker(Arc::clone(&self.client)));
        tokio::spawn(scheduler(
            Arc::clone(&self.client),
            Arc::clone(&self.bot),
            self.chat_id,
        ));

        self.bot.set_my_commands(Command::bot_commands()).await?;
        self.bot
            .set_chat_menu_button()
            .menu_button(MenuButton::Commands)
            .await?;

        Command::repl(
            self.bot,
            move |bot: Arc<Bot>, msg: Message, cmd: Command| {
                let client = Arc::clone(&self.client);
                async move {
                    if let Err(err) = answer(bot, msg, cmd, client, start_date).await {
                        match err {
                            AnswerError::TeloxideError(err) => return Err(err),
                            _ => {
                                log::error!(
                                    "{err}{}",
                                    err.source()
                                        .map(|src| format!(". Source: {src}"))
                                        .unwrap_or("".to_owned())
                                );
                            }
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

#[cfg(feature = "shuttle")]
#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for EasyCarService {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        self.start()
            .await
            .expect("An error occurred while running the service!");
        Ok(())
    }
}

use teloxide::{prelude::*, types::ChatId};

pub struct BotLogger {
    chat_id: ChatId,
    bot: Bot,
}

impl BotLogger {
    pub fn new(teloxide_token: String, chat_id: ChatId) -> Self {
        Self {
            chat_id,
            bot: Bot::new(teloxide_token),
        }
    }

    pub async fn log(&self, error_message: &str) {
        println!("{error_message}");
        self.bot
            .send_message(self.chat_id, error_message)
            .await
            .expect("Failed to log the message");
    }
}

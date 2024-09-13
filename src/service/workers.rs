use crate::{client::InfoCarClient, utils::date_from_string};
use chrono::{Duration as ChronoDuration, Utc};
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration as TokioDuration},
};

pub async fn session_worker(client: Arc<Mutex<InfoCarClient>>) {
    let mut expire_date = client
        .lock()
        .await
        .get_token_expire_date()
        .expect("Token expire date is empty");
    loop {
        let duration = expire_date - Utc::now() - ChronoDuration::minutes(5);
        println!("Token expires in: {}", duration.num_seconds());
        sleep(TokioDuration::from_secs(
            duration.num_seconds().try_into().unwrap(),
        ))
        .await;
        expire_date = client.lock().await.refresh_token().await.unwrap();
    }
}

pub async fn scheduler(client: Arc<Mutex<InfoCarClient>>, bot: Arc<Bot>, chat_id: ChatId) {
    let mut last_exam_id = "".to_owned();
    loop {
        sleep(TokioDuration::from_secs(10)).await;
        let closest_exam = match client.lock().await.get_nearest_exams(1).await {
            Ok(mut v) => v.pop().unwrap(),
            Err(err) => {
                bot.send_message(chat_id, format!("Error: {err}"))
                    .await
                    .unwrap();
                continue;
            }
        };

        if closest_exam.id == last_exam_id {
            println!("No change...");
            continue;
        }

        last_exam_id = closest_exam.id;

        let duration = date_from_string(&closest_exam.date)
            .signed_duration_since(Utc::now())
            .num_days();
        let exam_message = format!(
            "New exam is available! The next exam date is {} (in {} days)",
            closest_exam.date, duration
        );

        println!("{exam_message}");
        bot.send_message(chat_id, exam_message).await.unwrap();
    }
}

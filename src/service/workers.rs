use crate::{
    client::{GetExamsError, InfoCarClient},
    utils::date_from_string,
};
use chrono::{Duration as ChronoDuration, Utc};
use info_car_api::error::EnrollError;
use std::{error::Error, sync::Arc};
use teloxide::{prelude::*, types::ParseMode};
use tokio::{
    sync::Mutex,
    time::{sleep, Duration as TokioDuration},
};

pub async fn session_worker(client: Arc<Mutex<InfoCarClient>>) {
    // A margin is used to refresh the token while it's still valid
    let token_refresh_margin = ChronoDuration::minutes(5);
    let mut expire_date = client
        .lock()
        .await
        .get_token_expire_date()
        .expect("Token expire date is empty");
    log::trace!("Got the token expire date ({expire_date})");

    loop {
        let duration = expire_date - Utc::now() - token_refresh_margin;
        let token_refresh_date = expire_date - token_refresh_margin;
        log::info!(
            "JWT will be refreshed on {} (in {} seconds)",
            token_refresh_date,
            duration.num_seconds()
        );
        sleep(TokioDuration::from_secs(
            duration
                .num_seconds()
                .try_into()
                .expect("Could not convert i64 TimeDelta to a u64"),
        ))
        .await;

        log::info!("Refreshing the token...");
        expire_date = client.lock().await.refresh_token().await.unwrap();
    }
}

pub async fn scheduler(client: Arc<Mutex<InfoCarClient>>, bot: Arc<Bot>, chat_id: ChatId) {
    let mut last_exam_id = "".to_owned();
    loop {
        let closest_exam = match client.lock().await.get_nearest_exams(1).await {
            Ok(mut v) => v.pop().unwrap(),
            Err(err) => {
                if let GetExamsError::GenericClientError(enroll_error) = &err {
                    if let EnrollError::GenericEndpointError(generic_error) = enroll_error {
                        if generic_error.0.get(0).expect("Empty vector").code == "invalid_token" {
                            bot.send_message(chat_id, "The token was invalid reloging...")
                                .await
                                .unwrap();
                            client.lock().await.refresh_token().await.unwrap();
                        }
                    }
                }
                log::error!(
                    "Got an error while retrieving new exams: {err}{}",
                    err.source()
                        .map(|src| format!(". Source: {src}"))
                        .unwrap_or("".to_owned())
                );
                bot.send_message(chat_id, format!("Error: {err}"))
                    .await
                    .unwrap();
                sleep(TokioDuration::from_secs(15)).await;
                continue;
            }
        };

        if closest_exam.id == last_exam_id {
            log::trace!("No change...");
            continue;
        }

        last_exam_id = closest_exam.id;

        let duration = date_from_string(&closest_exam.date)
            .signed_duration_since(Utc::now())
            .num_days();
        let exam_message = format!(
            "New exam is available! The next exam date is {} (in {} days) (ID: <code>{}</code>)",
            closest_exam.date, duration, &last_exam_id
        );

        log::info!("{exam_message}");
        bot.send_message(chat_id, exam_message)
            .parse_mode(ParseMode::Html)
            .await
            .unwrap();
        sleep(TokioDuration::from_secs(10)).await;
    }
}

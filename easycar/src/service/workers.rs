use crate::{
    client::{GetExamsError, InfoCarClient},
    utils::{date_from_string, readable_date_from_string},
};
use chrono::{Duration as ChronoDuration, Utc};
use info_car_api::error::EnrollError;
use std::{error::Error, sync::Arc};
use teloxide::{prelude::*, types::ParseMode};
use tokio::{
    sync::Mutex,
    time::{interval, sleep, Duration as TokioDuration},
};

pub async fn session_worker(client: Arc<Mutex<InfoCarClient>>) {
    // A margin is used to refresh the token while it's still valid
    let token_refresh_margin = ChronoDuration::minutes(5);
    let refresh_retry_timeout = ChronoDuration::seconds(15);
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
        expire_date = match client.lock().await.refresh_token().await {
            Ok(v) => v,
            Err(err) => {
                log::error!("Failed refresh the token. The attempt will be repeated in {} seconds. Error: {}", refresh_retry_timeout.num_seconds(), err);
                Utc::now() + refresh_retry_timeout
            }
        };
    }
}

pub async fn scheduler(client: Arc<Mutex<InfoCarClient>>, bot: Arc<Bot>, chat_id: ChatId) {
    let mut last_exam_id = "".to_owned();
    let mut interval = interval(TokioDuration::from_secs(15));
    loop {
        interval.tick().await;
        let closest_exam = match client.lock().await.get_nearest_exams(1).await {
            Ok(mut v) => v.pop().unwrap(),
            Err(err) => {
                if let GetExamsError::GenericClientError(EnrollError::GenericEndpointError(
                    generic_error,
                )) = &err
                {
                    if generic_error.0.first().expect("Empty vector").code == "invalid_token" {
                        bot.send_message(chat_id, "The token was invalid reloging...")
                            .await
                            .unwrap();
                        client.lock().await.refresh_token().await.unwrap();
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

        let date = readable_date_from_string(closest_exam.date);

        let exam_message = format!(
            "New exam is available! The next exam date is {} (in <b>{}</b> days) (ID: <code>{}</code>)",
            date, duration, &last_exam_id
        );

        log::info!("{exam_message}");
        bot.send_message(chat_id, exam_message)
            .parse_mode(ParseMode::Html)
            .await
            .unwrap();
    }
}

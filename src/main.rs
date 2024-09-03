use std::collections::HashMap;

use chrono::{DateTime, Days, Duration as ChronoDuration, Utc};
use dotenvy::dotenv;
use info_car_api::{
    client::{reservation::LicenseCategory, Client},
    utils::find_first_non_empty_practice_exam,
};
use tokio::{
    sync::mpsc,
    time::{sleep, Duration as TokioDuration},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect(".env file not found");

    let username = dotenvy::var("USERNAME")?;
    let password = dotenvy::var("PASSWORD")?;
    let webhook_url = dotenvy::var("WEBHOOK_URL")?;
    // let pesel = dotenvy::var("PESEL")?;
    // let phone_number = dotenvy::var("PHONE_NUMBER")?;
    // let pkk = dotenvy::var("PKK")?;
    let osk_id = "3";

    let mut client = Client::new();
    let make_client = reqwest::Client::new();

    // Create a channel for sending the token expire date
    let (tx, mut rx) = mpsc::channel::<DateTime<Utc>>(10);

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

    tokio::spawn(async move {
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
    });

    let mut last_id: String = "".to_owned();
    loop {
        if ry.try_recv().is_ok() {
            println!("Got refresh token signal. Refreshing...");
            client.refresh_token().await.unwrap();
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
                        let mut map = HashMap::new();
                        map.insert("title", "New exam available");
                        let exam_message = format!("Date: {}", exam[0].date);
                        map.insert("message", &exam_message);

                        let res = make_client
                            .post(&webhook_url)
                            .json(&map)
                            .send()
                            .await?
                            .text()
                            .await?;

                        println!("{res}");
                    } else {
                        println!("No change...")
                    }
                }
            }
            Err(err) => println!("{}", err),
        };

        sleep(TokioDuration::from_secs(10)).await;
    }
}

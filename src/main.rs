use chrono::{Days, Utc};
use dotenvy::dotenv;
use info_car_api::{
    client::{reservations::LicenseCategory, InfoCarClient},
    utils::find_first_non_empty_practice_exam,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect(".env file not found");

    let username = dotenvy::var("USERNAME")?;
    let password = dotenvy::var("PASSWORD")?;

    let mut client = InfoCarClient::new();
    client.login(&username, &password).await?;

    let mut last_id: String = "".to_owned();
    loop {
        let response = client
            .exam_schedule(
                "3".into(),
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
                    } else {
                        println!("No change...")
                    }
                }
            }
            Err(err) => println!("{}", err),
        };

        sleep(Duration::from_secs(10)).await;
    }
}

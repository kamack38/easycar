use easycar::service::{EasyCarService, UserData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_filename("Secrets.toml")?;

    // Get UserData
    let username = dotenvy::var("USERNAME")?;
    let password = dotenvy::var("PASSWORD")?;
    // let pesel = dotenvy::var("PESEL")?;
    // let phone_number = dotenvy::var("PHONE_NUMBER")?;
    // let pkk = dotenvy::var("PKK")?;
    let osk_id = "3";

    let chat_id = dotenvy::var("TELEGRAM_CHAT_ID")?;
    let teloxide_token = dotenvy::var("TELOXIDE_TOKEN")?;
    let user_data = UserData::new(username, password, osk_id.to_string(), chat_id);

    let service = EasyCarService::new(teloxide_token, user_data);
    service.start().await.expect("Service error");

    Ok(())
}

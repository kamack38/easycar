use std::num::NonZeroU32;

use easycar::{service::EasyCarService, UserData};
use info_car_api::types::ProfileIdType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    dotenvy::from_filename("Secrets.toml")?;

    // Get UserData
    let username = dotenvy::var("USERNAME")?;
    let password = dotenvy::var("PASSWORD")?;
    let pesel = dotenvy::var("PESEL")?;
    let phone_number = dotenvy::var("PHONE_NUMBER")?;
    let pkk = dotenvy::var("PKK")?;
    let osk_id: u32 = 3;

    let user_data = UserData::new(
        username,
        password,
        NonZeroU32::try_from(osk_id).expect("Osk_id is not a positive integer"),
    );

    let chat_id = dotenvy::var("TELEGRAM_CHAT_ID")?;
    let teloxide_token = dotenvy::var("TELOXIDE_TOKEN")?;

    let service = EasyCarService::new(
        teloxide_token,
        user_data,
        chat_id,
        pesel,
        phone_number,
        ProfileIdType::PKK(pkk),
    )
    .await?;
    service.start().await.expect("Service error");

    Ok(())
}

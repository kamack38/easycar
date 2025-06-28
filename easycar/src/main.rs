use std::num::NonZeroU32;

use easycar::{service::EasyCarService, UserData};
use info_car_api::types::ProfileIdType;

#[cfg(not(feature = "shuttle"))]
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
    let osk_id: u32 = dotenvy::var("OSK_ID")?.parse()?;

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

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<EasyCarService, shuttle_runtime::Error> {
    let username = secrets.get("USERNAME").expect("No USRENAME provided!");
    let password = secrets.get("PASSWORD").expect("No PASSWORD provided!");
    let pesel = secrets.get("PESEL").expect("No PESEL provided!");
    let phone_number = secrets
        .get("PHONE_NUMBER")
        .expect("No PHONE_NUMBER provided!");
    let pkk = secrets.get("PKK").expect("No PKK provided!");
    let osk_id: u32 = 3;

    let user_data = UserData::new(
        username,
        password,
        NonZeroU32::try_from(osk_id).expect("Osk_id is not a positive integer"),
    );

    let chat_id = secrets.get("TELEGRAM_CHAT_ID").unwrap();
    let teloxide_key = secrets
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

    Ok(EasyCarService::new(
        teloxide_key,
        user_data,
        chat_id,
        pesel,
        phone_number,
        ProfileIdType::PKK(pkk),
    )
    .await
    .unwrap())
}

use std::num::NonZeroU32;

use easycar::{service::EasyCarService, UserData};
use info_car_api::types::ProfileIdType;

struct ShuttleService(EasyCarService);

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for ShuttleService {
    async fn bind(mut self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        self.0
            .start()
            .await
            .expect("An error occurred while running the service!");

        Ok(())
    }
}

#[shuttle_runtime::main]
async fn init(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<ShuttleService, shuttle_runtime::Error> {
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

    println!("{teloxide_key}");

    Ok(ShuttleService(
        EasyCarService::new(
            teloxide_key,
            user_data,
            chat_id,
            pesel,
            phone_number,
            ProfileIdType::PKK(pkk),
        )
        .await
        .unwrap(),
    ))
}

use easycar::{service::EasyCarService, UserData};
use info_car_api::client::reservation::new::ProfileIdType;

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
    let username = secrets.get("USERNAME").unwrap();
    let password = secrets.get("PASSWORD").unwrap();
    let pesel = secrets.get("PESEL").unwrap();
    let phone_number = secrets.get("PHONE_NUMBER").unwrap();
    let pkk = secrets.get("PKK").unwrap();
    let osk_id = "3";

    let user_data = UserData::new(username, password, osk_id.to_string());

    let chat_id = secrets.get("TELEGRAM_CHAT_ID").unwrap();
    let teloxide_key = secrets
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

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

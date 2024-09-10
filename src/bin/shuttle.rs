use easycar::service::{EasyCarService, UserData};

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
    let teloxide_key = secrets
        .get("TELOXIDE_TOKEN")
        .expect("You need a teloxide key set for this to work!");

    let username = secrets.get("USERNAME").unwrap();
    let password = secrets.get("PASSWORD").unwrap();
    let chat_id = secrets.get("TELEGRAM_CHAT_ID").unwrap();

    Ok(ShuttleService(EasyCarService::new(
        teloxide_key,
        UserData::new(username, password, "3".to_owned(), chat_id),
    )))
}

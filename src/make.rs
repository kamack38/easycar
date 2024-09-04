use std::collections::HashMap;

pub struct MakeClient {
    client: reqwest::Client,
    webhook_url: String,
}

impl MakeClient {
    pub fn new(webhook_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            webhook_url,
        }
    }

    pub async fn send(
        &self,
        title: &str,
        message: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut map = HashMap::new();
        map.insert("title", title);
        map.insert("message", message);
        self.client.post(&self.webhook_url).json(&map).send().await
    }
}

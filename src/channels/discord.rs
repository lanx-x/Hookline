use crate::notification::{Notification, level_style};
use async_trait::async_trait;

use super::Channel;

pub struct DiscordChannel {
    pub name: String,
    pub webhook_url: String,
    client: reqwest::Client,
}

impl DiscordChannel {
    pub fn new(name: String, webhook_url: String) -> Self {
        Self {
            name,
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, notification: &Notification) -> Result<(), String> {
        let style = level_style(&notification.level);

        let payload = serde_json::json!({
            "embeds": [{
                "title": &notification.title,
                "description": &notification.message,
                "color": style.color
            }]
        });

        let resp = self
            .client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("failed to send discord webhook: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("discord api error: {}", resp.status()));
        }

        Ok(())
    }
}

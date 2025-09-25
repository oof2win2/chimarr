use std::panic;

use crate::{config, dispatchers::EventDispatcher, notifications::Notification};
use serde::Serialize;

#[derive(Serialize)]
struct DiscordWebhookBody {
    content: String,
}

#[derive(Clone)]
pub struct DiscordDispatcher {
    webhook_url: String,
    reqwest: reqwest::Client,
}
impl EventDispatcher for DiscordDispatcher {
    fn new() -> DiscordDispatcher {
        let webhook_url = config::app::discord::webhook_url();
        if webhook_url.is_err() {
            panic!("Webhook URL not provided")
        }

        let reqwest_client = reqwest::Client::new();

        DiscordDispatcher {
            webhook_url: webhook_url.unwrap(),
            reqwest: reqwest_client,
        }
    }

    async fn send_message(&self, msg: Notification) -> () {
        let body = DiscordWebhookBody {
            content: format!(
                "Notification {} ({}): {}",
                msg.id, msg.notification_type, msg.message
            )
            .to_string(),
        };
        self.reqwest
            .post(&self.webhook_url)
            .query(&[("wait", "true")])
            .json(&body)
            .send()
            .await;
    }
}

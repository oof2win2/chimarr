use std::panic;

use crate::{config, dispatchers::EventDispatcher, notifications::Notification};
use serde::Serialize;

#[derive(Serialize, Clone)]
struct DiscordWebhookBody {
    content: String,
}

#[derive(Clone)]
pub struct DiscordDispatcher {
    webhook_url: String,
    reqwest: reqwest::Client,
    message_queue: Vec<DiscordWebhookBody>,
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
            message_queue: vec![],
        }
    }

    fn send_message(&mut self, msg: Notification) -> () {
        let body = DiscordWebhookBody {
            content: format!(
                "Notification {} ({}): {}",
                msg.id, msg.notification_type, msg.message
            )
            .to_string(),
        };
        self.message_queue.push(body);
    }

    async fn flush_messages(&mut self) -> anyhow::Result<()> {
        // we use .drain so that we don't void the messages that didn't make it through
        for msg in self.message_queue.drain(..) {
            self.reqwest
                .post(&self.webhook_url)
                .query(&[("wait", "true")])
                .json(&msg)
                .send()
                .await?;
        }

        Ok(())
    }
}

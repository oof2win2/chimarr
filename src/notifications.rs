use std::hash::{DefaultHasher, Hash, Hasher};

use nanoid::nanoid;

use crate::dispatchers::{self, EventDispatcher, discord::DiscordDispatcher};

#[derive(Hash, Debug, Clone)]
pub enum NotificationType {
    Info,
    Error,
}

pub struct BareNotification {
    pub r#type: NotificationType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Notification {
    id: String,
    r#type: NotificationType,
    message: String,
    hash: u64,
}

pub struct NotificationManager {
    notifications: Vec<Notification>,
    discord: DiscordDispatcher,
}
impl NotificationManager {
    pub async fn new() -> NotificationManager {
        let discord = dispatchers::discord::DiscordDispatcher {};
        discord.initialize().await;

        NotificationManager {
            notifications: vec![],
            discord: discord,
        }
    }

    pub fn send_notification_sync(&mut self, notif: BareNotification) {
        if self.has_existing_notification(&notif) {
            return;
        }

        let hash = self.hash_notif(&notif);

        let notification = Notification {
            r#type: notif.r#type,
            message: notif.message,
            hash,
            id: nanoid!(),
        };

        self.notifications.push(notification.clone());

        // Spawn a task to send the notification without holding the mutex
        let discord = self.discord.clone();
        tokio::spawn(async move {
            discord.send_message(notification).await;
        });
    }

    fn has_existing_notification(&self, notif: &BareNotification) -> bool {
        let hash = self.hash_notif(notif);

        let exists = self.notifications.iter().find(|&notif| notif.hash == hash);

        exists.is_some()
    }

    fn hash_notif(&self, notif: &BareNotification) -> u64 {
        let mut hasher = DefaultHasher::new();
        notif.message.hash(&mut hasher);
        notif.r#type.hash(&mut hasher);
        hasher.finish()
    }
}

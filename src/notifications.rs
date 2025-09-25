use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use nanoid::nanoid;

use crate::dispatchers::{self, EventDispatcher, discord::DiscordDispatcher};

#[derive(Hash, Debug, Clone)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
}
impl Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Info => {
                write!(f, "INFO");
                Ok(())
            }
            NotificationType::Warning => {
                write!(f, "WARNING");
                Ok(())
            }
            NotificationType::Error => {
                write!(f, "ERROR");
                Ok(())
            }
        }
    }
}

pub struct BareNotification {
    pub notification_type: NotificationType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub notification_type: NotificationType,
    pub message: String,
    hash: u64,
}

pub struct NotificationManager {
    notifications: Vec<Notification>,
    discord: DiscordDispatcher,
}
impl NotificationManager {
    pub async fn new() -> NotificationManager {
        let discord = dispatchers::discord::DiscordDispatcher::new();

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
            notification_type: notif.notification_type,
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
        notif.notification_type.hash(&mut hasher);
        hasher.finish()
    }
}

use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};
use tokio::sync::Mutex;

use nanoid::nanoid;
use tokio_cron::{Job, Scheduler};

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
    discord: Arc<Mutex<DiscordDispatcher>>,
}
impl NotificationManager {
    pub async fn new(scheduler: &mut Scheduler) -> NotificationManager {
        let discord = Arc::new(Mutex::new(dispatchers::discord::DiscordDispatcher::new()));
        let discord_clone = discord.clone();

        scheduler.add(Job::named("Send all messages", "* * * * * *", move || {
            let discord_clone_inner = discord_clone.clone();
            async move {
                let mut discord_guard = discord_clone_inner.lock().await;
                let _ = discord_guard.flush_messages().await;
            }
        }));

        NotificationManager {
            notifications: vec![],
            discord,
        }
    }

    pub async fn send_notification(&mut self, notif: BareNotification) {
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

        self.discord.lock().await.send_message(notification);
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

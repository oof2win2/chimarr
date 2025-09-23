use crate::notifications::Notification;

pub mod discord;

pub trait EventDispatcher {
    async fn initialize(&self) -> ();
    async fn send_message(&self, msg: Notification) -> ();
}

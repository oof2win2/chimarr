use crate::notifications::Notification;

pub mod discord;

pub trait EventDispatcher {
    fn new() -> Self
    where
        Self: Sized;
    async fn send_message(&self, msg: Notification) -> ();
}

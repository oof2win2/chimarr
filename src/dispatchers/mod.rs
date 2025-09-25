use crate::notifications::Notification;

pub mod discord;

pub trait EventDispatcher {
    fn new() -> Self
    where
        Self: Sized;

    fn send_message(&mut self, msg: Notification) -> ();
    async fn flush_messages(&mut self) -> anyhow::Result<()>;
}

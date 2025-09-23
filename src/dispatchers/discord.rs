use crate::{dispatchers::EventDispatcher, notifications::Notification};

#[derive(Clone)]
pub struct DiscordDispatcher {}
impl EventDispatcher for DiscordDispatcher {
    async fn initialize(&self) -> () {}
    async fn send_message(&self, msg: Notification) -> () {
        println!("Sending message {:?}", msg);
    }
}

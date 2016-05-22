extern crate crossbeam;
extern crate serde_json;

pub mod irc;
pub mod discord;
pub mod server;

use std::sync::mpsc::Sender;

pub struct ReplyMessage(pub ChatMessage, pub Option<Sender<Option<String>>>);

#[derive(Deserialize, Debug)]
pub struct ChatMessage {
    pub source: String,
    pub author: String,
    pub message: String,
}

extern crate crossbeam;
extern crate serde_json;

mod irc;
mod discord;
mod server;

use std::sync::mpsc::Sender;

pub struct ReplyMessage(pub ChatMessage, pub Option<Sender<String>>);

#[derive(Deserialize, Debug)]
pub struct ChatMessage {
    pub source: String,
    pub author: String,
    pub message: String,
}


pub fn connect(scope: &crossbeam::Scope, sender: Sender<ReplyMessage>)
{
    let sender_clone = sender.clone();
    scope.spawn(move || irc::connect("testing", sender_clone));
    let sender_clone = sender.clone();
    scope.spawn(move || discord::connect(sender_clone));
    scope.spawn(move || server::listen(sender));
}
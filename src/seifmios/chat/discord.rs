extern crate discord;
extern crate serde_json;

use self::discord::Discord;
use self::discord::model::Event;
use std::fs::File;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use chat::ChatMessage;
use chat::ReplyMessage;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct Config {
    name: String,
    email: String,
    password: String,
}

pub fn connect<P>(sender: Sender<ReplyMessage>, path: P)
    where P: AsRef<Path>
{
    let file = File::open(path).unwrap();
    let config: Config = serde_json::from_reader(file).unwrap();
    let discord = Discord::new(config.email.as_str(), config.password.as_str()).expect("login failed");

    let (mut connection, _) = discord.connect().expect("connect failed");
    println!("Ready.");
    loop {
        match connection.recv_event() {
            Ok(Event::Closed(n)) => {
                println!("Discord closed on us with status {}", n);
                let (new_connection, _) = discord.connect().expect("connect failed");
                connection = new_connection;
            }
            Ok(Event::MessageCreate(message)) => {
                println!("{} says: {}", message.author.name, message.content);
                if message.content == "!quit" {
                    println!("Quitting.");
                    break
                }
                else {
                    let chat_message = ChatMessage {
                            source: message.channel_id.0.to_string(),
                            author: message.author.name,
                            message: message.content.clone(),
                        };
                    if message.content.contains(config.name.as_str()) {
                        let (reply_sender, reply_reciever) = channel();
                        sender.send(ReplyMessage(chat_message, Some(reply_sender))).unwrap();
                        if let Some(reply_string) = reply_reciever.recv().unwrap() {
                            let _ = discord.send_message(&message.channel_id, &reply_string, "", false);
                        }
                    }
                    else {
                        sender.send(ReplyMessage(chat_message, None)).unwrap();
                    }
                }
            }
            Ok(_) => {}
            Err(err) => {
                println!("Receive error: {:?}", err);
                let (new_connection, _) = discord.connect().expect("connect failed");
                connection = new_connection;
            }
        }
    }

    discord.logout().expect("logout failed");
}

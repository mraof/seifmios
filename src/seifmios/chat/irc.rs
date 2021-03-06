extern crate irc;
use self::irc::client::prelude::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use chat::ChatMessage;
use chat::ReplyMessage;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

extern crate itertools;
use self::itertools::Itertools;

const IRC_RECONNECT_WAIT: u64 = 1;
const IRC_MILI_LIMITER: u64 = 1500;

pub fn connect<P>(sender: Sender<ReplyMessage>, path: P)
    where P: AsRef<Path>
{
    let config = Config::load(path)
        .unwrap_or_else(|e| panic!("IRC Fatal: Couldn't load config file: {}", e));
    let mut last_send = Instant::now();
    loop {
        let server = IrcServer::from_config(config.clone())
            .unwrap_or_else(|e| panic!("IRC Fatal: Couldn't make server: {}", e));
        server.identify().unwrap_or_else(|e| panic!("IRC Fatal: Failed to identify server: {}", e));
        for message in server.iter() {
            let message = message.unwrap_or_else(|e| panic!("IRC Fatal: Failed to get message: {}", e));
            match message.command {
                Command::PRIVMSG(target, msg) => {
                    let name = message.prefix.unwrap_or_else(|| panic!("IRC Fatal: Unable to get msg name"));
                    let nick = server.config().nickname().to_string();
                    let chat_message = ChatMessage {
                        source: target.clone(),
                        author: name,
                        message: msg.split(' ').filter(|e| e != &nick).join(" "),
                    };
                    if msg.contains(&nick) && Instant::now() - last_send > Duration::from_millis(IRC_MILI_LIMITER) {
                        let (reply_sender, reply_reciever) = channel();
                        sender.send(ReplyMessage(chat_message, Some(reply_sender)))
                            .unwrap_or_else(|e| panic!("IRC Fatal: Message sender closed: {}", e));
                        let reply_message = reply_reciever.recv()
                            .unwrap_or_else(|e| panic!("IRC Fatal: Reply receiver failed: {}", e));
                        if let Some(reply_string) = reply_message {
                            if let Some(fc) = reply_string.chars().next() {
                                if fc != '.' && fc != '/' {
                                    server.send_privmsg(target.as_str(), reply_string.as_str())
                                        .unwrap_or_else(|e| panic!("IRC Fatal: Failed to send message: {}", e));
                                }
                            }
                        }
                        last_send = Instant::now();
                    } else {
                        sender.send(ReplyMessage(chat_message, None))
                            .unwrap_or_else(|e| panic!("IRC Fatal: Message sender closed: {}", e));
                    }
                },
                _ => {},
            }
        }
        // Connection ended, so we need to wait an amount of time before trying again
        sleep(Duration::from_secs(IRC_RECONNECT_WAIT));
    }
}

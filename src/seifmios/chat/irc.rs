extern crate irc;
use self::irc::client::prelude::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use chat::ChatMessage;
use chat::ReplyMessage;
use std::path::Path;

pub fn connect<P>(sender: Sender<ReplyMessage>, path: P)
    where P: AsRef<Path>
{
    let config = Config::load(path).unwrap();
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    for message in server.iter() {
        let message = message.unwrap();
        match message.command {
            Command::PRIVMSG(target, msg) => {
                let name = message.prefix.unwrap();
                println!("{}: {}: {}", target, name.clone(), msg);
                if msg == "!quit"
                {
                    server.send_quit("Quitting").unwrap();
                }
                else
                {
                    let chat_message = ChatMessage {
                        source: target.clone(),
                        author: name,
                        message: msg.clone(),
                    };
                    if msg.contains(server.config().nickname()) {
                        let (reply_sender, reply_reciever) = channel();
                        sender.send(ReplyMessage(chat_message, Some(reply_sender))).unwrap();
                        let reply_message = reply_reciever.recv().unwrap();
                        if let Some(fc) = reply_message.chars().next() {
                            if fc != '.' && fc != '/' {
                                server.send_privmsg(target.as_str(), reply_message.as_str()).unwrap();
                            }
                        }
                    }
                    else {
                        sender.send(ReplyMessage(chat_message, None)).unwrap();
                    }
                }
            },
            _ => (),
        }
    }
}

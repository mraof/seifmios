extern crate irc;
use self::irc::client::prelude::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use chat::ChatMessage;
use chat::ReplyMessage;

pub fn connect(name: &str, sender: Sender<ReplyMessage>)
{
    let config = Config::load(format!("config/irc/{}.json", name)).unwrap();
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
                        println!("{:?}", server.send_privmsg(target.as_str(), reply_reciever.recv().unwrap().as_str()).unwrap());
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

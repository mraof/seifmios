extern crate irc;
use self::irc::client::prelude::*;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

pub fn connect(name: &str, sender: Sender<(String, String, String, Option<Sender<String>>)>)
{
    let config = Config::load(format!("irc/{}.json", name)).unwrap();
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    for message in server.iter() {
        let message = message.unwrap();
        print!("{:?}", message);
        match message.command {
            Command::PRIVMSG(target, msg) => {
                if msg == "!quit"
                {
                    server.send_quit("Quitting").unwrap();
                }
                else
                {
                    /*let source = lex.source(target.clone());
                    let author = lex.author(source.clone(), message.prefix.unwrap());
                    lex.tell(source.clone(), author, msg.clone());*/
                    if msg.contains(server.config().nickname()) {
                        let (reply_sender, reply_reciever) = channel();
                        sender.send((target.clone(), message.prefix.unwrap(), msg, Some(reply_sender)));
                        println!("{:?}", server.send_privmsg(target.as_str(), reply_reciever.recv().unwrap().as_str()).unwrap());
                    }
                    else {
                        sender.send((target, message.prefix.unwrap(), msg, None));
                    }
                }
            },
            _ => (),
        }
    }
}
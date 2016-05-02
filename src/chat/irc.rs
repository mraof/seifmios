extern crate irc;
extern crate crossbeam;
extern crate rand;
use self::irc::client::prelude::*;
use text;

pub fn connect<R: rand::Rng>(name: &str, lex: &mut text::Lexicon<R>)
{
    let config = Config::load(format!("irc/{}.json", name)).unwrap();
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    for message in server.iter() {
        let message = message.unwrap();
        print!("{:?}", message);
        match message.command {
            Command::PRIVMSG(target, msg) => {
                if(msg == "!quit")
                {
                    server.send_quit("Quitting").unwrap();
                }
                else
                {
                    let source = lex.source(target.clone());
                    let author = lex.author(source.clone(), message.prefix.unwrap());
                    lex.tell(source.clone(), author, msg.clone());
                    if msg.contains(server.config().nickname())
                    {
                        println!("{:?}", server.send_privmsg(target.as_str(), lex.initiate(source).unwrap().as_str()).unwrap());
                    }
                }
            },
            _ => (),
        }
    }
}
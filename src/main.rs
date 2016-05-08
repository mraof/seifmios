#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde;
extern crate rand;
extern crate crossbeam;

use std::io::{BufReader, BufRead, stdout, Write};
use std::fs::File;
use std::sync::mpsc::{channel, TryRecvError};
use std::thread::spawn;

mod text;
mod cli;
mod chat;

fn reset() {
    print!(">");
    stdout().flush().unwrap();
}

fn main() {
    use rand::SeedableRng;
    let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
    let console = lex.source("console".to_string());
    let me = lex.author(console.clone(), "me".to_string());
    let (sender, receiver) = channel();
    let mut server_running = false;
    reset();
    for decision in cli::new() {
        use cli::Decision;
        match decision {
            Decision::None => {
                match receiver.try_recv() {
                    Ok(chat::ReplyMessage(message, replier)) => {
                        let source = lex.source(message.source.clone());
                        let author = lex.author(source.clone(), message.author);
                        lex.tell(source.clone(), author.clone(), message.message);
                        if let Some(reply_sender) = replier {
                            if let Some(reply) = lex.respond(source) {
                                if reply_sender.send(reply).is_err() {
                                    println!("");
                                    println!("Warning: Reply sender from {} closed unexpectedly", message.source);
                                    reset();
                                }
                            }
                        }
                    },
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => {
                        panic!("Fatal: The main sender just disappeared!?");
                    },
                }
                lex.think();
            },
            Decision::ImportLines(filename) => {
                let author = lex.author(console.clone(), filename.clone());
                let file = File::open(&filename);
                match file {
                    Ok(f) => {
                        for (index, line) in BufReader::new(f).lines().enumerate() {
                            match line {
                                Ok(s) => {
                                    lex.tell(console.clone(), author.clone(), s);
                                    if (index + 1) % 10000 == 0 {
                                        println!("On line {} of {}", index + 1, filename);
                                    }
                                },
                                Err(_) => {
                                    println!("Ignoring: File had read error on line {}", index + 1);
                                },
                            }
                        }
                    },
                    Err(_) => {
                        println!("Ignoring: Unable to open file");
                    },
                }
                reset();
            },
            Decision::ShowCategories => {
                lex.show_categories();
                reset();
            },
            Decision::Respond => {
                if let Some(s) = lex.initiate(console.clone()) {
                    println!("{}", s);
                }
                reset();
            },
            Decision::Tell(s) => {
                lex.tell(console.clone(), me.clone(), s);
                reset();
            },
            Decision::ResetCursor => {
                reset();
            },
            Decision::ConnectServer => {
                if server_running {
                    println!("Ignored: Server already running");
                } else {
                    let sender = sender.clone();
                    spawn(move || chat::server::listen(sender));
                    server_running = true;
                }
                reset();
            },
            Decision::ConnectIrc(config) => {
                let sender = sender.clone();
                spawn(move || chat::irc::connect(sender, config));
                reset();
            },
            Decision::ConnectDiscord(config) => {
                let sender = sender.clone();
                spawn(move || chat::discord::connect(sender, config));
                reset();
            },
        }
    }
}

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde;
extern crate rand;
extern crate crossbeam;

use std::io::{BufReader, BufRead};
use std::fs::File;
use std::sync::mpsc::{channel, TryRecvError};
use std::thread::spawn;

mod text;
mod cli;
mod chat;

fn main() {
    use rand::SeedableRng;
    let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
    let console = lex.source("console".to_string());
    let me = lex.author(console.clone(), "me".to_string());
    let (sender, receiver) = channel();
    let mut server_running = false;
    for response in cli::new() {
        match response {
            Some((decision, mut socket)) => {
                use cli::Decision;
                match decision {
                    Decision::Quit => {
                        return;
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
                                                socket.msg(&format!("On line {} of {}", index + 1, filename));
                                            }
                                        },
                                        Err(_) => {
                                            socket.msg(&format!("Ignored: File had read error on line {}", index + 1));
                                        },
                                    }
                                }
                            },
                            Err(_) => {
                                socket.msg("Ignored: Unable to open file");
                            },
                        }
                    },
                    Decision::ShowCategories => {
                        lex.show_categories(&mut socket);
                    },
                    Decision::Respond => {
                        if let Some(s) = lex.respond(console.clone()) {
                            socket.msg(&format!("Original: {}\nResponse: {}", s.0, s.1));
                        }
                    },
                    Decision::Tell(s) => {
                        lex.tell(console.clone(), me.clone(), s);
                    },
                    Decision::ConnectServer => {
                        if server_running {
                            socket.msg("Ignored: Server already running");
                        } else {
                            let sender = sender.clone();
                            spawn(move || chat::server::listen(sender));
                            server_running = true;
                        }
                    },
                    Decision::ConnectIrc(config) => {
                        let sender = sender.clone();
                        spawn(move || chat::irc::connect(sender, config));
                    },
                    Decision::ConnectDiscord(config) => {
                        let sender = sender.clone();
                        spawn(move || chat::discord::connect(sender, config));
                    },
                    Decision::SetCocategoryRatio(f) => {
                        lex.cocategorization_ratio = f;
                    },
                    Decision::GetCocategoryRatio => {
                        socket.msg(&format!("{}", lex.cocategorization_ratio));
                    },
                    Decision::SetTravelDistance(steps) => {
                        lex.cocategory_travel_distance = steps;
                    },
                    Decision::GetTravelDistance => {
                        socket.msg(&format!("{}", lex.cocategory_travel_distance));
                    },
                    Decision::SetCocategorizeMagnitude(cycles) => {
                        lex.cocategorize_magnitude = cycles;
                    },
                    Decision::GetCocategorizeMagnitude => {
                        socket.msg(&format!("{}", lex.cocategorize_magnitude));
                    },
                    Decision::SetForwardEdgeDistance(cycles) => {
                        lex.forward_edge_distance = cycles;
                    },
                    Decision::GetForwardEdgeDistance => {
                        socket.msg(&format!("{}", lex.forward_edge_distance));
                    },
                    Decision::SetBackwardEdgeDistance(cycles) => {
                        lex.backward_edge_distance = cycles;
                    },
                    Decision::GetBackwardEdgeDistance => {
                        socket.msg(&format!("{}", lex.backward_edge_distance));
                    },
                    Decision::SetForwardWordDistance(cycles) => {
                        lex.forward_word_distance = cycles;
                    },
                    Decision::GetForwardWordDistance => {
                        socket.msg(&format!("{}", lex.forward_word_distance));
                    },
                    Decision::SetBackwardWordDistance(cycles) => {
                        lex.backward_word_distance = cycles;
                    },
                    Decision::GetBackwardWordDistance => {
                        socket.msg(&format!("{}", lex.backward_word_distance));
                    },
                    Decision::FindRelation(words) => {
                        lex.find_relation(words, &mut socket);
                    },
                }
            },
            None => {
                match receiver.try_recv() {
                    Ok(chat::ReplyMessage(message, replier)) => {
                        let source = lex.source(message.source.clone());
                        let author = lex.author(source.clone(), message.author);
                        lex.tell(source.clone(), author.clone(), message.message);
                        if let Some(reply_sender) = replier {
                            if let Some(reply) = lex.respond(source) {
                                if reply_sender.send(Some(reply.1)).is_err() {
                                    println!("Warning: Reply sender from {} closed unexpectedly", message.source);
                                }
                            } else {
                                if reply_sender.send(None).is_err() {
                                    println!("Warning: Reply sender from {} closed unexpectedly", message.source);
                                }
                            }
                        }
                    },
                    Err(TryRecvError::Empty) => lex.think(),
                    Err(TryRecvError::Disconnected) => panic!("Fatal: The main sender just disappeared!?"),
                }
            },
        }
    }
}

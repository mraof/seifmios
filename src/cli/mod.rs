use std::thread::{JoinHandle, spawn};
use std::sync::mpsc::{Receiver, channel};
use std::io::{self, BufRead};
extern crate itertools;
use self::itertools::Itertools;
use std::sync::mpsc::TryRecvError;

extern crate either;

pub enum Decision {
    None,
    ImportLines(String),
    ShowCategories,
    ResetCursor,
    Respond,
    Tell(String),
    ConnectServer,
    ConnectIrc(String),
    ConnectDiscord(String),
    ChangeCocategoryRatio(f64),
    GetCocategoryRatio,
}

pub fn new() -> Iter {
    let (sender, receiver) = channel();
    Iter{
        _thread: spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(s) => {
                        let params = s.split('`')
                            .enumerate()
                            .flat_map(|(iter, s)| {
                                // If we are not in a quotation
                                if iter % 2 == 0 {
                                    either::Left(s.split(' '))
                                } else {
                                    use std::iter::once;
                                    either::Right(once(s))
                                }
                            })
                            .filter(|s| !s.is_empty()).map(|s| s.to_string()).collect_vec();
                        if sender.send(params).is_err() {
                            break;
                        }
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
        }),
        receiver: receiver,
    }
}

pub struct Iter {
    _thread: JoinHandle<()>,
    receiver: Receiver<Vec<String>>,
}

impl Iterator for Iter {
    type Item = Decision;

    fn next(&mut self) -> Option<Decision> {
        match self.receiver.try_recv() {
            Ok(params) => {
                let help = || {
                    println!("Available commands: import, connect, list, respond, tell, get, set");
                    Some(Decision::ResetCursor)
                };

                match params.len() {
                    0 => {
                        help()
                    },
                    _ => {
                        match &*params[0] {
                            "help" => {
                                help()
                            },
                            "import" => {
                                if params.len() < 2 {
                                    println!("Usage: import <import type>");
                                    println!("Available import types: lines");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match &*params[1] {
                                        "lines" => {
                                            if params.len() != 3 {
                                                println!("Usage: import lines <filname>");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                println!("Importing lines from `{}`...", params[2]);
                                                Some(Decision::ImportLines(params[2].to_string()))
                                            }
                                        },
                                        _ => {
                                            println!("Ignored: Unrecognized import type");
                                            Some(Decision::ResetCursor)
                                        },
                                    }
                                }
                            },
                            "set" => {
                                if params.len() < 2 {
                                    println!("Usage: set <value>");
                                    println!("Values: cc_ratio");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match &*params[1] {
                                        "cc_ratio" => {
                                            if params.len() != 3 {
                                                println!("Usage: set cc_ratio <ratio>");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                match params[2].parse::<f64>() {
                                                    Ok(f) => {
                                                        Some(Decision::ChangeCocategoryRatio(f))
                                                    },
                                                    Err(e) => {
                                                        println!("Ignored: Error converting value: {}", e);
                                                        Some(Decision::ResetCursor)
                                                    },
                                                }
                                            }
                                        },
                                        _ => {
                                            println!("Ignored: Unrecognized set value");
                                            Some(Decision::ResetCursor)
                                        },
                                    }
                                }
                            },
                            "get" => {
                                if params.len() < 2 {
                                    println!("Usage: get <value>");
                                    println!("Values: cc_ratio");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match &*params[1] {
                                        "cc_ratio" => {
                                            if params.len() != 2 {
                                                println!("Usage: get cc_ratio");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                Some(Decision::GetCocategoryRatio)
                                            }
                                        },
                                        _ => {
                                            println!("Ignored: Unrecognized get value");
                                            Some(Decision::ResetCursor)
                                        },
                                    }
                                }
                            },
                            "connect" => {
                                if params.len() < 2 {
                                    println!("Ignored: connect takes at least a connect type");
                                    println!("Connect types: server, irc, discord");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match &*params[1] {
                                        "server" => {
                                            if params.len() != 2 {
                                                println!("Ignored: no extra parameters needed");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                Some(Decision::ConnectServer)
                                            }
                                        },
                                        "irc" => {
                                            if params.len() != 3 {
                                                println!("Usage: connect irc <config>");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                Some(Decision::ConnectIrc(params[2].to_string()))
                                            }
                                        },
                                        "discord" => {
                                            if params.len() != 3 {
                                                println!("Usage: connect discord <config>");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                Some(Decision::ConnectDiscord(params[2].to_string()))
                                            }
                                        },
                                        _ => {
                                            println!("Ignored: Unrecognized connect type");
                                            Some(Decision::ResetCursor)
                                        },
                                    }
                                }
                            },
                            "list" => {
                                if params.len() != 2 {
                                    println!("Usage: list <list type>");
                                    println!("Available list types: categories");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match &*params[1] {
                                        "categories" => {
                                            Some(Decision::ShowCategories)
                                        },
                                        _ => {
                                            println!("Ignored: Unrecognized list type");
                                            Some(Decision::ResetCursor)
                                        },
                                    }
                                }
                            },
                            "respond" => {
                                if params.len() != 1 {
                                    println!("Usage: respond");
                                    Some(Decision::ResetCursor)
                                } else {
                                    Some(Decision::Respond)
                                }
                            },
                            "tell" => {
                                if params.len() != 2 {
                                    println!("Usage: tell <message>");
                                    Some(Decision::ResetCursor)
                                } else {
                                    Some(Decision::Tell(params[1].to_string()))
                                }
                            },
                            _ => {
                                println!("Ignored: Unrecognized command");
                                Some(Decision::ResetCursor)
                            },
                        }
                    }
                }
            },
            Err(TryRecvError::Empty) => {
                Some(Decision::None)
            },
            Err(TryRecvError::Disconnected) => {
                None
            },
        }
    }
}

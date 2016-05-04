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
}

pub fn new() -> Iter {
    let (sender, receiver) = channel();
    Iter{
        _thread: spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(s) => {
                        if sender.send(s).is_err() {
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
    receiver: Receiver<String>,
}

impl Iterator for Iter {
    type Item = Decision;

    fn next(&mut self) -> Option<Decision> {
        match self.receiver.try_recv() {
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
                    .filter(|s| !s.is_empty()).collect_vec();

                match params.len() {
                    0 => {
                        println!("Available commands: import, connect, list, respond, tell");
                        Some(Decision::ResetCursor)
                    },
                    _ => {
                        match params[0] {
                            "import" => {
                                if params.len() != 3 {
                                    println!("Ignored: import takes 2 params");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match params[1] {
                                        "lines" => {
                                            println!("Importing lines from `{}`...", params[2]);
                                            Some(Decision::ImportLines(params[2].to_string()))
                                        },
                                        _ => {
                                            println!("Ignored: Unrecognized import type");
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
                                    match params[1] {
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
                                                println!("Ignored: connect irc requires a config path");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                Some(Decision::ConnectIrc(params[3].to_string()))
                                            }
                                        },
                                        "discord" => {
                                            if params.len() != 3 {
                                                println!("Ignored: connect discord requires a config path");
                                                Some(Decision::ResetCursor)
                                            } else {
                                                Some(Decision::ConnectDiscord(params[3].to_string()))
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
                                    println!("Ignored: list takes 1 param");
                                    Some(Decision::ResetCursor)
                                } else {
                                    match params[1] {
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
                                    println!("Ignored: respond takes no params");
                                    Some(Decision::ResetCursor)
                                } else {
                                    Some(Decision::Respond)
                                }
                            },
                            "tell" => {
                                if params.len() != 2 {
                                    println!("Ignored: respond takes 1 param");
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

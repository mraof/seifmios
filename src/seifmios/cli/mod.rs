use std::thread::{JoinHandle, spawn};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

extern crate serde_json;
use self::serde_json::de::from_str;

extern crate zmq;
use self::zmq::Socket;

#[path = "../../shared/ipc.rs"]
mod ipc;

extern crate either;

pub struct SocketLend {
    socket: Option<Socket>,
    socket_return: Sender<Socket>,
}

impl SocketLend {
    pub fn msg(&mut self, m: &str) {
        self.socket.as_mut().unwrap().send_str(m, 0).unwrap_or_else(|e| {
            println!("Warning: Unable to send message over return socket: {}", e);
        });
    }
}

impl Drop for SocketLend {
    fn drop(&mut self) {
        self.msg("");
        self.socket_return.send(self.socket.take().unwrap()).unwrap_or_else(|e| {
            println!("Warning: Failed to return socket; cli now unavailable: {}", e);
        });
    }
}

pub enum Decision {
    Quit,
    ImportLines(String),
    ShowCategories,
    Respond,
    Tell(String),
    ConnectServer,
    ConnectIrc(String),
    ConnectDiscord(String),
    SetCocategoryRatio(f64),
    GetCocategoryRatio,
    SetTravelDistance(i32),
    GetTravelDistance,
}

pub fn new() -> Iter {
    let (sender, receiver) = channel();
    let (socket_return, socket_receiver) = channel();
    Iter{
        _thread: spawn(move || {
            let mut context = zmq::Context::new();
            // Create pair socket which allows only one connection at a time
            let mut socket = context.socket(zmq::PAIR)
                .unwrap_or_else(|e| panic!("Error: Failed to open zmq socket: {}", e));
            // Attempt to bind ipc file to socket
            socket.bind(ipc::PATH).unwrap_or_else(|e| panic!("Error: Failed to connect to ipc file: {}", e));
            // Receive commands
            loop {
                match socket.recv_string(0) {
                    Ok(m) => {
                        match m {
                            Ok(s) => {
                                // Parse JSON into string vector
                                let v = match from_str::<Vec<String>>(&s) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        println!("Ignored: Unable to parse cli command from JSON: {}", e);
                                        continue;
                                    },
                                };
                                // Send the command vector along with the socket
                                match sender.send((v, SocketLend{
                                    socket: Some(socket),
                                    socket_return: socket_return.clone(),
                                })) {
                                    Ok(_) => {},
                                    Err(e) => panic!("Error: IPC thread unable to send: {}", e),
                                }
                            },
                            Err(_) => println!("Ignored: Unable to parse cli command from JSON"),
                        }
                    },
                    Err(e) => println!("Warning: Failed to get command: {}", e),
                }
                socket = socket_receiver.recv().unwrap_or_else(|e| panic!("Fatal: Unable to retrieve socket: {}", e));
            }
        }),
        receiver: receiver,
    }
}

pub struct Iter {
    _thread: JoinHandle<()>,
    receiver: Receiver<(Vec<String>, SocketLend)>,
}

impl Iterator for Iter {
    type Item = Option<(Decision, SocketLend)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.receiver.try_recv() {
            Ok((params, mut socket)) => {
                // let socket_fail = || panic!("Warning: Failed to respond to command");

                let help = |s: &mut SocketLend| {
                    s.msg("Available commands: quit, import, connect, list, respond, tell, get, set");
                };

                match params.len() {
                    0 => {
                        help(&mut socket);
                        Some(None)
                    },
                    _ => {
                        match &*params[0] {
                            "help" => {
                                help(&mut socket);
                                Some(None)
                            },
                            "quit" => {
                                if params.len() != 1 {
                                    socket.msg("Usage: quit");
                                    Some(None)
                                } else {
                                    Some(Some((Decision::Quit, socket)))
                                }
                            },
                            "import" => {
                                if params.len() < 2 {
                                    socket.msg("Usage: import <import type>");
                                    socket.msg("Available import types: lines");
                                    Some(None)
                                } else {
                                    match &*params[1] {
                                        "lines" => {
                                            if params.len() != 3 {
                                                socket.msg("Usage: import lines <filname>");
                                                Some(None)
                                            } else {
                                                socket.msg(&format!("Importing lines from `{}`...", params[2]));
                                                Some(Some((Decision::ImportLines(params[2].to_string()), socket)))
                                            }
                                        },
                                        _ => {
                                            socket.msg("Ignored: Unrecognized import type");
                                            Some(None)
                                        },
                                    }
                                }
                            },
                            "set" => {
                                if params.len() < 2 {
                                    socket.msg("Usage: set <value>");
                                    socket.msg("Values: cc_ratio, cc_travel");
                                    Some(None)
                                } else {
                                    match &*params[1] {
                                        "cc_ratio" => {
                                            if params.len() != 3 {
                                                socket.msg("Usage: set cc_ratio <ratio>");
                                                Some(None)
                                            } else {
                                                match params[2].parse::<f64>() {
                                                    Ok(f) => {
                                                        Some(Some((Decision::SetCocategoryRatio(f), socket)))
                                                    },
                                                    Err(e) => {
                                                        socket.msg(&format!("Ignored: Error converting value: {}\n", e));
                                                        Some(None)
                                                    },
                                                }
                                            }
                                        },
                                        "cc_travel" => {
                                            if params.len() != 3 {
                                                socket.msg("Usage: set cc_travel <steps>");
                                                Some(None)
                                            } else {
                                                match params[2].parse::<i32>() {
                                                    Ok(steps) => {
                                                        Some(Some((Decision::SetTravelDistance(steps), socket)))
                                                    },
                                                    Err(e) => {
                                                        socket.msg(&format!("Ignored: Error converting value: {}\n", e));
                                                        Some(None)
                                                    },
                                                }
                                            }
                                        },
                                        _ => {
                                            socket.msg("Ignored: Unrecognized set value");
                                            Some(None)
                                        },
                                    }
                                }
                            },
                            "get" => {
                                if params.len() < 2 {
                                    socket.msg("Usage: get <value>");
                                    socket.msg("Values: cc_ratio, cc_travel");
                                    Some(None)
                                } else {
                                    match &*params[1] {
                                        "cc_ratio" => {
                                            if params.len() != 2 {
                                                socket.msg("Usage: get cc_ratio");
                                                Some(None)
                                            } else {
                                                Some(Some((Decision::GetCocategoryRatio, socket)))
                                            }
                                        },
                                        "cc_travel" => {
                                            if params.len() != 2 {
                                                socket.msg("Usage: get cc_travel");
                                                Some(None)
                                            } else {
                                                Some(Some((Decision::GetTravelDistance, socket)))
                                            }
                                        },
                                        _ => {
                                            socket.msg("Ignored: Unrecognized get value");
                                            Some(None)
                                        },
                                    }
                                }
                            },
                            "connect" => {
                                if params.len() < 2 {
                                    socket.msg("Ignored: connect takes at least a connect type");
                                    socket.msg("Connect types: server, irc, discord");
                                    Some(None)
                                } else {
                                    match &*params[1] {
                                        "server" => {
                                            if params.len() != 2 {
                                                socket.msg("Ignored: no extra parameters needed");
                                                Some(None)
                                            } else {
                                                Some(Some((Decision::ConnectServer, socket)))
                                            }
                                        },
                                        "irc" => {
                                            if params.len() != 3 {
                                                socket.msg("Usage: connect irc <config>");
                                                Some(None)
                                            } else {
                                                Some(Some((Decision::ConnectIrc(params[2].to_string()), socket)))
                                            }
                                        },
                                        "discord" => {
                                            if params.len() != 3 {
                                                socket.msg("Usage: connect discord <config>");
                                                Some(None)
                                            } else {
                                                Some(Some((Decision::ConnectDiscord(params[2].to_string()), socket)))
                                            }
                                        },
                                        _ => {
                                            socket.msg("Ignored: Unrecognized connect type");
                                            Some(None)
                                        },
                                    }
                                }
                            },
                            "list" => {
                                if params.len() != 2 {
                                    socket.msg("Usage: list <list type>");
                                    socket.msg("Available list types: categories");
                                    Some(None)
                                } else {
                                    match &*params[1] {
                                        "categories" => {
                                            Some(Some((Decision::ShowCategories, socket)))
                                        },
                                        _ => {
                                            socket.msg("Ignored: Unrecognized list type");
                                            Some(None)
                                        },
                                    }
                                }
                            },
                            "respond" => {
                                if params.len() != 1 {
                                    socket.msg("Usage: respond");
                                    Some(None)
                                } else {
                                    Some(Some((Decision::Respond, socket)))
                                }
                            },
                            "tell" => {
                                if params.len() != 2 {
                                    socket.msg("Usage: tell <message>");
                                    Some(None)
                                } else {
                                    Some(Some((Decision::Tell(params[1].to_string()), socket)))
                                }
                            },
                            _ => {
                                socket.msg("Ignored: Unrecognized command");
                                Some(None)
                            },
                        }
                    }
                }
            },
            Err(TryRecvError::Empty) => {
                Some(None)
            },
            Err(TryRecvError::Disconnected) => {
                None
            },
        }
    }
}

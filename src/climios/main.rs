#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde;

extern crate serde_json;
use serde_json::ser::to_string;

extern crate itertools;
use itertools::Itertools;

extern crate zmq;

use std::env::args;
use std::path::*;

#[path = "../shared/ipc.rs"]
mod ipc;

fn main() {
    // Get zmq context
    let mut context = zmq::Context::new();

    // Open response socket for bidirectional operation
    let mut socket = match context.socket(zmq::PAIR) {
        Ok(s) => s,
        Err(e) => panic!("Error: Failed to open zmq socket: {}", e),
    };

    // Connect to the file
    match socket.connect(ipc::PATH) {
        Ok(_) => {},
        Err(e) => panic!("Error: Failed to connect to ipc file: {}", e),
    }

    // Make vector of strings
    let mut v = args().skip(1).map(|s| s.to_string()).collect_vec();

    match v.len() {
        0 => {},
        _ => {
            match v[0].as_str() {
                "import" => {
                    match v.len() {
                        1 => {}
                        _ => {
                            match v[1].as_str() {
                                "lines" => {
                                    if v.len() == 3 {
                                        let new_path = Path::new(&v[2])
                                            .canonicalize()
                                            .unwrap_or_else(|e| panic!("Error: Unable to canonicalize path: {}", e))
                                            .to_str()
                                            .unwrap_or_else(|| panic!("Error: Failed to convert path to string"))
                                            .to_string();
                                        v[2] = new_path;
                                    }
                                },
                                _ => {}
                            }
                        },
                    }
                },
                "connect" => {
                    match v.len() {
                        1 => {}
                        _ => {
                            match v[1].as_str() {
                                "irc" => {
                                    if v.len() == 3 {
                                        let new_path = Path::new(&v[2])
                                            .canonicalize()
                                            .unwrap_or_else(|e| panic!("Error: Unable to canonicalize path: {}", e))
                                            .to_str()
                                            .unwrap_or_else(|| panic!("Error: Failed to convert path to string"))
                                            .to_string();
                                        v[2] = new_path;
                                    }
                                },
                                "discord" => {
                                    if v.len() == 3 {
                                        let new_path = Path::new(&v[2])
                                            .canonicalize()
                                            .unwrap_or_else(|e| panic!("Error: Unable to canonicalize path: {}", e))
                                            .to_str()
                                            .unwrap_or_else(|| panic!("Error: Failed to convert path to string"))
                                            .to_string();
                                        v[2] = new_path;
                                    }
                                },
                                _ => {}
                            }
                        },
                    }
                },
                _ => {},
            }
        },
    }

    // Aquire JSON vector string from args
    let s = match to_string(&v) {
        Ok(s) => s,
        Err(e) => panic!("Error: JSON parsing failure: {}", e),
    };

    socket.send_str(&s, 0).unwrap_or_else(|e| panic!("Error: Unable to send JSON command vector: {}", e));

    loop {
        match socket.recv_string(0) {
            Ok(m) => match m {
                Ok(s) => match s.as_str() {
                    "" => break,
                    s => println!("{}", s),
                },
                Err(_) => println!("Warning: Got invalid string message"),
            },
            Err(e) => panic!("Error: Failed to get response: {}", e),
        }
    }
}

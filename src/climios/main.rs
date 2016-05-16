#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde;

extern crate serde_json;
use serde_json::ser::to_string;

extern crate itertools;
use itertools::Itertools;

extern crate zmq;

use std::env::args;

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

    // Aquire JSON vector string from args
    let s = match to_string(&args().skip(1).map(|s| s.to_string()).collect_vec()) {
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

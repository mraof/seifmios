extern crate serde_json;
use std::net::TcpListener;
use std::io::{BufReader, BufRead};
use std::thread;
use chat::ChatMessage;
use chat::ReplyMessage;
use std::sync::mpsc::Sender;

pub fn listen(sender: Sender<ReplyMessage>) {
    let listener = TcpListener::bind("127.0.0.1:2933").expect("Failed to bind port");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let sender = sender.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        let message: Result<ChatMessage, _> = serde_json::from_str(line.unwrap().as_str());
                        match message {
                            Ok(message) => {
                                println!("{:?}", message);
                                let _ = sender.send(ReplyMessage(message, None));
                            },
                            Err(e) => println!("Error parsing json: {:?}", e),
                        }
                    }
                });
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}
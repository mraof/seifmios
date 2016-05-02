extern crate rand;
extern crate crossbeam;

use std::sync::mpsc::sync_channel;
use std::env::args;
use std::fs::File;
use std::io::{BufReader, stdin, stdout};
use std::io::prelude::*;

mod text;
mod chat;

fn pause(message: &str) {
    let mut stdin = stdin();
    let mut stdout = stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "{}", message).unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();

    // Print newline
    println!("");
}

fn main() {
    crossbeam::scope(|scope| {
        use rand::SeedableRng;
        let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
        let nowhere = lex.source("nowhere".to_string());
        let nobody = lex.author(nowhere.clone(), "nobody".to_string());
        println!("Loading file to lexicon...");
        for arg in args().skip(1) {
            for (index, line) in BufReader::new(File::open(arg.clone()).ok().unwrap()).lines().enumerate() {
                lex.tell(nowhere.clone(), nobody.clone(), line.ok().unwrap());
                if index % 10000 == 0 {
                    println!("On line {} of {}", index, arg);
                }
            }
        }
        println!("Finished adding file to lexicon.");
        println!("Starting learning process.");
        loop {
            let (sender, receiver) = sync_channel(0);
            scope.spawn(move || {
                pause("Press any key to finish learning and print sample information...");
                sender.send(()).ok().unwrap();
            });
            while let Err(_) = receiver.try_recv() {
                lex.think();
            }
            lex.show_categories();
            for _ in 0..32 {
                println!("lex says: {}", lex.initiate(nowhere.clone()).unwrap());
            }
        }
    });
}

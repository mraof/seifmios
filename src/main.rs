extern crate rand;
extern crate crossbeam;

use std::io::{BufReader, BufRead, stdout, Write};
use std::fs::File;

mod text;
mod cli;

fn main() {
    use rand::SeedableRng;
    let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
    let console = lex.source("console".to_string());
    let me = lex.author(console.clone(), "me".to_string());
    print!(">");
    stdout().flush().unwrap();
    for decision in cli::new() {
        use cli::Decision;
        match decision {
            Decision::None => {
                lex.think();
            },
            Decision::ImportLines(filename) => {
                let author = lex.author(console.clone(), filename.clone());
                for (index, line) in BufReader::new(File::open(&filename).ok().unwrap()).lines().enumerate() {
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
                print!(">");
                stdout().flush().unwrap();
            },
            Decision::ShowCategories => {
                lex.show_categories();
                print!(">");
                stdout().flush().unwrap();
            },
            Decision::Respond => {
                if let Some(s) = lex.initiate(console.clone()) {
                    println!("{}", s);
                }
                print!(">");
                stdout().flush().unwrap();
            },
            Decision::Tell(s) => {
                lex.tell(console.clone(), me.clone(), s);
                print!(">");
                stdout().flush().unwrap();
            },
            Decision::ResetCursor => {
                print!(">");
                stdout().flush().unwrap();
            },
        }
    }
}

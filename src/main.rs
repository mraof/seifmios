extern crate rand;
extern crate crossbeam;

use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;

mod text;

fn main() {
    crossbeam::scope(|scope| {
        use rand::SeedableRng;
        let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
        let nowhere = lex.source("nowhere".to_string());
        let nobody = lex.author(nowhere.clone(), "nobody".to_string());
        lex.tell(nowhere.clone(), nobody.clone(), "nothing is good".to_string());
        lex.tell(nowhere.clone(), nobody.clone(), "everything is good".to_string());
        lex.tell(nowhere.clone(), nobody.clone(), "nothing is bad".to_string());
        let (sender, receiver) = sync_channel(0);
        scope.spawn(move || {
            thread::sleep(Duration::from_millis(5000));
            sender.send(()).ok().unwrap();
        });
        while let Err(_) = receiver.try_recv() {
            lex.think();
        }
        lex.show_categories();
        for _ in 0..32 {
            println!("lex says: {}", lex.initiate(nowhere.clone()).unwrap());
        }
    });
}

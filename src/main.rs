extern crate rand;
extern crate crossbeam;

mod text;
mod cli;

fn main() {
    use rand::SeedableRng;
    let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
    for decision in cli::new() {
        use cli::Decision;
        match decision {
            Decision::None => {
                lex.think();
            },
        }
    }
}

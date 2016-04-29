extern crate rand;

mod text;

fn main() {
    use rand::SeedableRng;
    let mut lex = text::Lexicon::new(rand::Isaac64Rng::from_seed(&[1, 2, 3, 4]));
    let nowhere = lex.source("nowhere".to_string());
    let nobody = lex.author(nowhere.clone(), "nobody".to_string());
    let message = lex.tell(nowhere.clone(), nobody, "nothing".to_string());
    assert!(message.is_none());
}

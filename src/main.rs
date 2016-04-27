mod text;

fn main() {
    let mut lex = text::Lexicon::default();
    let nowhere = lex.source("nowhere".to_string());
    let nobody = lex.author(nowhere.clone(), "nobody".to_string());
    let message = lex.tell(nowhere.clone(), nobody, "nothing".to_string());
    assert!(message.is_none());
}

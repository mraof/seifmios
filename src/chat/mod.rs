extern crate rand;
mod irc;
use text;

pub fn connect<R: rand::Rng>(lex: &mut text::Lexicon<R>)
{
    irc::connect("testing", lex);
}
extern crate rand;
mod irc;
use std::sync::mpsc::Sender;

pub fn connect(sender: Sender<(String, String, String, Option<Sender<String>>)>)
{
    irc::connect("testing", sender);
}
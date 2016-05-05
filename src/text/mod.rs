extern crate rand;
use rand::Rng;

#[macro_use]
mod pord_macros;

mod lexicon;
mod word_instance;
mod category;
mod message;
mod conversation;
mod author;
mod source;
mod word;

use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;

pub type WordCell = Rc<RefCell<Word>>;
pub type AuthorCell = Rc<RefCell<Author>>;
pub type SourceCell = Rc<RefCell<Source>>;
pub type MessageCell = Rc<RefCell<Message>>;
pub type CategoryCell = Rc<RefCell<Category>>;
pub type ConversationCell = Rc<RefCell<Conversation>>;
pub type InstanceCell = Rc<RefCell<WordInstance>>;

fn wrap<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}

pub enum Mismatch<T> {
    // Incompatible for matching or no mismatch (exactly the same)
    None,
    // Exactly one mismatch
    One(T),
    // Multiple mismatches
    Multiple,
}

#[derive(Default)]
pub struct Lexicon<R: Rng> {
    rng: R,
    words: BTreeMap<String, WordCell>,
    sources: BTreeMap<String, SourceCell>,
    conversations: Vec<ConversationCell>,
    messages: Vec<MessageCell>,

    active_conversations: BTreeMap<*const Source, ConversationCell>,
}

#[derive(Deserialize, Serialize)]
pub struct SerialLexicon {
    words: BTreeMap<String, u64>,
    sources: BTreeMap<String, u64>,
    conversations: Vec<u64>,
    messages: Vec<u64>,

    // Maps to look things up by unique ID
    conversation_map: BTreeMap<u64, SerialConversation>,
    author_map: BTreeMap<u64, SerialAuthor>,
    source_map: BTreeMap<u64, SerialSource>,
    word_instance_map: BTreeMap<u64, SerialWordInstance>,
    message_map: BTreeMap<u64, SerialMessage>,
    category_map: BTreeMap<u64, SerialCategory>,
    word_map: BTreeMap<u64, SerialWord>,
}

pub struct Conversation {
    source: SourceCell,
    messages: Vec<MessageCell>,
}

#[derive(Deserialize, Serialize)]
struct SerialConversation {
    source: u64,
    messages: Vec<u64>,
}

pub struct Author {
    source: SourceCell,
    name: String,
}

#[derive(Deserialize, Serialize)]
struct SerialAuthor {
    source: u64,
    name: String,
}

pub struct Source {
    name: String,
    messages: u64,
    authors: BTreeMap<String, AuthorCell>,
}

#[derive(Deserialize, Serialize)]
struct SerialSource {
    name: String,
    messages: u64,
    authors: BTreeMap<String, u64>,
}

pub struct WordInstance {
    word: WordCell,
    category: CategoryCell,
    message: MessageCell,
    index: usize,
}

#[derive(Deserialize, Serialize)]
struct SerialWordInstance {
    word: u64,
    category: u64,
    message: u64,
    index: u64,
}

pub struct Message {
    author: AuthorCell,
    conversation: ConversationCell,
    index: usize,
    instances: Vec<InstanceCell>,
}

#[derive(Deserialize, Serialize)]
struct SerialMessage {
    author: u64,
    conversation: u64,
    index: u64,
    instances: Vec<u64>,
}

#[derive(Default)]
pub struct Category {
    instances: Vec<InstanceCell>,
    cocategories: Vec<CategoryCell>,
}

#[derive(Deserialize, Serialize)]
struct SerialCategory {
    instances: Vec<u64>,
    cocategories: Vec<u64>,
}

pub struct Word {
    name: String,
    instances: Vec<InstanceCell>,
}

#[derive(Deserialize, Serialize)]
struct SerialWord {
    name: String,
    instances: Vec<u64>,
}

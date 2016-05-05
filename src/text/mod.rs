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

#[derive(Deserialize, Serialize, Default)]
pub struct SerialLexicon {
    words: BTreeMap<String, u64>,
    sources: BTreeMap<String, u64>,
    conversations: Vec<u64>,
    messages: Vec<u64>,

    // Maps to look things up by unique ID
    conversation_vec: Vec<SerialConversation>,
    author_vec: Vec<SerialAuthor>,
    source_vec: Vec<SerialSource>,
    word_instance_vec: Vec<SerialWordInstance>,
    message_vec: Vec<SerialMessage>,
    category_vec: Vec<SerialCategory>,
    word_vec: Vec<SerialWord>,
}

pub struct Conversation {
    source: SourceCell,
    messages: Vec<MessageCell>,
}

#[derive(Deserialize, Serialize, Default)]
struct SerialConversation {
    source: u64,
    messages: Vec<u64>,
}

pub struct Author {
    source: SourceCell,
    name: String,
}

#[derive(Deserialize, Serialize, Default)]
struct SerialAuthor {
    source: u64,
    name: String,
}

pub struct Source {
    name: String,
    messages: u64,
    authors: BTreeMap<String, AuthorCell>,
}

#[derive(Deserialize, Serialize, Default)]
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

#[derive(Deserialize, Serialize, Default)]
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

#[derive(Deserialize, Serialize, Default)]
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

#[derive(Deserialize, Serialize, Default)]
struct SerialCategory {
    instances: Vec<u64>,
    cocategories: Vec<u64>,
}

pub struct Word {
    name: String,
    instances: Vec<InstanceCell>,
}

#[derive(Deserialize, Serialize, Default)]
struct SerialWord {
    name: String,
    instances: Vec<u64>,
}

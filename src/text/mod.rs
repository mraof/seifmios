use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;

type WordCell = Rc<RefCell<Word>>;
type AuthorCell = Rc<RefCell<Author>>;
type SourceCell = Rc<RefCell<Source>>;
type MessageCell = Rc<RefCell<Message>>;
type CategoryCell = Rc<RefCell<Category>>;
type ConversationCell = Rc<RefCell<Conversation>>;

const RATIO_CONTAINED_BEFORE_COMBINATION: f64 = 0.8;

struct Lexicon {
    words: BTreeMap<String, WordCell>,
    authors: BTreeMap<String, AuthorCell>,
    sources: BTreeMap<String, SourceCell>,
    conversations: Vec<ConversationCell>,
}

struct Conversation {
    source: SourceCell,
    messages: Vec<Message>,
}

struct Author {
    name: String,
}

struct Source {
    name: String,
    messages: u64,
}

struct MessageWord {
    word: WordCell,
    category: CategoryCell,
}

struct Message {
    author: AuthorCell,
    conversation: ConversationCell,
    index: usize,
    words: Vec<MessageWord>,
}

struct WordInstance {
    word: WordCell,
    message: MessageCell,
    index: usize,
}

struct Category {
    instances: Vec<WordInstance>,
    subcategories: Vec<CategoryCell>,
}

struct Word {
    name: String,
    instances: Vec<WordInstance>,
}

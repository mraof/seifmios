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
    words: BTreeMap<&'static str, WordCell>,
    authors: BTreeMap<&'static str, AuthorCell>,
    sources: BTreeMap<&'static str, SourceCell>,
    conversations: Vec<ConversationCell>,

    active_conversations: BTreeMap<*const Source, ConversationCell>,
}

impl Lexicon {
    fn tell(source: SourceCell, author: &str, message: &str) -> Option<String> {
        None
    }
}

struct Conversation {
    source: SourceCell,
    messages: Vec<MessageCell>,
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

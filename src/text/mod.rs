use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;
use std::cell::RefCell;

type WordCell = Rc<RefCell<Word>>;
type AuthorCell = Rc<RefCell<Author>>;
type SourceCell = Rc<RefCell<Source>>;
type MessageCell = Rc<RefCell<Message>>;
type CategoryCell = Rc<RefCell<Category>>;
type ConversationCell = Rc<RefCell<Conversation>>;

fn wrap<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}

const RATIO_CONTAINED_BEFORE_COMBINATION: f64 = 0.8;

struct Lexicon {
    words: BTreeMap<String, WordCell>,
    authors: BTreeMap<String, AuthorCell>,
    sources: BTreeMap<String, SourceCell>,
    conversations: Vec<ConversationCell>,

    active_conversations: BTreeMap<*const Source, ConversationCell>,
}

impl Lexicon {
    fn tell(&mut self, source: SourceCell, author: String, message: String) -> Option<String> {
        let author = match self.authors.entry(author.clone()) {
            Entry::Vacant(v) => {
                let a = wrap(Author{
                    name: author.clone(),
                });
                v.insert(a.clone());
                a
            },
            Entry::Occupied(o) => o.get().clone(),
        };
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
    supercategory: Option<CategoryCell>,
    instances: Vec<WordInstance>,
    subcategories: Vec<CategoryCell>,
}

struct Word {
    name: String,
    instances: Vec<WordInstance>,
}

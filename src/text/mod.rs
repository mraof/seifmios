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
    fn tell(&mut self, source: SourceCell, author: String, content: String) -> Option<String> {
        let conversation = match self.active_conversations.entry(&*source.borrow() as *const Source) {
            Entry::Vacant(v) => {
                let c = wrap(Conversation{
                    source: source.clone(),
                    messages: Vec::new(),
                });
                v.insert(c.clone());
                c
            },
            Entry::Occupied(o) => {
                o.get().clone()
            },
        };

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

        let message = wrap(Message{
            author: author.clone(),
            conversation: conversation.clone(),
            index: conversation.borrow().messages.len(),
            words: Vec::new(),
        });

        let words = content.split(' ').map(|s| {
            let string = s.to_string();
            match self.words.entry(string.clone()) {
                Entry::Vacant(v) => {
                    let mut w = wrap(Word{
                        name: string,
                        instances: Vec::new(),
                    });
                    v.insert(w.clone());
                    w
                },
                Entry::Occupied(o) => {
                    o.get().clone()
                },
            }
        });

        // Add words to the message and add instances to the words
        {
            // TODO: it
        }

        // TODO: Determine what to say back
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

struct WordInstance {
    word: WordCell,
    category: CategoryCell,
    message: MessageCell,
    index: usize,
}

struct Message {
    author: AuthorCell,
    conversation: ConversationCell,
    index: usize,
    words: Vec<WordInstance>,
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

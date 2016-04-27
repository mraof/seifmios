use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;
use std::cell::RefCell;

type WordCell = Rc<RefCell<Word>>;
pub type AuthorCell = Rc<RefCell<Author>>;
pub type SourceCell = Rc<RefCell<Source>>;
type MessageCell = Rc<RefCell<Message>>;
type CategoryCell = Rc<RefCell<Category>>;
type ConversationCell = Rc<RefCell<Conversation>>;
type InstanceCell = Rc<RefCell<WordInstance>>;

fn wrap<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}

// const RATIO_CONTAINED_BEFORE_COMBINATION: f64 = 0.8;

#[derive(Default)]
pub struct Lexicon {
    words: BTreeMap<String, WordCell>,
    sources: BTreeMap<String, SourceCell>,
    conversations: Vec<ConversationCell>,

    active_conversations: BTreeMap<*const Source, ConversationCell>,
}

impl Lexicon {
    pub fn source(&mut self, name: String) -> SourceCell {
        match self.sources.entry(name.clone()) {
            Entry::Vacant(v) => v.insert(wrap(Source{
                name: name,
                messages: 0,
                authors: BTreeMap::default(),
            })).clone(),
            Entry::Occupied(o) => o.get().clone(),
        }
    }
    pub fn author(&mut self, source: SourceCell, name: String) -> AuthorCell {
        match source.borrow_mut().authors.entry(name.clone()) {
            Entry::Vacant(v) => {
                let a = wrap(Author{
                    source: source.clone(),
                    name: name.clone(),
                });
                v.insert(a.clone());
                a
            },
            Entry::Occupied(o) => o.get().clone(),
        }
    }
    pub fn tell(&mut self, source: SourceCell, author: AuthorCell, content: String) -> Option<String> {
        let conversation = match self.active_conversations.entry(&*source.borrow() as *const Source) {
            Entry::Vacant(v) => {
                let c = wrap(Conversation{
                    source: source.clone(),
                    messages: Vec::new(),
                });
                v.insert(c.clone());
                self.conversations.push(c.clone());
                c
            },
            Entry::Occupied(o) => {
                o.get().clone()
            },
        };

        let message = wrap(Message{
            author: author.clone(),
            conversation: conversation.clone(),
            index: conversation.borrow().messages.len(),
            words: Vec::new(),
        });

        for word in content.split(' ').map(|s| {
            let string = s.to_string();
            match self.words.entry(string.clone()) {
                Entry::Vacant(v) => {
                    let w = wrap(Word{
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
        }) {
            // Create empty category for the word
            let category = wrap(Category::default());
            // Create instance of the word
            let instance = wrap(WordInstance{
                word: word.clone(),
                category: category.clone(),
                message: message.clone(),
                index: message.borrow().words.len(),
            });
            // Insert word instance into the word, category, and message for future reference
            message.borrow_mut().words.push(instance.clone());
            category.borrow_mut().instances.push(instance.clone());
            word.borrow_mut().instances.push(instance);
        }

        // Increment the messages by 1 for the source
        source.borrow_mut().messages += 1;

        // TODO: Determine what to say back
        None
    }
}

struct Conversation {
    source: SourceCell,
    messages: Vec<MessageCell>,
}

pub struct Author {
    source: SourceCell,
    name: String,
}

pub struct Source {
    name: String,
    messages: u64,
    authors: BTreeMap<String, AuthorCell>,
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
    words: Vec<InstanceCell>,
}

#[derive(Default)]
struct Category {
    supercategory: Option<CategoryCell>,
    instances: Vec<InstanceCell>,
    subcategories: Vec<CategoryCell>,
}

struct Word {
    name: String,
    instances: Vec<InstanceCell>,
}

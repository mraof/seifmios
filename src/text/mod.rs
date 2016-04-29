extern crate rand;
extern crate itertools;
use self::itertools::Itertools;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::mpsc::Receiver;

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
pub struct Lexicon<R: rand::Rng> {
    rng: R,
    words: BTreeMap<String, WordCell>,
    sources: BTreeMap<String, SourceCell>,
    conversations: Vec<ConversationCell>,
    messages: Vec<MessageCell>,

    active_conversations: BTreeMap<*const Source, ConversationCell>,
}

impl<R: rand::Rng> Lexicon<R> {
    pub fn new(rng: R) -> Lexicon<R> {
        Lexicon{
            rng: rng,
            words: Default::default(),
            sources: Default::default(),
            conversations: Default::default(),
            messages: Default::default(),
            active_conversations: Default::default(),
        }
    }

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
            instances: Vec::new(),
        });

        self.messages.push(message.clone());

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
                index: message.borrow().instances.len(),
            });
            // Insert word instance into the word, category, and message for future reference
            message.borrow_mut().instances.push(instance.clone());
            category.borrow_mut().instances.push(instance.clone());
            word.borrow_mut().instances.push(instance);
        }

        // Increment the messages by 1 for the source
        source.borrow_mut().messages += 1;

        // TODO: Determine what to say back
        None
    }

    pub fn initiate(&mut self, source: SourceCell) -> Option<String> {
        let conversation = wrap(Conversation{
            source: source.clone(),
            messages: Vec::new(),
        });

        match self.active_conversations.entry(&*source.borrow() as *const Source) {
            Entry::Vacant(v) => {
                v.insert(conversation.clone());
            },
            Entry::Occupied(mut o) => {
                o.insert(conversation.clone());
            },
        };

        self.conversations.push(conversation.clone());

        let base = match self.rng.choose(&self.messages[..]) {
            Some(m) => m.clone(),
            None => return None,
        };

        let bborrow = base.borrow();

        Some(bborrow.instances.iter()
            .map(|instance| instance.borrow().category.clone())
            // TODO: Allow random choosing from co-category instances as well
            .map(|category| self.rng.choose(&category.borrow().instances[..]).unwrap().clone())
            .map(|instance| {
                let ins = instance.borrow();
                let word = ins.word.borrow();
                word.name.clone()
            })
            .join(" "))
    }

    pub fn think(&mut self, end: Receiver<()>) {
        use std::sync::mpsc::TryRecvError::Empty;
        while end.try_recv() == Err(Empty) {
            // Choose a random message or break if there are none
            let message = match self.rng.choose(&self.messages[..]) {
                Some(m) => m.clone(),
                None => break,
            };

            enum Amount<T> {
                None,
                One(T),
                Multiple,
            }

            // Vector of absolute perfect matches
            let mut vones = Vec::new();

            // Look through each word in the message
            for word in &message.borrow().instances {
                let borrow = word.borrow();
                // Check each instance in that words instances
                for instance in &borrow.word.borrow().instances {
                    // Get the message for each instance
                    let omessage = instance.borrow().message.clone();
                    // Make sure we dont get the same message twice and assure the same length
                    if omessage != message &&
                        omessage.borrow().instances.len() == message.borrow().instances.len() {
                        // Find what kind of matches exist between the messages
                        if let Amount::One(best) = message.borrow().instances.iter()
                            .zip(omessage.borrow().instances.iter())
                            .fold(Amount::None, |acc, ws| {
                                let bs = (ws.0.borrow(), ws.1.borrow());
                                match acc {
                                    Amount::None => {
                                        if bs.0.word.borrow().name !=
                                            bs.1.word.borrow().name {
                                            Amount::One((ws.0.clone(), ws.1.clone()))
                                        } else {
                                            Amount::None
                                        }
                                    },
                                    Amount::One(best) => {
                                        if bs.0.word.borrow().name !=
                                            bs.1.word.borrow().name {
                                            Amount::Multiple
                                        } else {
                                            Amount::One(best)
                                        }
                                    },
                                    Amount::Multiple => Amount::Multiple,
                                }
                            }) {
                            vones.push(best);
                        }
                    }
                }
            }

            // Now that we have perfect matches, merge them into the same Category
            for ms in vones {
                // We only want to combine if they aren't already in the same category
                if ms.0.borrow().category != ms.1.borrow().category {
                    let cats = (ms.0.borrow().category.clone(), ms.1.borrow().category.clone());
                    Category::merge(cats);
                }
            }
        }
    }

    pub fn show_categories(&self) {
        use std::collections::BTreeSet;
        let mut set = BTreeSet::new();
        for message in &self.messages {
            for instance in &message.borrow().instances {
                let ib = instance.borrow();
                set.insert(&*ib.category.borrow() as *const Category);
            }
        }

        for cat in set {
            println!("Category:");
            let catr = unsafe{&*cat};
            for instance in &catr.instances {
                let ib = instance.borrow();
                println!("\t{}", ib.word.borrow().name);
            }
        }
    }
}

macro_rules! pointer_eq {
    ($s:ident) => {
        impl PartialEq for $s {
            fn eq(&self, other: &Self) -> bool {
                self as *const Self == other as *const Self
            }

            fn ne(&self, other: &Self) -> bool {
                self as *const Self != other as *const Self
            }
        }
    };
}

struct Conversation {
    source: SourceCell,
    messages: Vec<MessageCell>,
}

pointer_eq!(Conversation);

pub struct Author {
    source: SourceCell,
    name: String,
}

pointer_eq!(Author);

pub struct Source {
    name: String,
    messages: u64,
    authors: BTreeMap<String, AuthorCell>,
}

pointer_eq!(Source);

struct WordInstance {
    word: WordCell,
    category: CategoryCell,
    message: MessageCell,
    index: usize,
}

pointer_eq!(WordInstance);

struct Message {
    author: AuthorCell,
    conversation: ConversationCell,
    index: usize,
    instances: Vec<InstanceCell>,
}

pointer_eq!(Message);

#[derive(Default)]
struct Category {
    instances: Vec<InstanceCell>,
    cocategories: Vec<CategoryCell>,
}

impl Category {
    fn merge(cs: (CategoryCell, CategoryCell)) -> CategoryCell {
        let cc = wrap(Self::default());
        {
            let clos = |cat: &CategoryCell| {
                for instance in cat.borrow().instances.iter().cloned() {
                    instance.borrow_mut().category = cc.clone();
                }
            };
            clos(&cs.0);
            clos(&cs.1);
            let mut ccm = cc.borrow_mut();
            ccm.instances.append(&mut cs.0.borrow_mut().instances);
            ccm.instances.append(&mut cs.1.borrow_mut().instances);
        }
        cc
    }
}

pointer_eq!(Category);

struct Word {
    name: String,
    instances: Vec<InstanceCell>,
}

pointer_eq!(Word);

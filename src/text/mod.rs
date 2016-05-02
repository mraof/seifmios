extern crate rand;
extern crate itertools;
use self::itertools::Itertools;

use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Entry;
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;

use std::fmt;

const RATIO_TO_COCATEGORIZE: f64 = 0.8;

pub type WordCell = Rc<RefCell<Word>>;
pub type AuthorCell = Rc<RefCell<Author>>;
pub type SourceCell = Rc<RefCell<Source>>;
pub type MessageCell = Rc<RefCell<Message>>;
pub type CategoryCell = Rc<RefCell<Category>>;
pub type ConversationCell = Rc<RefCell<Conversation>>;
pub type InstanceCell = Rc<RefCell<WordInstance>>;

enum Mismatch<T> {
    // Incompatible for matching or no mismatch (exactly the same)
    None,
    // Exactly one mismatch
    One(T),
    // Multiple mismatches
    Multiple,
}

fn wrap<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}

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
    /// Make a new lexion. It needs its own Rng for internal purposes of learning.
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

    /// Get a source by its unique name.
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

    /// Get an author identifier from a particular source.
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

    /// Tell a message to the lexicon and potentially get a response back.
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

        // Learn the message immediately
        self.learn(message);

        // TODO: Determine what to say back
        None
    }

    /// Have seifmios attempt to initiate a conversation at a source, but it may fail.
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

        Some(
            bborrow.instances.iter()
                .map(|instance| instance.borrow().category.clone())
                // TODO: Allow random choosing from co-category instances as well
                .map(|category| {
                    let b = category.borrow();
                    // Find the count of how many word instances exist total
                    let mut count = b.instances.len();
                    for cocategory in &b.cocategories {
                        count += cocategory.borrow().instances.len();
                    }
                    // Generate an index based on the count
                    let mut i = self.rng.gen_range(0, count);
                    match b.instances.get(i) {
                        // It was in the original category
                        Some(ins) => ins.clone(),
                        // It was in a cocategory
                        None => {
                            // Subtract the cocategory length from the index
                            i -= b.instances.len();
                            for cocategory in &b.cocategories {
                                let b = cocategory.borrow();
                                // If it was in this category
                                if let Some(ins) = b.instances.get(i) {
                                    // Clone the instance and return it
                                    return ins.clone();
                                }
                                // Otherwise subtract by the amount of instances in this cocategory
                                i -= b.instances.len();
                            }
                            // The index should point to some category, so this is unreachable
                            unreachable!();
                        },
                    }
                })
                .map(|instance| {
                    let ins = instance.borrow();
                    let word = ins.word.borrow();
                    word.name.clone()
                })
                .join(" ")
        )
    }

    /// Thinks one iteration
    pub fn think(&mut self) {
        // Learn a random message if there are some
        let m = match self.rng.choose(&self.messages[..]) {
            Some(m) => m.clone(),
            None => return,
        };

        self.learn(m);

        // Get two random categories (we already know messages exist from above)
        Category::cocategorize((
            {
                let b = self.rng.choose(&self.messages[..]).unwrap().borrow();
                let b = self.rng.choose(&b.instances[..]).unwrap().borrow();
                b.category.clone()
            },
            {
                let b = self.rng.choose(&self.messages[..]).unwrap().borrow();
                let b = self.rng.choose(&b.instances[..]).unwrap().borrow();
                b.category.clone()
            },
        ));
    }

    pub fn learn(&mut self, message: MessageCell) {
        // Vector of absolute perfect matches
        let mut vones = Vec::new();

        // Look through each word in the message
        for word in &message.borrow().instances {
            let borrow = word.borrow();
            // Check each instance in that words instances
            for instance in &borrow.word.borrow().instances {
                // Get the message for each instance
                let omessage = instance.borrow().message.clone();
                // Find what kind of matches exist between the messages
                if let Mismatch::One(best) =
                    Message::category_and_word_mismatch((message.clone(), omessage.clone())) {
                    vones.push(best);
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

    /// Print all multiple categories and return the amount of categories total
    pub fn show_categories(&self) -> usize {
        let mut set = BTreeSet::new();
        for message in &self.messages {
            for instance in &message.borrow().instances {
                let ib = instance.borrow();
                set.insert(&*ib.category.borrow() as *const Category);
            }
        }

        let len = set.len();

        for cat in set {
            let catr = unsafe{&*cat};
            if catr.instances.len() != 1 {
                println!("Category:");
                for cocategory in &catr.cocategories {
                    println!("\tCocategory:");
                    let catr = cocategory.borrow();
                    for instance in &catr.instances {
                        let ib = instance.borrow();
                        println!("\t\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow());
                    }
                }
                for instance in &catr.instances {
                    let ib = instance.borrow();
                    println!("\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow());
                }
            }
        }
        len
    }
}

macro_rules! pointer_ord {
    ($s:ident) => {
        // Implement on the type
        impl PartialEq for $s {
            fn eq(&self, other: &Self) -> bool {
                self as *const Self == other as *const Self
            }

            fn ne(&self, other: &Self) -> bool {
                self as *const Self != other as *const Self
            }
        }

        impl Eq for $s {}

        impl PartialOrd for $s {
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                (self as *const Self).partial_cmp(&(other as *const Self))
            }

            fn lt(&self, other: &Self) -> bool {
                (self as *const Self) < (other as *const Self)
            }

            fn le(&self, other: &Self) -> bool {
                self as *const Self <= other as *const Self
            }

            fn gt(&self, other: &Self) -> bool {
                self as *const Self > other as *const Self
            }

            fn ge(&self, other: &Self) -> bool {
                self as *const Self >= other as *const Self
            }
        }

        impl Ord for $s {
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                (self as *const Self).cmp(&(other as *const Self))
            }
        }
    };
}

pub struct Conversation {
    source: SourceCell,
    messages: Vec<MessageCell>,
}

pointer_ord!(Conversation);

pub struct Author {
    source: SourceCell,
    name: String,
}

pointer_ord!(Author);

pub struct Source {
    name: String,
    messages: u64,
    authors: BTreeMap<String, AuthorCell>,
}

pointer_ord!(Source);

pub struct WordInstance {
    word: WordCell,
    category: CategoryCell,
    message: MessageCell,
    index: usize,
}

impl WordInstance {
    fn coincidence_level(ins: (&InstanceCell, &InstanceCell)) -> usize {
        let bs = (ins.0.borrow(), ins.1.borrow());
        let ms = (bs.0.message.borrow(), bs.1.message.borrow());
        for i in 1.. {
            let msins = (
                (ms.0.instances.get(bs.0.index - i), ms.1.instances.get(bs.1.index - i)),
                (ms.0.instances.get(bs.0.index + i), ms.1.instances.get(bs.1.index + i))
            );

            match msins.0 {
                // There are two words
                (Some(i0), Some(i1)) => {
                    // If both the categories and words don't match
                    if !Category::are_cocategories((&i0.borrow().category, &i1.borrow().category))
                        && i0.borrow().word != i1.borrow().word {
                        // The coincidence level doesn't go this far
                        return i-1;
                    }
                },
                // The sentence ends in both spots
                (None, None) => {
                    // We can't go any further, but we also need to check the right instances
                    match msins.1 {
                        (Some(i0), Some(i1)) => {
                            // If both the categories and words don't match
                            if !Category::are_cocategories(
                                (&i0.borrow().category, &i1.borrow().category))
                                && i0.borrow().word != i1.borrow().word {
                                // The coincidence level doesn't go this far
                                return i-1;
                            }
                        },
                        (None, None) => {
                            // We can't go any further on either side, so this is the coincidence
                            return i;
                        },
                        _ => {
                            // Any combination of Some and None is a mismatch
                            return i-1;
                        }
                    }
                }
                _ => {
                    // Any combination of Some and None is a mismatch
                    return i-1;
                }
            }
        }
        unreachable!();
    }
}

pointer_ord!(WordInstance);

pub struct Message {
    author: AuthorCell,
    conversation: ConversationCell,
    index: usize,
    instances: Vec<InstanceCell>,
}

impl Message {
    fn string(&self) -> String {
        self.instances.iter()
            .map(|i| {
                let b = i.borrow();
                let b = b.word.borrow();
                b.name.clone()
            })
            .join(" ")
    }
    fn mismatch<F>(messages: (MessageCell, MessageCell), diff: F) -> Mismatch<(InstanceCell, InstanceCell)>
        where F: Fn((&WordInstance, &WordInstance)) -> bool
    {
        if messages.0 == messages.1 ||
            messages.0.borrow().instances.len() != messages.1.borrow().instances.len() {
            return Mismatch::None;
        }
        messages.0.borrow().instances.iter()
            .zip(messages.1.borrow().instances.iter())
            .fold(Mismatch::None, |acc, ws| {
                let bs = (ws.0.borrow(), ws.1.borrow());
                match acc {
                    Mismatch::None => {
                        if diff((&*bs.0, &*bs.1)) {
                            Mismatch::One((ws.0.clone(), ws.1.clone()))
                        } else {
                            Mismatch::None
                        }
                    },
                    Mismatch::One(best) => {
                        if diff((&*bs.0, &*bs.1)) {
                            Mismatch::Multiple
                        } else {
                            Mismatch::One(best)
                        }
                    },
                    Mismatch::Multiple => Mismatch::Multiple,
                }
            })
    }
    /*fn category_mismatch(messages: (MessageCell, MessageCell)) ->
        Mismatch<(InstanceCell, InstanceCell)> {
        Self::mismatch(messages, |ins| ins.0.category != ins.1.category)
    }

    fn word_mismatch(messages: (MessageCell, MessageCell)) ->
        Mismatch<(InstanceCell, InstanceCell)> {
        Self::mismatch(messages, |ins| ins.0.word != ins.1.word)
    }*/

    fn category_and_word_mismatch(messages: (MessageCell, MessageCell)) ->
        Mismatch<(InstanceCell, InstanceCell)> {
        Self::mismatch(messages, |ins| ins.0.word != ins.1.word && ins.0.category != ins.1.category)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string())
    }
}

pointer_ord!(Message);

#[derive(Default)]
pub struct Category {
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
        // TODO: Save all the cocategories from before and cocategorize them with this category
        // except for the categories we just removed of course
        cc
    }

    /// Determine if the categories should be cocategories
    fn cocategorize(cs: (CategoryCell, CategoryCell)) {
        // First, check to see if they are the same category
        if cs.0 == cs.1 {
            // Nothing to do in that case
            return;
        }

        // Get the total amount of instances in cs.0
        let total = cs.0.borrow().instances.len();
        // Make a counter to see how many instances coincide
        let mut coincidences = 0;

        // Look through all the instances between both categories
        {
            let bs = (cs.0.borrow(), cs.1.borrow());
            for i0 in bs.0.instances.iter() {
                // We see if there is any coincidence for this instance
                for i1 in bs.1.instances.iter() {
                    // It is impossible for two different categories to contain the same instance,
                    // so that doesn't need to be checked for.

                    // TODO: Look behind and ahead by more than just 1 instance
                    // Check if the coincidence level is at least 1 (for now)
                    if WordInstance::coincidence_level((i0, i1)) >= 1 {
                        // Increment the amount of coincidences
                        coincidences += 1;
                        // Break so we dont count any more
                        break;
                    }
                }
            }
        }

        // If the amount of coincidences is sufficient enough
        if coincidences as f64 / total as f64 > RATIO_TO_COCATEGORIZE {
            // Make these cocategories

            // Check if they are already cocategories
            if cs.0.borrow().cocategories.contains(&cs.1) {
                // Then we are done
                return;
            }

            // Add it
            cs.0.borrow_mut().cocategories.push(cs.1.clone());
            cs.1.borrow_mut().cocategories.push(cs.0.clone());
        } else {
            // Unmake these cocategories

            // Find position in vector of cocategories
            let csp0 = cs.0.borrow().cocategories.iter().position(|i| *i == cs.1);
            // If it wasnt found
            if csp0.is_none() {
                // Then we are done
                return;
            }
            let csp1 = cs.1.borrow().cocategories.iter().position(|i| *i == cs.0);

            // Remove it
            cs.0.borrow_mut().cocategories.swap_remove(csp0.unwrap());
            // In the case this panics, two cocategories didnt contain each other (bad stuff!)
            cs.1.borrow_mut().cocategories.swap_remove(csp1.unwrap());
        }
    }

    fn are_cocategories(cs: (&CategoryCell, &CategoryCell)) -> bool {
        // If they are the same category they are cocategories
        if cs.0 == cs.1 {
            true
        } else {
            cs.0.borrow().cocategories.contains(&cs.1)
        }
    }
}

pointer_ord!(Category);

pub struct Word {
    name: String,
    instances: Vec<InstanceCell>,
}

pointer_ord!(Word);

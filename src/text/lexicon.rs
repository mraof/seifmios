extern crate rand;
extern crate itertools;
use self::itertools::Itertools;
use super::*;
use super::wrap;

use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Entry;

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
    pub fn tell(&mut self, source: SourceCell, author: AuthorCell, content: String) {
        let conversation = match self.active_conversations.entry(source.clone()) {
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
    }

    /// Switch conversations
    pub fn switch(&mut self, source: SourceCell) {
        let conversation = wrap(Conversation{
            source: source.clone(),
            messages: Vec::new(),
        });

        match self.active_conversations.entry(source.clone()) {
            Entry::Vacant(v) => {
                v.insert(conversation.clone());
            },
            Entry::Occupied(mut o) => {
                o.insert(conversation.clone());
            },
        };

        self.conversations.push(conversation.clone());
    }

    /// Say something based on the conversation context
    pub fn respond(&mut self, source: SourceCell) -> Option<String> {
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

    /// Have seifmios attempt to initiate a conversation at a source, but it may fail.
    pub fn initiate(&mut self, source: SourceCell) -> Option<String> {
        self.switch(source.clone());
        self.respond(source)
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
                set.insert(ib.category.clone());
            }
        }

        let len = set.len();

        for cat in set {
            let catr = cat.borrow();
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

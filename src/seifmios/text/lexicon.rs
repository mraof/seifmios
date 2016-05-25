extern crate rand;
extern crate itertools;
use self::itertools::Itertools;
use super::*;
use super::wrap;

use super::super::cli::SocketLend;

use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Entry;

const RATIO_TO_COCATEGORIZE: f64 = 0.4;
const COCATEGORY_TRAVEL_DISTANCE: i32 = 0;
const COCATEGORIZE_MAGNITUDE: i32 = 65536;
const FORWARD_EDGE_DISTANCE: usize = 1;
const BACKWARD_EDGE_DISTANCE: usize = 1;
const FORWARD_WORD_DISTANCE: usize = 1;
const BACKWARD_WORD_DISTANCE: usize = 1;

impl<R: rand::Rng> Lexicon<R> {
    /// Make a new lexion. It needs its own Rng for internal purposes of learning.
    pub fn new(rng: R) -> Lexicon<R> {
        Lexicon{
            rng: rng,
            cocategorization_ratio: RATIO_TO_COCATEGORIZE,
            cocategory_travel_distance: COCATEGORY_TRAVEL_DISTANCE,
            cocategorize_magnitude: COCATEGORIZE_MAGNITUDE,
            forward_edge_distance: FORWARD_EDGE_DISTANCE,
            backward_edge_distance: BACKWARD_EDGE_DISTANCE,
            forward_word_distance: FORWARD_WORD_DISTANCE,
            backward_word_distance: BACKWARD_WORD_DISTANCE,
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
            last_checked_at: 0,
            author: author.clone(),
            conversation: conversation.clone(),
            index: conversation.borrow().messages.len(),
            instances: Vec::new(),
        });

        self.messages.push(message.clone());

        // Add message to conversation
        {
            let mut cb = conversation.borrow_mut();
            cb.messages.push(message.clone());
        }

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
    pub fn respond(&mut self, source: SourceCell) -> Option<(String, String)> {
        use std::collections::VecDeque;
        let base = match self.rng.choose(&self.messages[..]) {
            Some(m) => m.clone(),
            None => return None,
        };

        // Make a double-ended vec for building the message out of categories
        let mut instances = VecDeque::new();

        let mut orig_index = if let Some(con) = self.active_conversations.get(&source) {
            let cb = con.borrow();
            if let Some(conm) = cb.messages.last() {
                let mb = conm.borrow();
                instances.push_back(self.rng.choose(&mb.instances[..]).unwrap().clone());
                0
            } else {
                instances.push_back(self.rng.choose(&base.borrow().instances[..]).unwrap().clone());
                8192
            }
        } else {
            instances.push_back(self.rng.choose(&base.borrow().instances[..]).unwrap().clone());
            8192
        };

        let forward_instance_chooser = |b: &Category, rng: &mut R| {
            // Find the count of how many word instances exist total
            let mut count = b.instances.len();
            for cocategory in &b.precocategories {
                count += cocategory.borrow().instances.len();
            }
            // Generate an index based on the count
            let mut i = rng.gen_range(0, count);
            match b.instances.get(i) {
                // It was in the original category
                Some(ins) => ins.clone(),
                // It was in a cocategory
                None => {
                    // Subtract the cocategory length from the index
                    i -= b.instances.len();
                    for cocategory in &b.precocategories {
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
        };

        let backward_instance_chooser = |b: &Category, rng: &mut R| {
            // Find the count of how many word instances exist total
            let mut count = b.instances.len();
            for cocategory in &b.postcocategories {
                count += cocategory.borrow().instances.len();
            }
            // Generate an index based on the count
            let mut i = rng.gen_range(0, count);
            match b.instances.get(i) {
                // It was in the original category
                Some(ins) => ins.clone(),
                // It was in a cocategory
                None => {
                    // Subtract the cocategory length from the index
                    i -= b.instances.len();
                    for cocategory in &b.postcocategories {
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
        };

        // Iterate forwards weaving between messages and adding instances to the vec
        loop {
            let ins = instances.back().unwrap().borrow().next_instance();
            if let Some(i) = ins {
                let b = i.borrow();
                instances.push_back(forward_instance_chooser(&b.category.borrow(), &mut self.rng));
            } else {
                break;
            }
        }
        // Iterate backwards to reach the beginning of the message
        loop {
            let ins = instances.front().unwrap().borrow().prev_instance();
            if let Some(i) = ins {
                let b = i.borrow();
                instances.push_front(backward_instance_chooser(&b.category.borrow(), &mut self.rng));
                orig_index += 1;
            } else {
                break;
            }
        }

        Some((
            instances.iter()
                .map(|instance| {
                    let ins = instance.borrow();
                    let word = ins.word.borrow();
                    word.name.clone()
                })
                .join(" "),
            instances.iter()
                .cloned()
                .enumerate()
                .map(|(index, mut instance)| {
                    if index != orig_index {
                        for _ in 0..self.cocategory_travel_distance {
                            instance = {
                                let b = instance.borrow();
                                if self.rng.gen_range(0, 2) == 0 {
                                    backward_instance_chooser(&b.category.borrow(), &mut self.rng)
                                } else {
                                    forward_instance_chooser(&b.category.borrow(), &mut self.rng)
                                }
                            };
                        }
                    }
                    instance
                })
                .map(|instance| {
                    let ins = instance.borrow();
                    let word = ins.word.borrow();
                    word.name.clone()
                })
                .join(" ")
        ))
    }

    /// Have seifmios attempt to initiate a conversation at a source, but it may fail.
    pub fn initiate(&mut self, source: SourceCell) -> Option<(String, String)> {
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

        self.learn(m.clone());
    }

    pub fn learn(&mut self, message: MessageCell) {
        // Only attempt to learn category if it hasn't been learned as of last message
        if message.borrow().last_checked_at != self.messages.len() {
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

            message.borrow_mut().last_checked_at = self.messages.len();
        }

        for _ in 0..self.cocategorize_magnitude {
            // Get two random categories (we already know messages exist from above)
            Category::cocategorize(
                (
                    {
                        let b = message.borrow();
                        let b = self.rng.choose(&b.instances[..]).unwrap().borrow();
                        b.category.clone()
                    },
                    {
                        let b = self.rng.choose(&self.messages[..]).unwrap().borrow();
                        let b = self.rng.choose(&b.instances[..]).unwrap().borrow();
                        b.category.clone()
                    },
                ),
                self.cocategorization_ratio,
                self.forward_edge_distance,
                self.backward_edge_distance,
                self.forward_word_distance,
                self.backward_word_distance,
            );
        }
    }

    /// Print all multiple categories and return the amount of categories total
    pub fn show_categories(&self, socket: &mut SocketLend) {
        let mut set = BTreeSet::new();
        for message in &self.messages {
            for instance in &message.borrow().instances {
                let ib = instance.borrow();
                set.insert(ib.category.clone());
            }
        }

        for cat in set {
            let catr = cat.borrow();
            if catr.instances.len() != 1 {
                socket.msg("Category:");
                for cocategory in &catr.precocategories {
                    socket.msg("\tPre-Cocategory:");
                    let catr = cocategory.borrow();
                    for instance in &catr.instances {
                        let ib = instance.borrow();
                        socket.msg(&format!("\t\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow()));
                    }
                }
                for cocategory in &catr.postcocategories {
                    socket.msg("\tPost-Cocategory:");
                    let catr = cocategory.borrow();
                    for instance in &catr.instances {
                        let ib = instance.borrow();
                        socket.msg(&format!("\t\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow()));
                    }
                }
                for instance in &catr.instances {
                    let ib = instance.borrow();
                    socket.msg(&format!("\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow()));
                }
            }
        }
    }

    pub fn find_relation(&self, words: (String, String), socket: &mut SocketLend) {
        let wls = (self.words.get(&words.0), self.words.get(&words.1));
        match wls {
            (Some(w0), Some(w1)) => {
                let b = w0.borrow();
                for instance in &b.instances {
                    let b = instance.borrow();
                    let cb = b.category.borrow();

                    for cins in &cb.instances {
                        if cins.borrow().word == *w1 {
                            socket.msg("Category:");
                            for instance in &cb.instances {
                                let ib = instance.borrow();
                                socket.msg(&format!("\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow()));
                            }
                            break;
                        }
                    }

                    // Check all the precategories
                    for prec in &cb.precocategories {
                        let cb = prec.borrow();
                        for cins in &cb.instances {
                            if cins.borrow().word == *w1 {
                                socket.msg("Pre-Cocategory:");
                                for instance in &cb.instances {
                                    let ib = instance.borrow();
                                    socket.msg(&format!("\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow()));
                                }
                                break;
                            }
                        }
                    }

                    // Check all the postcategories
                    for postc in &cb.postcocategories {
                        let cb = postc.borrow();
                        for cins in &cb.instances {
                            if cins.borrow().word == *w1 {
                                socket.msg("Post-Cocategory:");
                                for instance in &cb.instances {
                                    let ib = instance.borrow();
                                    socket.msg(&format!("\t{} ~ {}", ib.word.borrow().name, &*ib.message.borrow()));
                                }
                                break;
                            }
                        }
                    }
                }
            },
            (Some(_), None) => {
                socket.msg(&format!("Ignored: Word \"{}\" coldn't be found", words.1));
            },
            (None, Some(_)) => {
                socket.msg(&format!("Ignored: Word \"{}\" coldn't be found", words.0));
            },
            (None, None) => {
                socket.msg("Ignored: Neither word could be found");
            },
        }
    }
}

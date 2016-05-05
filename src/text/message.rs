extern crate itertools;
use self::itertools::Itertools;

use super::*;
use std::fmt;
use std::cmp::Ordering;

pointer_ord!(Message);

impl Message {
    pub fn string(&self) -> String {
        self.instances.iter()
            .map(|i| {
                let b = i.borrow();
                let b = b.word.borrow();
                b.name.clone()
            })
            .join(" ")
    }
    pub fn mismatch<F>(messages: (MessageCell, MessageCell), diff: F) -> Mismatch<(InstanceCell, InstanceCell)>
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

    pub fn category_and_word_mismatch(messages: (MessageCell, MessageCell)) ->
        Mismatch<(InstanceCell, InstanceCell)> {
        Self::mismatch(messages, |ins| ins.0.word != ins.1.word && ins.0.category != ins.1.category)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string())
    }
}

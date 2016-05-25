use super::*;
use std::cmp::Ordering;

pointer_ord!(WordInstance);

impl WordInstance {
    pub fn next_instance(&self) -> Option<InstanceCell> {
        let mb = self.message.borrow();
        mb.instances.get((self.index as isize + 1) as usize).cloned()
    }

    pub fn prev_instance(&self) -> Option<InstanceCell> {
        let mb = self.message.borrow();
        mb.instances.get((self.index as isize - 1) as usize).cloned()
    }

    pub fn precoincidence_neighbors(ins: (&InstanceCell, &InstanceCell), min_edge_distance: usize, words_only_distance: usize) -> bool {
        let bs = (ins.0.borrow(), ins.1.borrow());
        let ms = (bs.0.message.borrow(), bs.1.message.borrow());
        let preins = (ms.0.instances.get((bs.0.index as isize - 1) as usize),
                ms.1.instances.get((bs.1.index as isize - 1) as usize));

        match preins {
            // There are two words
            (Some(i0), Some(i1)) => {
                if Category::are_postcocategories((&i0.borrow().category, &i1.borrow().category)) ||
                    Category::are_precocategories((&i0.borrow().category, &i1.borrow().category)) {
                    true
                } else if i0.borrow().word == i1.borrow().word {
                    if words_only_distance == 0 {
                        true
                    } else {
                        WordInstance::precoincidence_neighbors((i0, i1),
                            min_edge_distance.saturating_sub(1),
                            words_only_distance.saturating_sub(1))
                    }
                } else {
                    false
                }
            },
            // End of the sentence is only a match if we had prior matches
            (None, None) => min_edge_distance == 0,
            _ => false,
        }
    }

    pub fn postcoincidence_neighbors(ins: (&InstanceCell, &InstanceCell), min_edge_distance: usize, words_only_distance: usize) -> bool {
        let bs = (ins.0.borrow(), ins.1.borrow());
        let ms = (bs.0.message.borrow(), bs.1.message.borrow());
        let postins = (ms.0.instances.get((bs.0.index as isize + 1) as usize),
                ms.1.instances.get((bs.1.index as isize + 1) as usize));

        match postins {
            // There are two words
            (Some(i0), Some(i1)) => {
                if Category::are_precocategories((&i0.borrow().category, &i1.borrow().category)) ||
                    Category::are_postcocategories((&i0.borrow().category, &i1.borrow().category)) {
                    true
                } else if i0.borrow().word == i1.borrow().word {
                    if words_only_distance == 0 {
                        true
                    } else {
                        WordInstance::postcoincidence_neighbors((i0, i1),
                            min_edge_distance.saturating_sub(1),
                            words_only_distance.saturating_sub(1))
                    }
                } else {
                    false
                }
            },
            // End of the sentence is only a match if this can be the edge
            (None, None) => min_edge_distance == 0,
            _ => false,
        }
    }
}

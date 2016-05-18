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

    pub fn precoincidence_neighbors(ins: (&InstanceCell, &InstanceCell)) -> bool {
        let bs = (ins.0.borrow(), ins.1.borrow());
        let ms = (bs.0.message.borrow(), bs.1.message.borrow());
        let preins = (ms.0.instances.get((bs.0.index as isize - 1) as usize),
                ms.1.instances.get((bs.1.index as isize - 1) as usize));

        match preins {
            // There are two words
            (Some(i0), Some(i1)) => {
                i0.borrow().word == i1.borrow().word ||
                    Category::are_postcocategories((&i0.borrow().category, &i1.borrow().category))
            },
            // The sentence ends in both spots
            (None, None) => true,
            _ => false,
        }
    }

    pub fn postcoincidence_neighbors(ins: (&InstanceCell, &InstanceCell)) -> bool {
        let bs = (ins.0.borrow(), ins.1.borrow());
        let ms = (bs.0.message.borrow(), bs.1.message.borrow());
        let postins = (ms.0.instances.get((bs.0.index as isize + 1) as usize),
                ms.1.instances.get((bs.1.index as isize + 1) as usize));

        match postins {
            // There are two words
            (Some(i0), Some(i1)) => {
                i0.borrow().word == i1.borrow().word ||
                    Category::are_precocategories((&i0.borrow().category, &i1.borrow().category))
            },
            // The sentence ends in both spots
            (None, None) => true,
            _ => false,
        }
    }
}

use super::*;
use super::wrap;
use std::cmp::Ordering;

pointer_ord!(Category);

impl Category {
    pub fn merge(cs: (CategoryCell, CategoryCell)) -> CategoryCell {
        let cc = wrap(Self::default());
        {
            let clos = |cat: &CategoryCell| {
                let b = cat.borrow();
                for precocat in &b.precocategories {
                    cc.borrow_mut().precocategories.insert(precocat.clone());
                    let mut pcb = precocat.borrow_mut();
                    pcb.precocategories.remove(cat);
                    pcb.precocategories.insert(cc.clone());
                }
                for postcocat in &b.postcocategories {
                    cc.borrow_mut().postcocategories.insert(postcocat.clone());
                    let mut pcb = postcocat.borrow_mut();
                    pcb.postcocategories.remove(cat);
                    pcb.postcocategories.insert(cc.clone());
                }
            };
            clos(&cs.0);
            clos(&cs.1);
            let mut ccm = cc.borrow_mut();
            ccm.instances.append(&mut cs.0.borrow_mut().instances);
            cs.0.borrow_mut().precocategories.clear();
            cs.0.borrow_mut().postcocategories.clear();
            ccm.instances.append(&mut cs.1.borrow_mut().instances);
            cs.1.borrow_mut().precocategories.clear();
            cs.1.borrow_mut().postcocategories.clear();
            for instance in &ccm.instances {
                instance.borrow_mut().category = cc.clone();
            }
        }
        cc
    }

    /// Determine if the categories should be cocategories
    pub fn cocategorize(cs: (CategoryCell, CategoryCell),
        cocategorization_ratio: f64,
        forward_edge_distance: usize,
        backward_edge_distance: usize,
        forward_word_distance: usize,
        backward_word_distance: usize,
    ) {
        // First, check to see if they are the same category
        if cs.0 == cs.1 {
            // Nothing to do in that case
            return;
        }

        // Get the total amount of instances in cs.0
        let total = cs.0.borrow().instances.len() * cs.1.borrow().instances.len();
        // Make a counter to see how many instances coincide
        let mut pre_coincidences = 0;
        let mut post_coincidences = 0;

        let needed = (total as f64 * cocategorization_ratio + 0.5) as usize + 1;

        // Look through all the instances between both categories
        {
            let bs = (cs.0.borrow(), cs.1.borrow());
            for i0 in bs.0.instances.iter() {
                // We see if there is any coincidence for this instance
                for i1 in bs.1.instances.iter() {
                    // It is impossible for two different categories to contain the same instance,
                    // so that doesn't need to be checked for.

                    // TODO: Look behind and ahead by more than just 1 instance
                    if WordInstance::precoincidence_neighbors((i0, i1), forward_edge_distance, forward_word_distance) {
                        // Increment the amount of coincidences
                        pre_coincidences += 1;
                    }
                    if WordInstance::postcoincidence_neighbors((i0, i1), backward_edge_distance, backward_word_distance) {
                        // Increment the amount of coincidences
                        post_coincidences += 1;
                    }
                }
            }
        }

        // If the amount of coincidences is sufficient enough
        if pre_coincidences >= needed {
            // Make these cocategories

            // This if statement allows the code to avoid trying to add the cocategory to the second set if
            // it knows it was found in the first one
            if cs.0.borrow_mut().precocategories.insert(cs.1.clone()) {
                cs.1.borrow_mut().precocategories.insert(cs.0.clone());
            }
        } else {
            // Unmake these cocategories

            // This if statement allows the code to avoid trying to remove the cocategory from the second set if
            // it knows it wasnt found in the first one
            if cs.0.borrow_mut().precocategories.remove(&cs.1) {
                cs.1.borrow_mut().precocategories.remove(&cs.0);
            }
        }

        // If the amount of coincidences is sufficient enough
        if post_coincidences >= needed {
            // Make these cocategories

            // This if statement allows the code to avoid trying to add the cocategory to the second set if
            // it knows it was found in the first one
            if cs.0.borrow_mut().postcocategories.insert(cs.1.clone()) {
                cs.1.borrow_mut().postcocategories.insert(cs.0.clone());
            }
        } else {
            // Unmake these cocategories

            // This if statement allows the code to avoid trying to remove the cocategory from the second set if
            // it knows it wasnt found in the first one
            if cs.0.borrow_mut().postcocategories.remove(&cs.1) {
                cs.1.borrow_mut().postcocategories.remove(&cs.0);
            }
        }
    }

    pub fn are_precocategories(cs: (&CategoryCell, &CategoryCell)) -> bool {
        // If they are the same category they are cocategories
        if cs.0 == cs.1 {
            true
        } else {
            cs.0.borrow().precocategories.contains(&cs.1)
        }
    }

    pub fn are_postcocategories(cs: (&CategoryCell, &CategoryCell)) -> bool {
        // If they are the same category they are cocategories
        if cs.0 == cs.1 {
            true
        } else {
            cs.0.borrow().postcocategories.contains(&cs.1)
        }
    }
}

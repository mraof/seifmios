use super::*;
use super::wrap;
use std::cmp::Ordering;

const RATIO_TO_COCATEGORIZE: f64 = 0.05;

pointer_ord!(Category);

impl Category {
    pub fn merge(cs: (CategoryCell, CategoryCell)) -> CategoryCell {
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
    pub fn cocategorize(cs: (CategoryCell, CategoryCell)) {
        // First, check to see if they are the same category
        if cs.0 == cs.1 {
            // Nothing to do in that case
            return;
        }

        // Get the total amount of instances in cs.0
        let total = cs.0.borrow().instances.len() * cs.1.borrow().instances.len();
        // Make a counter to see how many instances coincide
        let mut coincidences = 0;

        let needed = (total as f64 * RATIO_TO_COCATEGORIZE + 0.5) as usize + 1;

        // Look through all the instances between both categories
        {
            let bs = (cs.0.borrow(), cs.1.borrow());
            'outer: for i0 in bs.0.instances.iter() {
                // We see if there is any coincidence for this instance
                for i1 in bs.1.instances.iter() {
                    // It is impossible for two different categories to contain the same instance,
                    // so that doesn't need to be checked for.

                    // TODO: Look behind and ahead by more than just 1 instance
                    // Check if the coincidence level is at least 1 (for now)
                    if WordInstance::coincidence_level((i0, i1)) >= 1 {
                        // Increment the amount of coincidences
                        coincidences += 1;
                        if coincidences == needed {
                            break 'outer;
                        }
                    }
                }
            }
        }

        // If the amount of coincidences is sufficient enough
        if coincidences == needed {
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

    pub fn are_cocategories(cs: (&CategoryCell, &CategoryCell)) -> bool {
        // If they are the same category they are cocategories
        if cs.0 == cs.1 {
            true
        } else {
            cs.0.borrow().cocategories.contains(&cs.1)
        }
    }
}

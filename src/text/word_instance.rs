use super::*;
use std::cmp::Ordering;

pointer_ord!(WordInstance);

impl WordInstance {
    pub fn coincidence_level(ins: (&InstanceCell, &InstanceCell)) -> usize {
        let bs = (ins.0.borrow(), ins.1.borrow());
        let ms = (bs.0.message.borrow(), bs.1.message.borrow());
        for i in 1.. {
            let msins = (
                (ms.0.instances.get((bs.0.index as isize - i) as usize),
                    ms.1.instances.get((bs.1.index as isize - i) as usize)),
                (ms.0.instances.get((bs.0.index as isize + i) as usize),
                    ms.1.instances.get((bs.1.index as isize + i) as usize)),
            );

            match msins.0 {
                // There are two words
                (Some(i0), Some(i1)) => {
                    // If both the categories and words don't match
                    if !Category::are_cocategories((&i0.borrow().category, &i1.borrow().category))
                        && i0.borrow().word != i1.borrow().word {
                        // The coincidence level doesn't go this far
                        return (i - 1) as usize;
                    // They do match
                    } else {
                        // But we also need to check the right instances
                        match msins.1 {
                            (Some(i0), Some(i1)) => {
                                // If both the categories and words don't match
                                if !Category::are_cocategories(
                                    (&i0.borrow().category, &i1.borrow().category))
                                    && i0.borrow().word != i1.borrow().word {
                                    // The coincidence level doesn't go this far
                                    return (i - 1) as usize;
                                }
                                // Otherwise everything matches and we go to the next iteration
                            },
                            (None, None) => {
                                // We can't go any further on this side, so this is the coincidence
                                return i as usize;
                            },
                            _ => {
                                // Any combination of Some and None is a mismatch
                                return (i - 1) as usize;
                            }
                        }
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
                                return (i - 1) as usize;
                            // They do match
                            } else {
                                // This is the end but also the correct level
                                return i as usize;
                            }
                        },
                        (None, None) => {
                            // We can't go any further on either side, so this is the coincidence
                            return i as usize;
                        },
                        _ => {
                            // Any combination of Some and None is a mismatch
                            return (i - 1) as usize;
                        }
                    }
                    unreachable!();
                }
                _ => {
                    // Any combination of Some and None is a mismatch
                    return (i - 1) as usize;
                }
            }
        }
        unreachable!();
    }
}

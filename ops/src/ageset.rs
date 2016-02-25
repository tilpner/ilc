use std::collections::HashSet;
use std::hash::Hash;

use blist::BList;

/// So... this is a rather weird thing.
/// It allows to semi-efficiently check the oldest (earliest insertion)
/// elements for certain criteria and remove them in the order of insertion
/// if the criteria is met.
pub struct AgeSet<T> {
    fifo: BList<T>,
    set: HashSet<T>,
}

impl<T> AgeSet<T>
    where T: Eq + Hash + Clone
{
    pub fn new() -> Self {
        AgeSet {
            fifo: BList::new(),
            set: HashSet::new(),
        }
    }

    pub fn contains(&self, t: &T) -> bool {
        self.set.contains(t)
    }

    pub fn prune<F>(&mut self, kill: F)
        where F: Fn(&T) -> bool
    {
        while let Some(ref e) = self.fifo.front().map(T::clone) {
            if kill(&e) {
                let removed = self.fifo.pop_front().unwrap();
                self.set.remove(&e);
                assert!(*e == removed);
            } else {
                break;
            }
        }
    }

    pub fn push(&mut self, t: T) {
        self.fifo.push_back(t.clone());
        self.set.insert(t);
    }
}

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use super::weight_map::Item;

#[derive(Clone, Copy, Debug)]
pub struct Consequence {
    pub start: u32,
    pub item: Item,
    pub end: u32,
    pub weight: f64,
}

impl Hash for Consequence {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.item.hash(state);
        self.end.hash(state);
    }
}

impl Eq for Consequence {}

impl PartialEq for Consequence {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.item == other.item
            && self.end == other.end
            && self.weight == other.weight
    }
}

impl PartialOrd for Consequence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Consequence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // self.weight
        //     .partial_cmp(&other.weight)
        //     .expect("the weights should never be not a number")
        match self.weight.total_cmp(&other.weight) {
            Ordering::Equal => {
                let item = self.item.cmp(&other.item);
                if item != Ordering::Equal {
                    return item
                }
                let start = self.start.cmp(&other.start);
                if start != Ordering::Equal {
                    return start;
                }
                self.end.cmp(&other.end)
            }
            ord => ord,
        }
    }
}

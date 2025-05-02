use std::cmp::Ordering;

use super::weight_map::Item;


#[derive(Clone, Copy, Debug)]
pub struct Consequence {
    pub start: u64,
    pub item: Item,
    pub end: u64,
    pub weight: f64,
}

impl Eq for Consequence {}

impl PartialEq for Consequence {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.item == other.item && self.end == other.end
    }
}

impl PartialOrd for Consequence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Consequence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.weight < other.weight {
            Ordering::Less
        } else if self.weight > other.weight {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

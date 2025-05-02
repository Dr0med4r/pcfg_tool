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
        self.weight.partial_cmp(&other.weight).unwrap()
        // match self.weight.total_cmp(&other.weight) {
        //     Ordering::Equal => {
        //         let item = self.item.cmp(&other.item);
        //         if item != Ordering::Equal {
        //             return item
        //         }
        //         let start = self.start.cmp(&other.start);
        //         if start != Ordering::Equal {
        //             return start;
        //         }
        //         self.end.cmp(&other.end)
        //     }
        //     ord => ord,
        // }
    }
}

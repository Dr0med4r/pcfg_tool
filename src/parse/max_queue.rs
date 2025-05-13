use std::{cmp::Ordering, collections::BTreeMap};

use indexmap::{IndexMap, IndexSet};
use ordered_float::NotNan;
use radix_heap::RadixHeapMap;

use super::{consequence::Consequence, weight_map::Item};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ConsequenceWithoutWeight {
    start: u32,
    item: Item,
    end: u32,
}

#[derive(PartialEq, Eq)]
struct Key {
    weight: NotNan<f64>,
    idx: usize,
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.weight.cmp(&other.weight) {
            Ordering::Equal => self.idx.cmp(&other.idx),
            ord => ord,
        }
    }
}

#[derive(Default)]
pub struct MaxQueue {
    heap: BTreeMap<Key, ()>,
    data: IndexMap<ConsequenceWithoutWeight, NotNan<f64>, foldhash::fast::RandomState>,
}

// impl Default for MaxQueue {
//     fn default() -> Self {
//         Self {
//             heap: RadixHeapMap::new_at(NotNan::try_from(1.0).unwrap()),
//             data: Default::default(),
//         }
//     }
// }

impl MaxQueue {
    pub fn pop(&mut self) -> Option<Consequence> {
        self.heap.pop_last().map(|(key, _)| {
            let (remaining, _) = &self
                .data
                .get_index(key.idx)
                .expect("key should be in the map");
            Consequence {
                start: remaining.start,
                item: remaining.item,
                end: remaining.end,
                weight: *key.weight,
            }
        })
    }

    pub fn push(&mut self, item: Consequence) {
        // eprintln!("pushing: {:?}", item);
        let part = ConsequenceWithoutWeight {
            start: item.start,
            item: item.item,
            end: item.end,
        };
        let weight = NotNan::try_from(item.weight).expect("should not be NaN");
        let (idx, old) = self.data.insert_full(part, weight);
        if let Some(old) = old {
            if old < weight {
                self.heap.remove(&Key { weight: old, idx });
                self.heap.insert(Key { weight, idx }, ());
            }
        } else {
            self.heap.insert(Key { weight, idx }, ());
        }
        // self.heap.entry(Key { weight: old, idx })
        // let idx = match self.free.pop() {
        //     Some(idx) => {
        //         self.data[idx] = part;
        //         idx
        //     }
        //     None => {
        //         self.data.push(part);
        //         self.data.len() - 1
        //     }
        // };
        // self.heap.push(
        //     NotNan::try_from(item.weight).expect("should not be NaN"),
        //     idx,
        // );
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

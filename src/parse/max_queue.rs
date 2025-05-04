use std::cmp::Ordering;

use ordered_float::NotNan;
use radix_heap::{Radix, RadixHeapMap};

use super::{consequence::Consequence, weight_map::Item};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Ordered {
    idx: usize,
    float: NotNan<f64>,
}

impl PartialOrd for Ordered {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ordered {
    fn cmp(&self, other: &Self) -> Ordering {
        self.float.cmp(&other.float)
        // match self.float.cmp(&other.float) {
        //     Ordering::Equal => {}
        //     ord => return ord,
        // }
        // self.idx.cmp(&other.idx).reverse()
    }
}

impl Radix for Ordered {
    fn radix_similarity(&self, other: &Self) -> u32 {
        (self.idx, self.float).radix_similarity(&(other.idx, other.float))
    }

    const RADIX_BITS: u32 = 128u32;
}

struct ConsequenceWithoutWeight {
    start: u64,
    item: Item,
    end: u64,
}

pub struct MaxQueue {
    heap: RadixHeapMap<Ordered, ()>,
    data: Vec<ConsequenceWithoutWeight>,
    free: Vec<usize>,
}

impl Default for MaxQueue {
    fn default() -> Self {
        Self {
            heap: RadixHeapMap::new_at(Ordered {
                idx: 0,
                float: NotNan::try_from(1.0).unwrap(),
            }),
            data: Default::default(),
            free: Default::default(),
        }
    }
}

impl MaxQueue {
    pub fn pop(&mut self) -> Option<Consequence> {
        self.heap.pop().map(|(weight, _)| {
            // eprintln!("popping: {:?}", weight);
            // self.free.push(weight.idx);
            let remaining = &self.data[weight.idx];
            Consequence {
                start: remaining.start,
                item: remaining.item,
                end: remaining.end,
                weight: *weight.float,
            }
        })
    }

    pub fn push(&mut self, item: Consequence) {
        let part = ConsequenceWithoutWeight {
            start: item.start,
            item: item.item,
            end: item.end,
        };
        let idx = match self.free.pop() {
            Some(idx) => {
                self.data[idx] = part;
                idx
            }
            None => {
                self.data.push(part);
                self.data.len() - 1
            }
        };
        self.heap.push(
            Ordered {
                idx,
                float: NotNan::try_from(item.weight).expect("should not be NaN"),
            },
            (),
        );
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn rem_len(&self) -> usize {
        self.free.len()
    }
}

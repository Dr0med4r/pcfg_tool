use std::{cmp::Ordering, collections::BinaryHeap};

use super::{consequence::Consequence, weight_map::Item};

struct Ordered {
    idx: usize,
    float: f64,
}

struct ConsequenceWithoutWeight {
    start: u64,
    item: Item,
    end: u64,
}

#[derive(Default)]
pub struct MaxQueue {
    heap: BinaryHeap<Ordered>,
    data: Vec<ConsequenceWithoutWeight>,
    free: Vec<usize>,
}

impl MaxQueue {
    pub fn with_capacity(capacity: usize) -> Self {
        MaxQueue {
            heap: BinaryHeap::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
            free: Vec::with_capacity(capacity),
        }
    }
    pub fn pop(&mut self) -> Option<Consequence> {
        self.heap.pop().map(|weight| {
            self.free.push(weight.idx);
            let remaining = &self.data[weight.idx];
            Consequence {
                start: remaining.start,
                item: remaining.item,
                end: remaining.end,
                weight: weight.float,
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
        self.heap.push(Ordered {
            idx,
            float: item.weight,
        });
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn rem_len(&self) -> usize {
        self.free.len()
    }
}

impl PartialOrd for Ordered {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ordered {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // self.weight
        //     .partial_cmp(&other.weight)
        //     .expect("the weights should never be not a number")
        self.float
            .partial_cmp(&other.float)
            .unwrap_or(Ordering::Equal)
    }
}

impl Eq for Ordered {}

impl PartialEq for Ordered {
    fn eq(&self, other: &Self) -> bool {
        self.float == other.float && self.idx == other.idx
    }
}

use std::collections::BinaryHeap;

use ordered_float::NotNan;
use radix_heap::RadixHeapMap;

use super::{
    consequence::Consequence,
    weight_map::{WeightMap, triangle_index},
};

#[derive(PartialEq, Eq)]
pub struct Key(NotNan<f64>, usize);

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

pub struct MaxQueue {
    heap: BinaryHeap<Key>,
    map: WeightMap<Consequence>,
    sentence_length: usize,
}

impl MaxQueue {
    pub fn new(items: usize, sentence_length: usize) -> Self {
        Self {
            heap: BinaryHeap::default(),
            map: WeightMap::with_capacity(items, sentence_length),
            sentence_length,
        }
    }

    pub fn pop(&mut self, mut viable_option: impl FnMut(usize) -> bool) -> Option<Consequence> {
        while let Some(Key(_weight, idx)) = self.heap.pop() {
            if viable_option(idx) {
                return Some(self.map.get_at_index(idx));
            }
        }
        None
    }

    pub fn push(&mut self, item: Consequence, key: f64) {
        let idx = triangle_index(
            self.sentence_length as u32,
            u32::from(item.item),
            item.start,
            item.end,
        );
        if self.map.get_at_index(idx).weight < item.weight {
            self.map.set_index(idx, item);
        }
        self.heap
            .push(Key(NotNan::try_from(key).expect("should not be NaN"), idx));
    }
}

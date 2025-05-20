use ordered_float::NotNan;
use radix_heap::RadixHeapMap;

use super::{
    consequence::Consequence,
    weight_map::{Item, WeightMap, triangle_index},
};

#[derive(Clone, Copy)]
struct ConsequenceWithoutWeight {
    start: u32,
    item: Item,
    end: u32,
}

impl Default for ConsequenceWithoutWeight {
    fn default() -> Self {
        Self {
            start: Default::default(),
            item: Item::NonTerminal(0),
            end: Default::default(),
        }
    }
}

pub struct MaxQueue {
    heap: RadixHeapMap<NotNan<f64>, usize>,
    map: WeightMap<ConsequenceWithoutWeight>,
    sentence_length: usize,
}

impl MaxQueue {
    pub fn new(items: usize, sentence_length: usize) -> Self {
        Self {
            heap: RadixHeapMap::new_at(NotNan::try_from(1.0).unwrap()),
            map: WeightMap::with_capacity(items, sentence_length),
            sentence_length,
        }
    }

    pub fn pop(&mut self, mut viable_option: impl FnMut(usize) -> bool) -> Option<Consequence> {
        while let Some((weight, idx)) = self.heap.pop() {
            if viable_option(idx) {
                let remaining = self.map.get_at_index(idx);
                return Some(Consequence {
                    start: remaining.start,
                    item: remaining.item,
                    end: remaining.end,
                    weight: *weight,
                });
            }
        }
        None
    }

    pub fn push(&mut self, item: Consequence) {
        let part = ConsequenceWithoutWeight {
            start: item.start,
            item: item.item,
            end: item.end,
        };
        let idx = triangle_index(
            self.sentence_length as u32,
            u32::from(item.item),
            item.start,
            item.end,
        );
        self.map.set_index(idx, part);
        self.heap.push(
            NotNan::try_from(item.weight).expect("should not be NaN"),
            idx,
        );
    }
}

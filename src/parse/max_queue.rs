use ordered_float::NotNan;
use radix_heap::RadixHeapMap;

use super::{consequence::Consequence, weight_map::Item};

struct ConsequenceWithoutWeight {
    start: u32,
    item: Item,
    end: u32,
}

pub struct MaxQueue {
    heap: RadixHeapMap<NotNan<f64>, usize>,
    data: Vec<ConsequenceWithoutWeight>,
    free: Vec<usize>,
}

impl Default for MaxQueue {
    fn default() -> Self {
        Self {
            heap: RadixHeapMap::new_at(NotNan::try_from(1.0).unwrap()),
            data: Default::default(),
            free: Default::default(),
        }
    }
}

impl MaxQueue {
    pub fn pop(&mut self) -> Option<Consequence> {
        self.heap.pop().map(|(weight, idx)| {
            let remaining = &self.data[idx];
            self.free.push(idx);
            Consequence {
                start: remaining.start,
                item: remaining.item,
                end: remaining.end,
                weight: *weight,
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
            NotNan::try_from(item.weight).expect("should not be NaN"),
            idx,
        );
    }
}

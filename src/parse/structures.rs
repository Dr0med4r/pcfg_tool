use foldhash::HashMap;
use nom::{
    Parser, bytes::complete::is_a, character::complete::char, character::complete::space0,
    combinator::map, multi::many_till, sequence::delimited,
};

use crate::induce::parse_tree::{ParseTree, atom};

use std::process::exit;

use std::hash::Hash;

use std::cmp::Ordering;

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

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum Item {
    NonTerminal(u64),
    Terminal(u64),
}

impl From<Item> for u64 {
    fn from(val: Item) -> Self {
        match val {
            Item::NonTerminal(u) => u,
            Item::Terminal(u) => u,
        }
    }
}

#[derive(Debug)]
pub struct Rule<T> {
    pub(crate) lhs: T,
    pub(crate) rhs: Vec<T>,
    pub(crate) weight: f64,
}

impl<T: Hash> Hash for Rule<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lhs.hash(state);
        self.rhs.hash(state);
    }
}

impl<T: Eq> Eq for Rule<T> {}

impl<T: PartialEq> PartialEq for Rule<T> {
    fn eq(&self, other: &Self) -> bool {
        // do not check the weight as there is only one rule in the grammar file
        self.lhs == other.lhs && self.rhs == other.rhs
    }
}

impl Rule<String> {
    pub(crate) fn from_rule(input: &str) -> Self {
        let to_string = |e: &str| e.to_string();
        let to_float = |e: &str| {
            e.parse::<f64>().unwrap_or_else(|e| {
                eprintln!("{e}");
                exit(1)
            })
        };
        let (_, (lhs, _, _, (rhs, weight))) = match (
            map(atom, to_string),
            char('-'),
            char('>'),
            many_till(
                map(atom, to_string),
                map(delimited(space0, is_a("1234567890."), space0), to_float),
            ),
        )
            .parse(input)
        {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };
        Rule { lhs, rhs, weight }
    }

    pub(crate) fn from_lexicon(input: &str) -> Self {
        let to_string = |e: &str| e.to_string();
        let to_float = |e: &str| {
            e.parse::<f64>().unwrap_or_else(|e| {
                eprintln!("lexicon parsing: {e}");
                exit(1)
            })
        };
        let (_, (lhs, rhs, weight)) = match (
            map(atom, to_string),
            map(atom, to_string),
            map(atom, to_float),
        )
            .parse(input)
        {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };
        Rule {
            lhs,
            rhs: vec![rhs],
            weight,
        }
    }
}

fn triangle_index(triangle_length: u64, triangle: u64, i: u64, j: u64) -> usize {
    ((triangle * WeightMap::elements(triangle_length))
        + WeightMap::elements(triangle_length - i - 1)
        + (triangle_length - j)) as usize
}

pub struct WeightMapIterator<'a> {
    data: &'a Vec<f64>,
    sentence_length: u64,
    /// the item over which to iterate
    item: Item,
    /// the fixed position of the value
    fixed: u64,
    /// the start position of the value
    pos: u64,
    /// if the fixed position is the end or the start
    start: bool,
}

impl Iterator for WeightMapIterator<'_> {
    type Item = Consequence;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            eprintln!("pos: {}", self.pos);
            if self.start {
                if self.pos > self.sentence_length {
                    return None;
                }
            } else if self.pos >= self.fixed {
                return None;
            }

            let index = if self.start {
                triangle_index(
                    self.sentence_length,
                    u64::from(self.item),
                    self.fixed,
                    self.pos,
                )
            } else {
                triangle_index(
                    self.sentence_length,
                    u64::from(self.item),
                    self.pos,
                    self.fixed,
                )
            };
            let pos = self.pos;
            self.pos += 1;
            if self.data[index] != 0.0 {
                if self.start {
                    return Some(Consequence {
                        start: self.fixed,
                        item: self.item,
                        end: pos,
                        weight: self.data[index],
                    });
                } else {
                    return Some(Consequence {
                        start: pos,
                        item: self.item,
                        end: self.fixed,
                        weight: self.data[index],
                    });
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct WeightMap {
    data: Vec<f64>,
    sentence_length: u64,
}

impl WeightMap {
    fn elements(len: u64) -> u64 {
        (len * (len + 1)) / 2
    }

    fn index(&self, consequence: Consequence) -> usize {
        // n: max_len index(a,b) = size(n-a-1) + n-b
        triangle_index(
            self.sentence_length,
            u64::from(consequence.item),
            consequence.start,
            consequence.end,
        )
    }

    pub fn get_consequence(&self, consequence: Consequence) -> f64 {
        assert!(consequence.start < consequence.end);
        assert!(consequence.end <= self.sentence_length);
        assert!(consequence.start < self.sentence_length);
        self.data[self.index(consequence)]
    }

    pub fn get_with_index(&self, item: Item, start: u64, end: u64) -> f64 {
        assert!(start < end);
        assert!(end <= self.sentence_length);
        assert!(start < self.sentence_length);
        self.data[triangle_index(self.sentence_length, u64::from(item), start, end)]
    }

    pub fn with_capacity(items: usize, sentence_length: usize) -> Self {
        let length = items * WeightMap::elements(sentence_length as u64) as usize;
        let mut data = Vec::with_capacity(length);
        data.resize(length, 0.0);
        WeightMap {
            data,
            sentence_length: sentence_length as u64,
        }
    }

    pub fn set(&mut self, consequence: Consequence) {
        let index = self.index(consequence);
        self.data[index] = consequence.weight
    }

    pub fn get_starts_at(&self, item: Item, start: u64) -> impl Iterator<Item = Consequence> {
        WeightMapIterator {
            sentence_length: self.sentence_length,
            data: &self.data,
            item,
            fixed: start,
            pos: start + 1,
            start: true,
        }
    }

    pub fn get_ends_at(&self, item: Item, end: u64) -> impl Iterator<Item = Consequence> {
        WeightMapIterator {
            sentence_length: self.sentence_length,
            data: &self.data,
            item,
            fixed: end,
            pos: 0,
            start: false,
        }
    }

    pub fn convert_to_parse_tree(
        self,
        initial: Item,
        string_lookup: HashMap<String, u64>,
    ) -> ParseTree<String> {
        // let mut tree = ParseTree::default();
        // tree.root = string_lookup[];
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn weightmap_test() {
        const RULES: u64 = 4;
        const SENTENCE: u64 = 4;
        let mut weight_map = WeightMap::with_capacity(RULES as usize, SENTENCE as usize);
        for rule in 0..RULES {
            for x in 0..SENTENCE {
                for y in x + 1..=SENTENCE {
                    eprintln!("{rule}({x}{y})");
                    weight_map.set(Consequence {
                        start: x,
                        item: Item::NonTerminal(rule),
                        end: y,
                        weight: rule as f64 + x as f64 + y as f64 / (3.0 * 3.0 * 4.0),
                    });
                }
            }
        }
        for rule in 0..RULES {
            for x in 0..SENTENCE {
                for y in x + 1..=SENTENCE {
                    let value = weight_map.get_consequence(Consequence {
                        start: x,
                        item: Item::NonTerminal(rule),
                        end: y,
                        weight: 0.5,
                    });
                    assert_eq!(value, rule as f64 + x as f64 + y as f64 / (3.0 * 3.0 * 4.0));
                }
            }
        }
    }

    #[test]
    fn weightmap_starts_at_test() {
        const RULES: u64 = 4;
        const SENTENCE: u64 = 4;
        let mut weight_map = WeightMap::with_capacity(RULES as usize, SENTENCE as usize);
        let consequence1 = Consequence {
            start: 0,
            item: Item::NonTerminal(1),
            end: 1,
            weight: 0.1,
        };
        weight_map.set(consequence1);
        let consequence2 = Consequence {
            start: 0,
            item: Item::NonTerminal(1),
            end: 3,
            weight: 0.1,
        };
        weight_map.set(consequence2);
        let consequence3 = Consequence {
            start: 1,
            item: Item::NonTerminal(1),
            end: 3,
            weight: 0.1,
        };
        weight_map.set(consequence3);

        let mut iter = weight_map.get_starts_at(Item::NonTerminal(1), 0);
        assert_eq!(iter.next(), Some(consequence1));
        assert_eq!(iter.next(), Some(consequence2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn weightmap_ends_at_test() {
        const RULES: u64 = 4;
        const SENTENCE: u64 = 4;
        let mut weight_map = WeightMap::with_capacity(RULES as usize, SENTENCE as usize);
        let consequence1 = Consequence {
            start: 0,
            item: Item::NonTerminal(1),
            end: 4,
            weight: 0.1,
        };
        weight_map.set(consequence1);
        let consequence2 = Consequence {
            start: 3,
            item: Item::NonTerminal(1),
            end: 4,
            weight: 0.1,
        };
        weight_map.set(consequence2);
        let consequence3 = Consequence {
            start: 1,
            item: Item::NonTerminal(1),
            end: 3,
            weight: 0.1,
        };
        weight_map.set(consequence3);

        let mut iter = weight_map.get_ends_at(Item::NonTerminal(1), 4);
        assert_eq!(iter.next(), Some(consequence1));
        assert_eq!(iter.next(), Some(consequence2));
        assert_eq!(iter.next(), None);
    }
}

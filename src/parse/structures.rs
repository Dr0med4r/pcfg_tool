use nom::{
    Parser, bytes::complete::is_a, character::complete::char, character::complete::space0,
    combinator::map, multi::many_till, sequence::delimited,
};

use crate::induce::parse_tree::atom;

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

#[derive(Debug, PartialEq)]
pub struct WeightMap {
    data: Vec<f64>,
    sentence_length: u64,
}

impl WeightMap {
    fn elements(len: u64) -> u64 {
        (len * (len + 1)) / 2
    }

    fn index(&self, consequence: &Consequence) -> usize {
        // n: max_len index(a,b) = size(n-a-1) + n-b
        ((u64::from(consequence.item) * WeightMap::elements(self.sentence_length))
            + WeightMap::elements(self.sentence_length - consequence.start - 1)
            + (self.sentence_length - consequence.end)) as usize
    }

    pub fn get(&self, consequence: &Consequence) -> f64 {
        assert!(consequence.start < consequence.end);
        assert!(consequence.end <= self.sentence_length);
        assert!(consequence.start < self.sentence_length);
        self.data[self.index(consequence)]
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

    pub fn set(&mut self, consequence: &Consequence) {
        let index = self.index(consequence);
        self.data[index] = consequence.weight
    }

    pub fn get_starts_at(&self, item: Item, start: u64) -> Option<Consequence> {
        for end in start + 1..=self.sentence_length {
            let index = self.index(&Consequence {
                start,
                item,
                end,
                weight: 0.0,
            });
            if self.data[index] != 0.0 {
                return Some(Consequence {
                    start,
                    item,
                    end,
                    weight: self.data[index],
                });
            }
        }
        None
    }

    pub fn get_ends_at(&self, item: Item, end: u64) -> Option<Consequence> {
        for start in 0..end {
            let index = self.index(&Consequence {
                start,
                item,
                end,
                weight: 0.0,
            });
            if self.data[index] != 0.0 {
                return Some(Consequence {
                    start,
                    item,
                    end,
                    weight: self.data[index],
                });
            }
        }
        None
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
                    weight_map.set(&Consequence {
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
                    let value = weight_map.get(&Consequence {
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
}

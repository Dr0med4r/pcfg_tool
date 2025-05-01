use nom::{
    Parser, bytes::complete::is_a, character::complete::char, character::complete::space0,
    combinator::map, multi::many_till, sequence::delimited,
};

use crate::induce::parse_tree::atom;

use std::process::exit;

use std::hash::Hash;

use std::cmp::Ordering;

pub(crate) struct Consequence {
    pub(crate) start: u64,
    pub(crate) item: Item,
    pub(crate) end: u64,
    pub(crate) weight: f64,
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
                eprintln!("{e}");
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

pub struct WeightMap {
    data: Vec<f64>,
    sentence_length: u64,
}

impl WeightMap {
    fn index(&self, consequence: &Consequence) -> usize {
        // n: max_len index(a,b) = size(n-a-1) + n-b
        let prev = (self.sentence_length - consequence.start - 1)
            * (self.sentence_length - consequence.start)
            / 2;
        ((u64::from(consequence.item) * (self.sentence_length * (self.sentence_length + 1) / 2))
            + prev
            + (self.sentence_length - consequence.end)) as usize
    }

    pub fn get(&self, consequence: &Consequence) -> f64 {
        assert!(consequence.start < consequence.end);
        assert!(consequence.end <= self.sentence_length);
        assert!(consequence.start < self.sentence_length);
        self.data[self.index(consequence)]
    }

    pub fn with_capacity(rules: usize, sentence_length: usize) -> Self {
        let length = rules * (sentence_length * (sentence_length + 1) / 2);
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn weightmap_test() {
        let mut weight_map = WeightMap::with_capacity(4, 4);
        for rule in 0..4 {
            for x in 0..4 {
                for y in x + 1..5 {
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
        for rule in 0..4 {
            for x in 0..4 {
                for y in x + 1..5 {
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

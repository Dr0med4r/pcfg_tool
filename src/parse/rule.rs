use std::process::exit;

use nom::{
    Parser,
    character::complete::{char, space0},
    combinator::map,
    multi::many_till,
    number::complete::recognize_float,
    sequence::delimited,
};
use std::hash::Hash;

use crate::induce::parse_tree::atom;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Rhs<T> {
    Unary(T),
    Binary(T, T),
}

#[derive(Debug)]
pub struct Rule<T> {
    pub lhs: T,
    pub rhs: Rhs<T>,
    pub weight: f64,
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
                eprintln!("parsing rule \"{input}\": {e}");
                exit(1)
            })
        };
        let (_, (lhs, _, _, (mut rhs, weight))) = match (
            map(atom, to_string),
            char('-'),
            char('>'),
            many_till(
                map(atom, to_string),
                map(delimited(space0, recognize_float, space0), to_float),
            ),
        )
            .parse(input)
        {
            Ok(a) => a,
            Err(e) => {
                eprintln!("parsing rule \"{input}\": {e}");
                exit(1);
            }
        };
        let last = rhs.pop();
        let first = rhs.pop();
        if !rhs.is_empty() {
            eprintln!("expecting binary rules");
            exit(1);
        }
        let rhs = if first.is_some() {
            let (Some(item1), Some(item2)) = (first, last) else {
                panic!()
            };
            Rhs::Binary(item1, item2)
        } else if last.is_some() {
            let Some(item) = last else { panic!() };
            Rhs::Unary(item)
        } else {
            eprintln!("malformed rules");
            exit(1);
        };
        Rule { lhs, rhs, weight }
    }

    pub(crate) fn from_lexicon(input: &str) -> Self {
        let to_string = |e: &str| e.to_string();
        let to_float = |e: &str| {
            e.parse::<f64>().unwrap_or_else(|e| {
                eprintln!("parsing lexicon \"{input}\": {e}");
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
                eprintln!("parsing lexicon \"{input}\": {e}");
                exit(1);
            }
        };
        Rule {
            lhs,
            rhs: Rhs::Unary(rhs),
            weight,
        }
    }
}

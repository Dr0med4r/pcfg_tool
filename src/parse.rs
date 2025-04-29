use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader},
    path::Path,
    process::exit,
};

use foldhash::HashMap;
use foldhash::HashSet;

use nom::{
    Parser,
    bytes::complete::is_a,
    character::{char, complete::space0},
    combinator::map,
    multi::many_till,
    sequence::delimited,
};

use crate::induce::parse_tree::atom;

struct Consequence<'a> {
    start: u64,
    item: &'a Item,
    end: u64,
    weight: f64,
}

impl<'a> Eq for Consequence<'a> {}

impl<'a> PartialEq for Consequence<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.item == other.item && self.end == other.end
    }
}

impl<'a> PartialOrd for Consequence<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.start.partial_cmp(&other.start) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.item.partial_cmp(&other.item) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.end.partial_cmp(&other.end)
    }
}

impl<'a> Ord for Consequence<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.weight < other.weight {
            return Ordering::Less;
        } else if self.weight > other.weight {
            return Ordering::Greater;
        } else {
            return Ordering::Equal;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Item {
    NonTerminal(String),
    Terminal(String),
}

#[derive(Debug)]
pub struct Rule<T> {
    lhs: T,
    rhs: Vec<T>,
    weight: f64,
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

impl Rule<Item> {
    fn from_rule(input: &str) -> Self {
        let to_nonterminal = |e: &str| Item::NonTerminal(e.to_string());
        let to_float = |e: &str| {
            e.parse::<f64>().unwrap_or_else(|e| {
                eprintln!("{e}");
                exit(1)
            })
        };
        let (_, (lhs, _, _, (rhs, weight))) = match (
            map(atom, to_nonterminal),
            char('-'),
            char('>'),
            many_till(
                map(atom, to_nonterminal),
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
    fn from_lexicon(input: &str) -> Self {
        let to_nonterminal = |e: &str| Item::NonTerminal(e.to_string());
        let to_terminal = |e: &str| Item::Terminal(e.to_string());
        let to_float = |e: &str| {
            e.parse::<f64>().unwrap_or_else(|e| {
                eprintln!("{e}");
                exit(1)
            })
        };
        let (_, (lhs, rhs, weight)) = match (
            map(atom, to_nonterminal),
            map(atom, to_terminal),
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

struct WeightMap {
    data: Vec<f64>,
    rules: usize,
    len: usize,
}

impl WeightMap {
    fn get(self, consequence: Consequence) {
        let index = consequence.item * (consequence.end * consequence.start)/2 ;
        return self.data.get(index)
    }

    fn with_capacity(rules: usize, len: usize) -> Self {
        let length = rules * (len * (len + 1) / 2);
        let mut data = Vec::with_capacity(length);
        data.resize(length, 0.0);
        WeightMap { data, rules, len }
    }

    fn set(&self, consequence: Consequence) -> _ {
        todo!()
    }
}

/// appends rules into all_rules and all nonterminals as key into lookup_rules
pub fn parse_rules<'a>(
    all_rules: &'a mut Vec<Rule<Item>>,
    path: &Path,
    rule: bool,
) -> HashMap<Item, HashSet<&'a Rule<Item>>> {
    let Ok(rules) = File::open(path) else {
        eprintln!("cannot open rules file");
        exit(1);
    };
    let mut rhs_grammar: HashMap<Item, HashSet<&Rule<Item>>> = HashMap::default();
    for line in BufReader::new(rules).lines() {
        let Ok(line) = line else {
            eprintln!("cannot read rule");
            exit(1);
        };
        if rule {
            all_rules.push(Rule::from_rule(&line));
        } else {
            all_rules.push(Rule::from_lexicon(&line));
        }
    }
    for rule in all_rules {
        for nonterminal in &rule.rhs {
            let Some(l) = rhs_grammar.get_mut(nonterminal) else {
                continue;
            };
            l.insert(rule);
        }
    }
    rhs_grammar
}

pub fn deduce(
    line: String,
    rule_lookup: HashMap<Item, HashSet<&Rule<Item>>>,
    lexicon: HashMap<Item, HashSet<&Rule<Item>>>,
) -> Vec<(Item, f64)> {
    let mut queue = BinaryHeap::new();
    for (index, word) in line.split_whitespace().enumerate() {
        for rule in lexicon.get(&Item::Terminal(word.to_owned())).unwrap() {
            queue.push(Consequence {
                start: index as u64,
                item: &rule.lhs,
                end: (index + 1) as u64,
                weight: rule.weight,
            });
        }
    }
    let mut consequences = Vec::new();
    let weight_map = WeightMap::with_capacity(rule_lookup.len(), queue.len());
    while let Some(consequence) = queue.pop() {
        if weight_map.get(consequence) == 0 {
            weight_map.set(consequence);
        }
    }

    return consequences;
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn from_string_test() {
        let rule = "A -> B 0.5";
        let rule = Rule::from_rule(rule);
        assert_eq!(
            Rule {
                lhs: Item::NonTerminal("A".to_string()),
                rhs: vec![Item::NonTerminal("B".to_string())],
                weight: 0.5
            },
            rule
        );
        let rule = " ROOT -> B C D 0.57   ";
        let rule = Rule::from_rule(rule);
        assert_eq!(
            Rule {
                lhs: Item::NonTerminal("ROOT".to_string()),
                rhs: vec![
                    Item::NonTerminal("B".to_string()),
                    Item::NonTerminal("C".to_string()),
                    Item::NonTerminal("D".to_string())
                ],
                weight: 0.57
            },
            rule
        );
    }
}

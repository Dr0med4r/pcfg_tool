mod structures;

use std::{
    collections::BinaryHeap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::exit,
};

use foldhash::HashMap;
use foldhash::HashSet;
use structures::{Item, Rule, WeightMap};

/// appends rules into all_rules and all nonterminals as keys into lookup_rules
pub fn parse_rules(
    string_map: &mut HashMap<String, u64>,
    rhs_grammar: &mut HashMap<Item, HashSet<Rule<Item>>>,
    path: &Path,
    is_rule: bool,
) {
    let Ok(rules) = File::open(path) else {
        eprintln!("cannot open rules file");
        exit(1);
    };
    let mut rule_count: u64 = 0;
    for line in BufReader::new(rules).lines() {
        let Ok(line) = line else {
            eprintln!("cannot read rule");
            exit(1);
        };
        insert_rule_into_map(string_map, is_rule, rhs_grammar, &mut rule_count, line);
    }
}

fn insert_rule_into_map(
    string_map: &mut HashMap<String, u64>,
    is_rule: bool,
    rhs_grammar: &mut HashMap<Item, HashSet<Rule<Item>>>,
    nonterminal_count: &mut u64,
    line: String,
) {
    let rule = if is_rule {
        Rule::from_rule(&line)
    } else {
        Rule::from_lexicon(&line)
    };
    let mut rhs = Vec::new();
    for nonterminal in rule.rhs {
        let item = *string_map.entry(nonterminal).or_insert_with(|| {
            let count = *nonterminal_count;
            *nonterminal_count += 1;
            count
        });
        rhs.push(if is_rule {
            Item::NonTerminal(item)
        } else {
            Item::Terminal(item)
        });
    }
    let lhs = Item::NonTerminal(*string_map.entry(rule.lhs).or_insert_with(|| {
        let lhs = *nonterminal_count;
        *nonterminal_count += 1;
        lhs
    }));
    for nonterminal in &rhs.clone() {
        let set = rhs_grammar.entry(*nonterminal).or_default();
        set.insert(Rule {
            lhs,
            rhs: rhs.clone(),
            weight: rule.weight,
        });
    }
}

pub fn transform_sentence(line: String, lexicon: &HashMap<String, u64>) -> Vec<Item> {
    line.split_whitespace()
        .map(|word| Item::Terminal(*lexicon.get(word).expect("this word is not in the lexicon")))
        .collect()
}

pub fn deduce(
    line: Vec<Item>,
    rule_lookup: &HashMap<Item, HashSet<Rule<Item>>>,
) -> Vec<(Item, f64)> {
    let mut queue = BinaryHeap::new();
    let sentence_length = line.len();
    for (index, word) in line.into_iter().enumerate() {
        for rule in rule_lookup.get(&word).unwrap() {
            queue.push(structures::Consequence {
                start: index as u64,
                item: rule.lhs,
                end: (index + 1) as u64,
                weight: rule.weight,
            });
        }
    }
    let mut weight_map = WeightMap::with_capacity(rule_lookup.len(), sentence_length);
    while let Some(consequence) = queue.pop() {
        if weight_map.get(&consequence) != 0.0 {
            continue;
        }
        weight_map.set(&consequence);
        for rule in rule_lookup.get(&consequence.item).expect("each rule should be in the lookup") {
            if consequence.item == rule.rhs[0] {
                for item in &rule.rhs[1..] {
                    
                }
            }
        }
    }

    todo!();
}

#[cfg(test)]
mod test {
    use foldhash::HashMapExt;
    use structures::*;

    use super::*;
    #[test]
    fn from_string_test() {
        let rule = "A -> B 0.5";
        let rule = structures::Rule::from_rule(rule);
        assert_eq!(
            Rule {
                lhs: "A".to_string(),
                rhs: vec!["B".to_string()],
                weight: 0.5
            },
            rule
        );
        let rule = " ROOT -> B C D 0.57   ";
        let rule = structures::Rule::from_rule(rule);
        assert_eq!(
            Rule {
                lhs: "ROOT".to_string(),
                rhs: vec!["B".to_string(), "C".to_string(), "D".to_string()],
                weight: 0.57
            },
            rule
        );
    }

    #[test]
    fn into_map_from_rules_test() {
        let mut string_map = HashMap::new();
        let is_rule = true;
        let mut rhs_grammar = HashMap::new();
        let mut nonterminal_count = 0;
        let line = "A -> B C 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            is_rule,
            &mut rhs_grammar,
            &mut nonterminal_count,
            line,
        );
        let desired_strings = HashMap::from_iter(vec![
            ("B".to_string(), 0),
            ("C".to_string(), 1),
            ("A".to_string(), 2),
        ]);
        assert_eq!(desired_strings, string_map);
        let desired_grammar = HashMap::from_iter(vec![
            (
                Item::NonTerminal(0),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(2),
                    rhs: vec![Item::NonTerminal(0), Item::NonTerminal(1)],
                    weight: 0.57,
                }]),
            ),
            (
                Item::NonTerminal(1),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(2),
                    rhs: vec![Item::NonTerminal(0), Item::NonTerminal(1)],
                    weight: 0.57,
                }]),
            ),
        ]);
        assert_eq!(desired_grammar, rhs_grammar);
    }

    #[test]
    fn into_map_from_lexicon_test() {
        let mut string_map = HashMap::new();
        let is_rule = false;
        let mut rhs_grammar = HashMap::new();
        let mut nonterminal_count = 0;
        let line = "A C 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            is_rule,
            &mut rhs_grammar,
            &mut nonterminal_count,
            line,
        );
        let desired_strings = HashMap::from_iter(vec![("C".to_string(), 0), ("A".to_string(), 1)]);
        assert_eq!(desired_strings, string_map);
        let desired_grammar = HashMap::from_iter(vec![(
            Item::Terminal(0),
            HashSet::from_iter(vec![Rule {
                lhs: Item::NonTerminal(1),
                rhs: vec![Item::Terminal(0)],
                weight: 0.57,
            }]),
        )]);
        assert_eq!(desired_grammar, rhs_grammar);
    }

    #[test]
    fn into_map_from_both_test() {
        let mut string_map = HashMap::new();
        let mut rhs_grammar = HashMap::new();
        let mut nonterminal_count = 0;
        let lexicon_line = "B D 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            false,
            &mut rhs_grammar,
            &mut nonterminal_count,
            lexicon_line,
        );
        let rule_line = "A -> B C 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            true,
            &mut rhs_grammar,
            &mut nonterminal_count,
            rule_line,
        );
        let desired_strings = HashMap::from_iter(vec![
            ("D".to_string(), 0),
            ("B".to_string(), 1),
            ("C".to_string(), 2),
            ("A".to_string(), 3),
        ]);
        assert_eq!(desired_strings, string_map);
        let desired_grammar = HashMap::from_iter(vec![
            (
                Item::NonTerminal(1),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(3),
                    rhs: vec![Item::NonTerminal(1), Item::NonTerminal(2)],
                    weight: 0.57,
                }]),
            ),
            (
                Item::NonTerminal(2),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(3),
                    rhs: vec![Item::NonTerminal(1), Item::NonTerminal(2)],
                    weight: 0.57,
                }]),
            ),
            (
                Item::Terminal(0),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(1),
                    rhs: vec![Item::Terminal(0)],
                    weight: 0.57,
                }]),
            ),
        ]);
        assert_eq!(desired_grammar, rhs_grammar);
    }
}

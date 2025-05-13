pub mod consequence;
mod max_queue;
pub mod rule;
pub mod string_lookup;
pub mod weight_map;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::exit,
};

use crate::parse::rule::Rule;
use consequence::Consequence;
use foldhash::HashMap;
use foldhash::HashSet;
use max_queue::MaxQueue;
use rule::Rhs;
use string_lookup::StringLookup;
use weight_map::{Item, WeightMap};

/// appends rules into all_rules and all nonterminals as keys into lookup_rules
pub fn parse_rules(
    string_map: &mut StringLookup,
    rhs_grammar: &mut HashMap<Item, HashSet<Rule<Item>>>,
    all_rules: &mut HashMap<Item, HashMap<Rhs<Item>, f64>>,
    path: &Path,
    is_rule: bool,
) {
    let Ok(rules) = File::open(path) else {
        eprintln!("cannot open rules file");
        exit(1);
    };
    for line in BufReader::new(rules).lines() {
        let Ok(line) = line else {
            eprintln!("cannot read rule");
            exit(1);
        };
        insert_rule_into_map(string_map, is_rule, rhs_grammar, all_rules, line);
    }
}

fn insert_rule_into_map(
    string_map: &mut StringLookup,
    is_rule: bool,
    rhs_grammar: &mut HashMap<Item, HashSet<Rule<Item>>>,
    all_rules: &mut HashMap<Item, HashMap<Rhs<Item>, f64>>,
    line: String,
) {
    let rule = if is_rule {
        Rule::from_rule(&line)
    } else {
        Rule::from_lexicon(&line)
    };
    let rhs = match rule.rhs {
        Rhs::Unary(item) => {
            let item = string_map.insert_and_get(item) as u32;
            Rhs::Unary(if is_rule {
                Item::NonTerminal(item)
            } else {
                Item::Terminal(item)
            })
        }
        Rhs::Binary(item1, item2) => {
            let item1 = string_map.insert_and_get(item1) as u32;
            let item2 = string_map.insert_and_get(item2) as u32;
            Rhs::Binary(Item::NonTerminal(item1), Item::NonTerminal(item2))
        }
    };
    let lhs = Item::NonTerminal(string_map.insert_and_get(rule.lhs) as u32);
    all_rules
        .entry(lhs)
        .and_modify(|e| {
            e.insert(rhs, rule.weight);
        })
        .or_insert(HashMap::from_iter(vec![(rhs, rule.weight)]));
    match &rhs {
        Rhs::Unary(item) => {
            insert_rule(rhs_grammar, rule.weight, &rhs, lhs, item);
        }
        Rhs::Binary(item1, item2) => {
            insert_rule(rhs_grammar, rule.weight, &rhs, lhs, item1);
            insert_rule(rhs_grammar, rule.weight, &rhs, lhs, item2);
        }
    }
}

fn insert_rule(
    rhs_grammar: &mut HashMap<Item, HashSet<Rule<Item>>>,
    weight: f64,
    rhs: &Rhs<Item>,
    lhs: Item,
    item: &Item,
) {
    let set = rhs_grammar.entry(*item).or_default();
    set.insert(Rule {
        lhs,
        rhs: *rhs,
        weight,
    });
}

pub fn transform_sentence(line: &str, lexicon: &StringLookup) -> Vec<Item> {
    line.split_whitespace()
        .map(|word| {
            Item::Terminal(lexicon.get(word).expect("this word is not in the lexicon") as u32)
        })
        .collect()
}

pub fn deduce(
    line: &[Item],
    rule_lookup: &HashMap<Item, HashSet<Rule<Item>>>,
    start_item: Item,
    number_of_items: usize,
) -> WeightMap<f64> {
    let mut queue = MaxQueue::default();
    let sentence_length = line.len();
    for (index, word) in line.iter().enumerate() {
        for rule in rule_lookup
            .get(word)
            .expect("there is no rule that produces the word")
        {
            queue.push(Consequence {
                start: index as u32,
                item: rule.lhs,
                end: (index + 1) as u32,
                weight: rule.weight,
            });
        }
    }
    let mut weight_map = WeightMap::with_capacity(number_of_items, sentence_length);
    while let Some(consequence) = queue.pop() {
        if weight_map.get_consequence(&consequence) != 0.0 {
            continue;
        }
        weight_map.set(consequence);
        if consequence.start == 0
            && consequence.end == sentence_length as u32
            && consequence.item == start_item
        {
            break;
        }
        // iterate over all rules with the item on the right
        for rule in rule_lookup
            .get(&consequence.item)
            .expect("there should be a rule with each nonterminal")
        {
            match rule.rhs {
                Rhs::Unary(_) => {
                    add_replace(&mut queue, rule, &consequence);
                }
                Rhs::Binary(item1, item2) => {
                    add_left(&mut queue, &weight_map, rule, (item1, item2), &consequence);
                    add_right(&mut queue, &weight_map, rule, (item1, item2), &consequence);
                }
            }
        }
    }
    eprintln!("len: {}", queue.len());
    weight_map
}

fn add_replace(queue: &mut MaxQueue, rule: &Rule<Item>, consequence: &Consequence) {
    // if there is a rule with the item on the right side replace it with the left side
    queue.push(Consequence {
        start: consequence.start,
        item: rule.lhs,
        end: consequence.end,
        weight: consequence.weight * rule.weight,
    })
}

fn add_right(
    queue: &mut MaxQueue,
    weight_map: &WeightMap<f64>,
    rule: &Rule<Item>,
    rhs: (Item, Item),
    consequence: &Consequence,
) {
    // if there is a rule with the item last
    // add all consequences to the queue where the sequence of items is in the weight map
    // such that item1.end == item2.start, item2.end == item3.start ...
    // then Consequence {start: item1.start, item: lhs, end: itemn.end } is added
    if consequence.item == rhs.1 {
        for next in weight_map.get_ends_at(rhs.0, consequence.start) {
            queue.push(Consequence {
                start: next.start,
                item: rule.lhs,
                end: consequence.end,
                // always multiply left with right to preserve same value
                weight: next.weight * consequence.weight * rule.weight,
            });
        }
    }
}

fn add_left(
    queue: &mut MaxQueue,
    weight_map: &WeightMap<f64>,
    rule: &Rule<Item>,
    rhs: (Item, Item),
    consequence: &Consequence,
) {
    // if there is a rule with the item first
    // add all consequences to the queue where the sequence of items is in the weight map
    // such that item1.end == item2.start, item2.end == item3.start ...
    // then Consequence {start: item1.start, item: lhs, end: itemn.end } is added
    if consequence.item == rhs.0 {
        for next in weight_map.get_starts_at(rhs.1, consequence.end) {
            queue.push(Consequence {
                start: consequence.start,
                item: rule.lhs,
                end: next.end,
                // always multiply left with right to preserve same value
                weight: consequence.weight * next.weight * rule.weight,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use foldhash::HashMapExt;

    use super::*;
    #[test]
    fn from_string_test() {
        let rule = "A -> B 0.5";
        let rule = Rule::from_rule(rule);
        assert_eq!(
            Rule {
                lhs: "A".to_string(),
                rhs: Rhs::Unary("B".to_string()),
                weight: 0.5
            },
            rule
        );
        let rule = " ROOT -> B C 0.57   ";
        let rule = Rule::from_rule(rule);
        assert_eq!(
            Rule {
                lhs: "ROOT".to_string(),
                rhs: Rhs::Binary("B".to_string(), "C".to_string()),
                weight: 0.57
            },
            rule
        );
    }

    #[test]
    fn into_map_from_rules_test() {
        let mut string_map = StringLookup::default();
        let is_rule = true;
        let mut rhs_grammar = HashMap::new();
        let mut all_rules = HashMap::new();
        let line = "A -> B C 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            is_rule,
            &mut rhs_grammar,
            &mut all_rules,
            line,
        );
        let desired_strings =
            StringLookup::from_iter(vec!["B".to_string(), "C".to_string(), "A".to_string()]);
        assert_eq!(desired_strings, string_map);
        let desired_grammar = HashMap::from_iter(vec![
            (
                Item::NonTerminal(0),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(2),
                    rhs: Rhs::Binary(Item::NonTerminal(0), Item::NonTerminal(1)),
                    weight: 0.57,
                }]),
            ),
            (
                Item::NonTerminal(1),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(2),
                    rhs: Rhs::Binary(Item::NonTerminal(0), Item::NonTerminal(1)),
                    weight: 0.57,
                }]),
            ),
        ]);
        assert_eq!(desired_grammar, rhs_grammar);
    }

    #[test]
    fn into_map_from_lexicon_test() {
        let mut string_map = StringLookup::default();
        let is_rule = false;
        let mut rhs_grammar = HashMap::new();
        let mut all_rules = HashMap::new();
        let line = "A C 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            is_rule,
            &mut rhs_grammar,
            &mut all_rules,
            line,
        );
        let desired_strings = StringLookup::from_iter(vec!["C".to_string(), "A".to_string()]);
        assert_eq!(desired_strings, string_map);
        let desired_grammar = HashMap::from_iter(vec![(
            Item::Terminal(0),
            HashSet::from_iter(vec![Rule {
                lhs: Item::NonTerminal(1),
                rhs: Rhs::Unary(Item::Terminal(0)),
                weight: 0.57,
            }]),
        )]);
        assert_eq!(desired_grammar, rhs_grammar);
    }

    #[test]
    fn into_map_from_both_test() {
        let mut string_map = StringLookup::default();
        let mut rhs_grammar = HashMap::new();
        let mut all_rules = HashMap::new();
        let lexicon_line = "B D 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            false,
            &mut rhs_grammar,
            &mut all_rules,
            lexicon_line,
        );
        let rule_line = "A -> B C 0.57".to_string();
        insert_rule_into_map(
            &mut string_map,
            true,
            &mut rhs_grammar,
            &mut all_rules,
            rule_line,
        );
        let desired_strings = StringLookup::from_iter(vec![
            "D".to_string(),
            "B".to_string(),
            "C".to_string(),
            "A".to_string(),
        ]);
        assert_eq!(desired_strings, string_map);
        let desired_grammar = HashMap::from_iter(vec![
            (
                Item::NonTerminal(1),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(3),
                    rhs: Rhs::Binary(Item::NonTerminal(1), Item::NonTerminal(2)),
                    weight: 0.57,
                }]),
            ),
            (
                Item::NonTerminal(2),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(3),
                    rhs: Rhs::Binary(Item::NonTerminal(1), Item::NonTerminal(2)),
                    weight: 0.57,
                }]),
            ),
            (
                Item::Terminal(0),
                HashSet::from_iter(vec![Rule {
                    lhs: Item::NonTerminal(1),
                    rhs: Rhs::Unary(Item::Terminal(0)),
                    weight: 0.57,
                }]),
            ),
        ]);
        assert_eq!(desired_grammar, rhs_grammar);
        let desired_rules = HashMap::from_iter(vec![
            (
                Item::NonTerminal(3),
                HashMap::from_iter(vec![(
                    Rhs::Binary(Item::NonTerminal(1), Item::NonTerminal(2)),
                    0.57,
                )]),
            ),
            (
                Item::NonTerminal(1),
                HashMap::from_iter(vec![(Rhs::Unary(Item::Terminal(0)), 0.57)]),
            ),
        ]);
        assert_eq!(desired_rules, all_rules);
    }

    #[test]
    fn deduce_test() {
        let mut string_map = StringLookup::default();
        let mut grammar = HashMap::new();
        let mut all_rules = HashMap::new();
        let lexicon = vec![
            "W1 R 0.2".to_string(),
            "W2 S 1".to_string(),
            "W1 T 0.2".to_string(),
        ];
        for line in lexicon {
            insert_rule_into_map(&mut string_map, false, &mut grammar, &mut all_rules, line);
        }
        let rules = vec![
            "ROOT -> W1 W2 0.25".to_string(),
            "ROOT -> W2 W2 0.75".to_string(),
            "W1 -> W2 0.6".to_string(),
        ];
        for line in rules {
            insert_rule_into_map(&mut string_map, true, &mut grammar, &mut all_rules, line);
        }
        let initial = Item::NonTerminal(string_map.get("ROOT").unwrap() as u32);
        grammar.entry(initial).or_default();

        let line = transform_sentence("R S T", &string_map);
        let mut desired_weight_map = WeightMap::with_capacity(string_map.len(), line.len());
        // R: 0
        // W1: 1
        // S: 2
        // W2: 3
        // T: 4
        // ROOT: 5
        desired_weight_map.set(Consequence {
            start: 0,
            item: Item::NonTerminal(1),
            end: 1,
            weight: 0.2,
        });
        desired_weight_map.set(Consequence {
            start: 1,
            item: Item::NonTerminal(3),
            end: 2,
            weight: 1.0,
        });
        desired_weight_map.set(Consequence {
            start: 2,
            item: Item::NonTerminal(1),
            end: 3,
            weight: 0.2,
        });
        desired_weight_map.set(Consequence {
            start: 1,
            item: Item::NonTerminal(1),
            end: 2,
            weight: 0.6,
        });
        desired_weight_map.set(Consequence {
            start: 0,
            item: Item::NonTerminal(5),
            end: 2,
            weight: 0.05,
        });
        let weight_map = deduce(&line, &grammar, initial, string_map.len());
        assert_eq!(weight_map, desired_weight_map);
    }
}

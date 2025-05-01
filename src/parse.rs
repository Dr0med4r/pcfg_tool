pub mod structures;

use std::{
    collections::BinaryHeap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process::exit,
};

use foldhash::HashMap;
use foldhash::HashSet;
use structures::{Consequence, Item, Rule, WeightMap};

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
    start_item: Item,
    number_of_items: usize,
) -> WeightMap {
    let mut queue = BinaryHeap::new();
    let sentence_length = line.len();
    for (index, word) in line.iter().enumerate() {
        for rule in rule_lookup
            .get(word)
            .expect("there is no rule that produces the word")
        {
            queue.push(structures::Consequence {
                start: index as u64,
                item: rule.lhs,
                end: (index + 1) as u64,
                weight: rule.weight,
            });
        }
    }
    let mut weight_map = WeightMap::with_capacity(number_of_items, sentence_length);
    while let Some(consequence) = queue.pop() {
        if weight_map.get_consequence(&consequence) != 0.0 {
            continue;
        }
        weight_map.set(&consequence);
        if weight_map.get_consequence(&Consequence {
            start: 0,
            item: start_item,
            end: sentence_length as u64,
            weight: 0.0,
        }) != 0.0
        {
            break;
        }
        eprintln!("looking for {:?}", consequence);
        // iterate over all rules with the item on the right
        for rule in rule_lookup
            .get(&consequence.item)
            .expect("there should be a rule with each nonterminal")
        {
            // TODO either alway max two rules or do it correctly
            add_left(&mut queue, &weight_map, rule, consequence);
            add_right(&mut queue, &weight_map, rule, consequence);
            add_replace(&mut queue, rule, consequence);
        }
    }
    weight_map
}

fn add_replace(queue: &mut BinaryHeap<Consequence>, rule: &Rule<Item>, consequence: Consequence) {
    // if there is a rule with the item on the right side replace it with the left side
    if rule.rhs.len() == 1 {
        eprintln!("replace: {:?} with {:?}", consequence.item, rule.lhs);
        queue.push(Consequence {
            start: consequence.start,
            item: rule.lhs,
            end: consequence.end,
            weight: consequence.weight * rule.weight,
        })
    }
}

fn add_right(
    queue: &mut BinaryHeap<Consequence>,
    weight_map: &WeightMap,
    rule: &Rule<Item>,
    consequence: Consequence,
) {
    // if there is a rule with the item last
    // add all consequences to the queue where the sequence of items is in the weight map
    // such that item1.end == item2.start, item2.end == item3.start ...
    // then Consequence {start: item1.start, item: lhs, end: itemn.end } is added
    if consequence.item == rule.rhs[rule.rhs.len() - 1] {
        for item in rule.rhs[..rule.rhs.len() - 1].iter().rev() {
            if let Some(next) = weight_map.get_ends_at(*item, consequence.start) {
                eprintln!("add {:?} left of {:?}", &next, &consequence);
                queue.push(Consequence {
                    start: next.start,
                    item: rule.lhs,
                    end: consequence.end,
                    weight: consequence.weight * next.weight * rule.weight,
                });
            }
        }
    }
}

fn add_left(
    queue: &mut BinaryHeap<Consequence>,
    weight_map: &WeightMap,
    rule: &Rule<Item>,
    consequence: Consequence,
) {
    // if there is a rule with the item first
    // add all consequences to the queue where the sequence of items is in the weight map
    // such that item1.end == item2.start, item2.end == item3.start ...
    // then Consequence {start: item1.start, item: lhs, end: itemn.end } is added
    if consequence.item == rule.rhs[0] {
        for item in &rule.rhs[1..] {
            if let Some(next) = weight_map.get_starts_at(*item, consequence.end) {
                eprintln!("add {:?} right of {:?}", &next, &consequence);
                queue.push(Consequence {
                    start: consequence.start,
                    item: rule.lhs,
                    end: next.end,
                    weight: consequence.weight * next.weight * rule.weight,
                });
            }
        }
    }
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

    #[test]
    fn deduce_test() {
        let mut string_map = HashMap::new();
        let mut grammar = HashMap::new();
        let mut nonterminal_count = 0;
        let lexicon = vec![
            "W1 R 0.2".to_string(),
            "W2 S 1".to_string(),
            "W1 T 0.2".to_string(),
        ];
        for line in lexicon {
            insert_rule_into_map(
                &mut string_map,
                false,
                &mut grammar,
                &mut nonterminal_count,
                line,
            );
        }
        let rules = vec![
            "ROOT -> W1 W2 0.25".to_string(),
            "ROOT -> W2 W2 0.75".to_string(),
            "W1 -> W2 0.6".to_string(),
        ];
        for line in rules {
            insert_rule_into_map(
                &mut string_map,
                true,
                &mut grammar,
                &mut nonterminal_count,
                line,
            );
        }
        eprintln!("grammar: \n{:#?}", grammar);
        let initial = Item::NonTerminal(*string_map.get("ROOT").unwrap());
        grammar.entry(initial).or_default();

        let line = transform_sentence("R S T".to_string(), &string_map);
        let mut desired_weight_map = WeightMap::with_capacity(string_map.len(), line.len());
        // R: 0
        // W1: 1
        // S: 2
        // W2: 3
        // T: 4
        // ROOT: 5
        desired_weight_map.set(&Consequence {
            start: 0,
            item: Item::NonTerminal(1),
            end: 1,
            weight: 0.2,
        });
        desired_weight_map.set(&Consequence {
            start: 1,
            item: Item::NonTerminal(3),
            end: 2,
            weight: 1.0,
        });
        desired_weight_map.set(&Consequence {
            start: 2,
            item: Item::NonTerminal(1),
            end: 3,
            weight: 0.2,
        });
        desired_weight_map.set(&Consequence {
            start: 1,
            item: Item::NonTerminal(1),
            end: 2,
            weight: 0.6,
        });
        desired_weight_map.set(&Consequence {
            start: 0,
            item: Item::NonTerminal(5),
            end: 2,
            weight: 0.05,
        });
        let weight_map = deduce(line, &grammar, initial, string_map.len());
        assert_eq!(weight_map, desired_weight_map);
    }
}

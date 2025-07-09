pub mod consequence;
mod max_queue;
pub mod rule;
pub mod string_lookup;
pub mod weight_map;

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    process::exit,
};

use crate::{astar::ViterbiScore, parse::rule::Rule, smoothing::smooth_word};
use consequence::Consequence;
use foldhash::HashSet;
use foldhash::{HashMap, HashMapExt};
use max_queue::MaxQueue;
use rule::Rhs;
use string_lookup::StringLookup;
use weight_map::{Item, WeightMap};

pub fn parse(
    rules: &Path,
    lexicon: &Path,
    paradigma: &Option<String>,
    initial_nonterminal: &str,
    unking: &bool,
    smoothing: &bool,
    threshold_beam: &Option<u64>,
    rank_beam: &Option<u64>,
    astar: &Option<std::path::PathBuf>,
) {
    match paradigma {
        Some(paradigma) if paradigma == &"cyk".to_string() => exit(22),
        _ => {}
    }
    if threshold_beam.is_some() || rank_beam.is_some() {
        exit(22);
    }

    let mut string_lookup = StringLookup::default();
    let mut rule_lookup = HashMap::new();
    let mut all_rules = HashMap::new();
    parse_rules(
        &mut string_lookup,
        &mut rule_lookup,
        &mut all_rules,
        rules,
        true,
    );
    parse_rules(
        &mut string_lookup,
        &mut rule_lookup,
        &mut all_rules,
        lexicon,
        false,
    );
    let initial_nonterminal = Item::NonTerminal(
        string_lookup
            .get(initial_nonterminal)
            .expect("initial nonterminal is not in the rules") as u32,
    );
    rule_lookup.entry(initial_nonterminal).or_default();
    let mut rule_lookup_vec = vec![vec![]; rule_lookup.len()];
    for (item, set) in rule_lookup {
        rule_lookup_vec[u32::from(item) as usize] = set.into_iter().collect()
    }

    let scores = astar.as_ref().map(|astar| {
        ViterbiScore::new_from_file(astar, &string_lookup)
            .expect("Could not read from .outside file!")
    });
    for (line_number, line) in io::stdin().lines().enumerate() {
        let Ok(line) = line else {
            eprintln!("error reading line {}", line_number + 1);
            exit(1);
        };
        let line_items = transform_sentence(&line, &string_lookup, unking, smoothing);
        let rule_weights = deduce(
            &line_items,
            &rule_lookup_vec,
            scores.as_ref(),
            initial_nonterminal,
            string_lookup.len(),
        );
        if rule_weights.get_with_index(initial_nonterminal, 0, line_items.len() as u32) == 0.0 {
            println!("(NOPARSE {})", line)
        } else {
            let tree = rule_weights.convert_to_parse_tree(
                initial_nonterminal,
                0,
                line_items.len() as u32,
                &string_lookup,
                &all_rules,
                &mut line_items.into(),
            );
            println!("{tree}")
        }
    }
}

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
        insert_into_lookup(string_map, is_rule, rhs_grammar, all_rules, line);
    }
}

fn insert_into_lookup(
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
            insert_rule(rhs_grammar, rule.weight, rhs, lhs, item);
        }
        Rhs::Binary(item1, item2) => {
            insert_rule(rhs_grammar, rule.weight, rhs, lhs, item1);
            insert_rule(rhs_grammar, rule.weight, rhs, lhs, item2);
        }
    }
}

fn insert_rule(
    rhs_grammar: &mut HashMap<Item, HashSet<Rule<Item>>>,
    weight: f64,
    rhs: Rhs<Item>,
    lhs: Item,
    item: &Item,
) {
    let set = rhs_grammar.entry(*item).or_default();
    set.insert(Rule { lhs, rhs, weight });
}

pub fn transform_sentence(
    line: &str,
    lexicon: &StringLookup,
    unking: &bool,
    smoothing: &bool,
) -> Vec<Item> {
    line.split_whitespace()
        .map(|word| {
            let word_id = match lexicon.get(word) {
                Some(u) => u,
                None => {
                    if *unking {
                        lexicon
                            .get("UNK")
                            .expect("UNK is not in the lexicon. Did you use an unked input?")
                    } else if *smoothing {
                        lexicon.get(&smooth_word(word, true)).unwrap_or_else(|| {
                            lexicon.get("UNK-S").unwrap_or_else(|| {
                            eprintln!("{} is not in the lexicon. Use smoothed input or the input data is not sufficient", word);
                            exit(1);
                            })
                        })
                    } else {
                        eprintln!("'{}' is not in the lexicon. Maybe use unking", word);
                        exit(1)
                    }
                }
            } as u32;
            Item::Terminal(word_id)
        })
        .collect()
}

pub fn deduce(
    line: &[Item],
    rule_lookup: &[Vec<Rule<Item>>],
    scores: Option<&ViterbiScore>,
    start_item: Item,
    number_of_items: usize,
) -> WeightMap<f64> {
    let sentence_length = line.len();
    let mut queue = MaxQueue::new(number_of_items, sentence_length);
    let mut weight_map = WeightMap::with_capacity(number_of_items, sentence_length);
    for (index, word) in line.iter().enumerate() {
        for rule in rule_lookup
            .get(usize::from(*word))
            .expect("there is no rule that produces the word")
        {
            queue.push(
                Consequence {
                    start: index as u32,
                    item: rule.lhs,
                    end: (index + 1) as u32,
                    weight: rule.weight,
                },
                rule.weight,
            );
        }
    }
    while let Some(consequence) = queue.pop(|idx| !weight_map.index_is_set(idx)) {
        weight_map.set(consequence);
        if consequence.start == 0
            && consequence.end == sentence_length as u32
            && consequence.item == start_item
        {
            break;
        }
        // iterate over all rules with the item on the right
        for rule in rule_lookup
            .get(usize::from(consequence.item))
            .expect("there should be a rule with each nonterminal")
        {
            match rule.rhs {
                Rhs::Unary(_) => {
                    add_replace(&mut queue, rule, &consequence, scores);
                }
                Rhs::Binary(item1, item2) => {
                    add_left(
                        &mut queue,
                        &weight_map,
                        rule,
                        (item1, item2),
                        &consequence,
                        scores,
                    );
                    add_right(
                        &mut queue,
                        &weight_map,
                        rule,
                        (item1, item2),
                        &consequence,
                        scores,
                    );
                }
            }
        }
    }
    weight_map
}

fn add_replace(
    queue: &mut MaxQueue,
    rule: &Rule<Item>,
    consequence: &Consequence,
    scores: Option<&ViterbiScore>,
) {
    // if there is a rule with the item on the right side replace it with the left side
    let weight = consequence.weight * rule.weight;
    let key = if let Some(scores) = scores {
        weight * scores.get_outside(rule.lhs)
    } else {
        weight
    };
    queue.push(
        Consequence {
            start: consequence.start,
            item: rule.lhs,
            end: consequence.end,
            weight,
        },
        key,
    )
}

fn add_right(
    queue: &mut MaxQueue,
    weight_map: &WeightMap<f64>,
    rule: &Rule<Item>,
    rhs: (Item, Item),
    consequence: &Consequence,
    scores: Option<&ViterbiScore>,
) {
    // if there is a rule with the item last
    // add all consequences to the queue where the sequence of items is in the weight map
    // such that item1.end == item2.start, item2.end == item3.start ...
    // then Consequence {start: item1.start, item: lhs, end: itemn.end } is added
    if consequence.item == rhs.1 {
        for next in weight_map.get_ends_at(rhs.0, consequence.start) {
            let weight = next.weight * consequence.weight * rule.weight;
            let key = if let Some(scores) = scores {
                weight * scores.get_outside(rule.lhs)
            } else {
                weight
            };
            queue.push(
                Consequence {
                    start: next.start,
                    item: rule.lhs,
                    end: consequence.end,
                    // always multiply left with right to preserve same value
                    weight,
                },
                key,
            );
        }
    }
}

fn add_left(
    queue: &mut MaxQueue,
    weight_map: &WeightMap<f64>,
    rule: &Rule<Item>,
    rhs: (Item, Item),
    consequence: &Consequence,
    scores: Option<&ViterbiScore>,
) {
    // if there is a rule with the item first
    // add all consequences to the queue where the sequence of items is in the weight map
    // such that item1.end == item2.start, item2.end == item3.start ...
    // then Consequence {start: item1.start, item: lhs, end: itemn.end } is added
    if consequence.item == rhs.0 {
        for next in weight_map.get_starts_at(rhs.1, consequence.end) {
            let weight = consequence.weight * next.weight * rule.weight;
            let key = if let Some(scores) = scores {
                weight * scores.get_outside(rule.lhs)
            } else {
                weight
            };
            queue.push(
                Consequence {
                    start: consequence.start,
                    item: rule.lhs,
                    end: next.end,
                    // always multiply left with right to preserve same value
                    weight,
                },
                key,
            );
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
        insert_into_lookup(
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
        insert_into_lookup(
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
        insert_into_lookup(
            &mut string_map,
            false,
            &mut rhs_grammar,
            &mut all_rules,
            lexicon_line,
        );
        let rule_line = "A -> B C 0.57".to_string();
        insert_into_lookup(
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
            insert_into_lookup(&mut string_map, false, &mut grammar, &mut all_rules, line);
        }
        let rules = vec![
            "ROOT -> W1 W2 0.25".to_string(),
            "ROOT -> W2 W2 0.75".to_string(),
            "W1 -> W2 0.6".to_string(),
        ];
        for line in rules {
            insert_into_lookup(&mut string_map, true, &mut grammar, &mut all_rules, line);
        }
        let initial = Item::NonTerminal(string_map.get("ROOT").unwrap() as u32);
        grammar.entry(initial).or_default();
        let mut rule_lookup_vec = vec![vec![]; grammar.len()];
        for (item, set) in grammar {
            rule_lookup_vec[u32::from(item) as usize] = set.into_iter().collect()
        }

        let line = transform_sentence("R S T", &string_map, &false, &false);
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
        let weight_map = deduce(&line, &rule_lookup_vec, initial, string_map.len());
        assert_eq!(weight_map, desired_weight_map);
    }
}

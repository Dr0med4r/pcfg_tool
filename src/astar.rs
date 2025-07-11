use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

use foldhash::{HashMap, HashMapExt};
use ordered_float::NotNan;

use crate::parse::{
    parse_rules,
    rule::{Rhs, Rule},
    string_lookup::StringLookup,
    weight_map::Item,
};

pub fn out(rules: &Path, lexicon: &Path, grammar: &Option<String>, initial_nonterminal: &str) {
    let mut weights_location = match grammar {
        Some(location) => Box::new(
            File::create(format!("{}.outside", location))
                .expect("GRAMMAR.outside is not a correct location"),
        ) as Box<dyn Write>,
        None => Box::new(io::stdout()) as Box<dyn Write>,
    };

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
    let all_items: Vec<Item> = rule_lookup.keys().map(Item::clone).collect();
    let mut rule_lookup_vec = vec![vec![]; rule_lookup.len()];
    for (item, set) in rule_lookup {
        rule_lookup_vec[u32::from(item) as usize] = set.into_iter().collect()
    }
    let initial_nonterminal = string_lookup
        .get(initial_nonterminal)
        .expect("initial nonterminal not in grammar");

    let mut score = ViterbiScore::new(all_rules, rule_lookup_vec, all_items, initial_nonterminal);
    score.calculate_outside();
    score.print_weights(&mut weights_location, string_lookup);
}

pub struct ViterbiScore {
    r#in: Vec<f64>,
    out: Vec<f64>,
    all_rules: HashMap<Item, HashMap<Rhs<Item>, f64>>,
    rule_lookup: Vec<Vec<Rule<Item>>>,
    all_nonterminals: Vec<Item>,
}

impl ViterbiScore {
    fn new(
        all_rules: HashMap<Item, HashMap<Rhs<Item>, f64>>,
        rule_lookup: Vec<Vec<Rule<Item>>>,
        all_items: Vec<Item>,
        initial_nonterminal: usize,
    ) -> Self {
        let len = all_items.len();
        let mut out = vec![0f64; len];
        let all_items = all_items
            .into_iter()
            .filter(|e| match *e {
                Item::NonTerminal(_) => true,
                Item::Terminal(_) => false,
            })
            .collect();
        out[initial_nonterminal] = 1f64;
        Self {
            r#in: vec![0f64; len],
            out,
            all_rules,
            rule_lookup,
            all_nonterminals: all_items,
        }
    }

    pub fn new_from_file(path: &Path, string_lookup: &StringLookup) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut scores = Self {
            r#in: vec![],
            out: vec![0f64; string_lookup.len()],
            all_rules: HashMap::default(),
            rule_lookup: vec![],
            all_nonterminals: vec![],
        };
        for line in io::BufReader::new(file).lines().map_while(Result::ok) {
            let (item, score) = line
                .split_once(" ")
                .expect("Could not parse the .outside file!");
            let score = score
                .parse::<f64>()
                .expect("Could not convert score to float!");
            let item = string_lookup
                .get(item)
                .expect("The item is not in rules did you use the correct files?");
            scores.out[item] = score;
        }
        Ok(scores)
    }

    pub fn get_outside(&self, item: Item) -> f64 {
        self.out[usize::from(item)]
    }

    fn get_inside(&self, item: Item) -> f64 {
        self.r#in[usize::from(item)]
    }

    fn calculate_inside(&mut self) {
        for item in &self.all_nonterminals {
            let item_pos = usize::from(*item);
            let rules_with_item = self.all_rules.get(item);
            if rules_with_item.is_none() {
                eprintln!("why??");
                self.r#in[item_pos] = 0f64;
            } else {
                self.r#in[item_pos] = rules_with_item
                    .unwrap()
                    .values()
                    .map(|f| NotNan::new(*f).unwrap())
                    .max()
                    .unwrap_or_else(|| {
                        eprintln!("why??");
                        NotNan::new(0f64).unwrap()
                    })
                    .into();
            }
        }
        let mut changed = true;
        while changed {
            changed = false;
            for item in &self.all_nonterminals {
                let item_pos = usize::from(*item);
                let mut weight = 0f64;
                for rule in &self.rule_lookup[item_pos] {
                    let new_weight = self
                        .all_rules
                        .get(&rule.lhs)
                        .unwrap()
                        .get(&rule.rhs)
                        .unwrap()
                        * match rule.rhs {
                            Rhs::Unary(item) => self.get_inside(item),
                            Rhs::Binary(first, second) => {
                                self.get_inside(first) * self.get_inside(second)
                            }
                        };
                    if new_weight > weight {
                        weight = new_weight;
                    }
                }
                if weight > self.get_inside(*item) {
                    self.r#in[item_pos] = weight;
                    changed = true;
                }
            }
        }
    }

    fn calculate_outside(&mut self) {
        self.calculate_inside();
        let mut changed = true;
        while changed {
            changed = false;
            for item in &self.all_nonterminals {
                let item_pos = usize::from(*item);
                let mut weight = 0f64;
                for rule in &self.rule_lookup[item_pos] {
                    let new_weight = self.get_outside(rule.lhs)
                    // inside of children
                        * match rule.rhs {
                            Rhs::Unary(_) => 1f64,
                            Rhs::Binary(first, second) => {
                                if first == *item {
                                    self.get_inside(second)
                                } else {
                                    self.get_inside(first)
                                }
                            }
                        }
                    // weight of rule
                        * self
                            .all_rules
                            .get(&rule.lhs)
                            .unwrap()
                            .get(&rule.rhs)
                            .unwrap();
                    if new_weight > weight {
                        weight = new_weight;
                    }
                }
                if weight > self.out[item_pos] {
                    self.out[item_pos] = weight;
                    changed = true;
                }
            }
        }
    }

    fn print_weights(&self, weights_location: &mut Box<dyn Write>, string_lookup: StringLookup) {
        for item in &self.all_nonterminals {
            let item_string = string_lookup
                .get_string(usize::from(*item))
                .expect("should not be possible");
            let weight = self.get_outside(*item);
            writeln!(weights_location, "{} {}", item_string, weight)
                .expect("could not write to the .outside file");
        }
    }
}

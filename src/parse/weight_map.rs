use std::collections::VecDeque;

use foldhash::HashMap;

use crate::induce::parse_tree::ParseTree;

use super::{consequence::Consequence, rule::Rhs, string_lookup::StringLookup};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub enum Item {
    NonTerminal(u32),
    Terminal(u32),
}

impl From<Item> for u32 {
    fn from(val: Item) -> Self {
        match val {
            Item::NonTerminal(u) => u,
            Item::Terminal(u) => u,
        }
    }
}
impl From<Item> for usize {
    fn from(val: Item) -> Self {
        match val {
            Item::NonTerminal(u) => u as usize,
            Item::Terminal(u) => u as usize,
        }
    }
}

pub fn triangle_index(triangle_length: u32, triangle: u32, i: u32, j: u32) -> usize {
    ((triangle * elements(triangle_length))
        + elements(triangle_length - i - 1)
        + (triangle_length - j)) as usize
}

fn elements(len: u32) -> u32 {
    (len * (len + 1)) / 2
}

pub struct WeightMapIterator<'a> {
    data: &'a Vec<f64>,
    map: &'a WeightMap<f64>,
    sentence_length: u32,
    /// the item over which to iterate
    item: Item,
    /// the fixed position of the value
    fixed: u32,
    /// the moving position of the value
    pos: u32,
    /// if the fixed position is the end or the start
    start: bool,
}

impl Iterator for WeightMapIterator<'_> {
    type Item = Consequence;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
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
                    u32::from(self.item),
                    self.fixed,
                    self.pos,
                )
            } else {
                triangle_index(
                    self.sentence_length,
                    u32::from(self.item),
                    self.pos,
                    self.fixed,
                )
            };
            let pos = self.pos;
            self.pos += 1;
            if self.map.index_is_set(index) {
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
pub struct WeightMap<T> {
    data: Vec<T>,
    map: Vec<u8>,
    sentence_length: u32,
}

impl<T> WeightMap<T>
where
    T: Default + Copy,
{
    pub fn with_capacity(items: usize, sentence_length: usize) -> Self {
        let length = items * elements(sentence_length as u32) as usize;
        let data = vec![T::default(); length];
        let length = length / 8 + 1;
        let map = vec![0; length];
        WeightMap {
            data,
            map,
            sentence_length: sentence_length as u32,
        }
    }

    fn index(&self, consequence: &Consequence) -> usize {
        // n: max_len index(a,b) = size(n-a-1) + n-b
        triangle_index(
            self.sentence_length,
            u32::from(consequence.item),
            consequence.start,
            consequence.end,
        )
    }

    pub fn index_is_set(&self, index: usize) -> bool {
        let (index, rem) = (index / 8, index % 8);
        self.map[index] & 1 << rem > 0
    }

    pub fn get_at_index(&self, index: usize) -> T {
        self.data[index]
    }
    pub fn get_with_index(&self, item: Item, start: u32, end: u32) -> T {
        assert!(start < end);
        assert!(end <= self.sentence_length);
        assert!(start < self.sentence_length);
        self.data[triangle_index(self.sentence_length, u32::from(item), start, end)]
    }

    pub fn set_index(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }
}

impl WeightMap<f64> {
    pub fn set(&mut self, consequence: Consequence) {
        let index = self.index(&consequence);
        let (map_index, rem) = (index / 8, index % 8);
        self.map[map_index] |= 1 << rem;
        self.data[index] = consequence.weight
    }

    pub fn get_starts_at(&self, item: Item, start: u32) -> impl Iterator<Item = Consequence> {
        WeightMapIterator {
            sentence_length: self.sentence_length,
            data: &self.data,
            map: self,
            item,
            fixed: start,
            pos: start + 1,
            start: true,
        }
    }

    pub fn get_ends_at(&self, item: Item, end: u32) -> impl Iterator<Item = Consequence> {
        WeightMapIterator {
            sentence_length: self.sentence_length,
            data: &self.data,
            map: self,
            item,
            fixed: end,
            pos: 0,
            start: false,
        }
    }

    pub fn convert_to_parse_tree(
        &self,
        initial: Item,
        start: u32,
        end: u32,
        string_lookup: &StringLookup,
        all_rules: &HashMap<Item, HashMap<Rhs<Item>, f64>>,
        line: &mut VecDeque<Item>,
    ) -> ParseTree<String> {
        let root = string_lookup
            .get_string(usize::from(initial))
            .unwrap()
            .clone();
        let mut children = Vec::new();
        // check all rules if it is the rule applied
        'rule: for (rhs, weight_of_rule) in all_rules.get(&initial).unwrap() {
            match rhs {
                Rhs::Unary(rhs) => {
                    let weight_of_lhs = self.get_with_index(initial, start, end);
                    match rhs {
                        Item::NonTerminal(_) => {
                            let weight_of_rhs = self.get_with_index(*rhs, start, end);
                            if weight_of_rhs * weight_of_rule == weight_of_lhs {
                                let child = self.convert_to_parse_tree(
                                    *rhs,
                                    start,
                                    end,
                                    string_lookup,
                                    all_rules,
                                    line,
                                );
                                children.push(child);
                                break;
                            }
                        }
                        Item::Terminal(_) => {
                            if *weight_of_rule == weight_of_lhs {
                                let child = ParseTree {
                                    root: string_lookup
                                        .get_string(usize::from(line.pop_front().unwrap()))
                                        .unwrap()
                                        .clone(),
                                    children: vec![],
                                };
                                children.push(child);
                                break;
                            }
                        }
                    }
                    continue;
                }
                Rhs::Binary(item1, item2) => {
                    for partition in start + 1..end {
                        let l = self.get_with_index(*item1, start, partition);
                        let r = self.get_with_index(*item2, partition, end);
                        if l * r * weight_of_rule == self.get_with_index(initial, start, end) {
                            children.push(self.convert_to_parse_tree(
                                *item1,
                                start,
                                partition,
                                string_lookup,
                                all_rules,
                                line,
                            ));
                            children.push(self.convert_to_parse_tree(
                                *item2,
                                partition,
                                end,
                                string_lookup,
                                all_rules,
                                line,
                            ));
                            break 'rule;
                        }
                    }
                }
            }
        }
        if children.is_empty() {
            children.push(ParseTree {
                root: "ERROR".to_string(),
                children: vec![],
            });
            // panic!("convert probably called on unparsable tree")
        }
        ParseTree { root, children }
        // let mut tree = ParseTree::default();
        // tree.root = string_lookup.get_string(usize::from(initial));
        // let a = vec!["a"];
        // tree.children = a;
    }
}

#[cfg(test)]
mod test {
    use foldhash::HashMapExt;

    use crate::parse::{deduce, insert_rule_into_map, transform_sentence};

    use super::*;

    #[test]
    fn weightmap_test() {
        const RULES: u32 = 4;
        const SENTENCE: u32 = 4;
        let mut weight_map = WeightMap::with_capacity(RULES as usize, SENTENCE as usize);
        for rule in 0..RULES {
            for x in 0..SENTENCE {
                for y in x + 1..=SENTENCE {
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
                    let value = weight_map.get_with_index(Item::NonTerminal(rule), x, y);
                    assert_eq!(value, rule as f64 + x as f64 + y as f64 / (3.0 * 3.0 * 4.0));
                }
            }
        }
    }

    #[test]
    fn weightmap_starts_at_test() {
        const RULES: u32 = 4;
        const SENTENCE: u32 = 4;
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
        const RULES: u32 = 4;
        const SENTENCE: u32 = 4;
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

    #[test]
    fn weightmap_to_parsetree() {
        let mut string_map = StringLookup::default();
        let mut grammar = HashMap::new();
        let mut all_rules = HashMap::new();
        let lexicon = vec![
            "W1 R 0.1".to_string(),
            "W2 S 1".to_string(),
            "W1 T 0.3".to_string(),
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

        let line = transform_sentence("T S", &string_map);
        let weight_map = deduce(&line, &grammar, initial, string_map.len());
        let tree = weight_map.convert_to_parse_tree(
            initial,
            0,
            line.len() as u32,
            &string_map,
            &all_rules,
            &mut line.into(),
        );
        let desired_tree = ParseTree {
            root: "ROOT".to_string(),
            children: vec![
                ParseTree {
                    root: "W1".to_string(),
                    children: vec![ParseTree {
                        root: "T".to_string(),
                        children: vec![],
                    }],
                },
                ParseTree {
                    root: "W2".to_string(),
                    children: vec![ParseTree {
                        root: "S".to_string(),
                        children: vec![],
                    }],
                },
            ],
        };
        assert_eq!(desired_tree, tree);
    }

    #[ignore]
    #[test]
    #[should_panic]
    fn convert_to_parse_panic_test() {
        let mut string_map = StringLookup::default();
        let mut grammar = HashMap::new();
        let mut all_rules = HashMap::new();
        let lexicon = vec![
            "W1 R 0.1".to_string(),
            "W2 S 1".to_string(),
            "W1 T 0.3".to_string(),
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
        let weight_map = deduce(&line, &grammar, initial, string_map.len());
        weight_map.convert_to_parse_tree(
            initial,
            0,
            line.len() as u32,
            &string_map,
            &all_rules,
            &mut line.into(),
        );
    }
}

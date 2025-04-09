use std::{
    collections::HashMap,
    io::{self, Write},
};

use parse_tree::ParseTree;

pub mod parse_tree;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Rhs {
    Terminal(String),
    NonTerminals(Vec<String>),
}

fn create_grammar(grammar: &mut HashMap<String, HashMap<Rhs, u64>>, tree: ParseTree<&str>) {
    tree.execute_for_nodes(&mut |node| {
        if node.is_leaf() {
            return;
        }
        let non_terminal = node.root.to_string();
        let body: &mut HashMap<Rhs, u64> = match grammar.get_mut(&non_terminal) {
            Some(rhs) => rhs,
            None => {
                grammar.insert(non_terminal.clone(), HashMap::new());
                grammar
                    .get_mut(&non_terminal)
                    .expect("just inserted the value that is asked")
            }
        };
        let child = node.children.first().expect("node should not be a leaf");
        // assumes that if the child is a leaf it is also a terminal
        let lhs = if child.is_leaf() {
            Rhs::Terminal(child.root.to_string())
        } else {
            let child_names: Vec<String> = node
                .children
                .iter()
                .map(|child| child.root.to_string())
                .collect();
            Rhs::NonTerminals(child_names)
        };
        let probability: u64 = match body.get(&lhs) {
            Some(&probability) => probability + 1,
            None => 1,
        };
        body.insert(lhs, probability);
    });
}

fn transform_grammar(
    absolute_grammar: HashMap<String, HashMap<Rhs, u64>>,
) -> HashMap<String, HashMap<Rhs, f64>> {
    let mut grammar = HashMap::new();
    for (non_terminal, body) in absolute_grammar {
        let mut new_body = HashMap::new();
        let total = body.values().sum::<u64>() as f64;
        for (item, count) in body {
            new_body.insert(item, count as f64 / total);
        }
        grammar.insert(non_terminal, new_body);
    }
    grammar
}

pub fn induce_grammar() -> HashMap<String, HashMap<Rhs, f64>> {
    let mut absolute_grammar: HashMap<String, HashMap<Rhs, u64>> = HashMap::new();
    for line in io::stdin().lines() {
        let Ok(line) = line else { continue };
        let Ok((_, tree)) = parse_tree::element(&line) else {
            continue;
        };
        create_grammar(&mut absolute_grammar, tree);
    }
    transform_grammar(absolute_grammar)
}

pub fn write_grammar(
    rules: &mut Box<dyn Write>,
    lexicon: &mut Box<dyn Write>,
    words: &mut Box<dyn Write>,
    grammar: &HashMap<String, HashMap<Rhs, f64>>,
) {
    for (non_terminal, value) in grammar {
        for (body, probability) in value {
            match body {
                Rhs::Terminal(terminal) => {
                    writeln!(lexicon, "{} {} {}", non_terminal, terminal, probability)
                        .expect("cannot write to lexicon");
                    writeln!(words, "{}", terminal).expect("cannot write to words");
                }
                Rhs::NonTerminals(non_terminals) => {
                    writeln!(
                        rules,
                        "{} -> {} {}",
                        non_terminal,
                        non_terminals.join(" "),
                        probability
                    )
                    .expect("cannot write to rules");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_grammar_test() {
        let tree = ParseTree {
            root: "ROOT",
            children: vec![ParseTree {
                root: "NS",
                children: vec![ParseTree {
                    root: "hi",
                    children: vec![],
                }],
            }],
        }; // (ROOT (NS hi))
        let mut grammar: HashMap<String, HashMap<Rhs, u64>> = HashMap::new();
        create_grammar(&mut grammar, tree);
        assert_eq!(
            grammar,
            HashMap::from([
                (
                    "NS".to_string(),
                    HashMap::from([(Rhs::Terminal("hi".to_string()), 1)])
                ),
                (
                    "ROOT".to_string(),
                    HashMap::from([(Rhs::NonTerminals(vec!["NS".to_string()]), 1)])
                )
            ])
        );
    }

    #[test]
    fn create_grammar_test_2() {
        let tree = ParseTree {
            root: "ROOT",
            children: vec![
                ParseTree {
                    root: "NS",
                    children: vec![ParseTree {
                        root: "hi",
                        children: vec![],
                    }],
                },
                ParseTree {
                    root: "NS",
                    children: vec![ParseTree {
                        root: "ho",
                        children: vec![],
                    }],
                },
            ],
        }; // (ROOT (NS hi) (NS ho))
        let mut grammar: HashMap<String, HashMap<Rhs, u64>> = HashMap::new();
        create_grammar(&mut grammar, tree);
        assert_eq!(
            grammar,
            HashMap::from([
                (
                    "NS".to_string(),
                    HashMap::from([
                        (Rhs::Terminal("hi".to_string()), 1),
                        (Rhs::Terminal("ho".to_string()), 1),
                    ])
                ),
                (
                    "ROOT".to_string(),
                    HashMap::from([(
                        Rhs::NonTerminals(vec!["NS".to_string(), "NS".to_string()]),
                        1
                    )])
                )
            ])
        );
    }

    #[test]
    fn transform_grammar_test() {
        let grammar = HashMap::from([
            (
                "NS".to_string(),
                HashMap::from([
                    (Rhs::Terminal("hi".to_string()), 1),
                    (Rhs::Terminal("ho".to_string()), 2),
                ]),
            ),
            (
                "ROOT".to_string(),
                HashMap::from([(
                    Rhs::NonTerminals(vec!["NS".to_string(), "NS".to_string()]),
                    1,
                )]),
            ),
        ]); // (ROOT (NS hi) (NS ho))
        assert_eq!(
            transform_grammar(grammar),
            HashMap::from([
                (
                    "NS".to_string(),
                    HashMap::from([
                        (Rhs::Terminal("hi".to_string()), 1.0 / 3.0),
                        (Rhs::Terminal("ho".to_string()), 2.0 / 3.0),
                    ])
                ),
                (
                    "ROOT".to_string(),
                    HashMap::from([(
                        Rhs::NonTerminals(vec!["NS".to_string(), "NS".to_string()]),
                        1.0
                    )])
                )
            ])
        );
        let grammar = HashMap::from([
            (
                "NS".to_string(),
                HashMap::from([
                    (Rhs::Terminal("hi".to_string()), 1),
                    (Rhs::Terminal("ho".to_string()), 1),
                ]),
            ),
            (
                "ROOT".to_string(),
                HashMap::from([(
                    Rhs::NonTerminals(vec!["NS".to_string(), "NS".to_string()]),
                    1,
                )]),
            ),
        ]); // (ROOT (NS hi) (NS ho))
        assert_eq!(
            transform_grammar(grammar),
            HashMap::from([
                (
                    "NS".to_string(),
                    HashMap::from([
                        (Rhs::Terminal("hi".to_string()), 0.5),
                        (Rhs::Terminal("ho".to_string()), 0.5),
                    ])
                ),
                (
                    "ROOT".to_string(),
                    HashMap::from([(
                        Rhs::NonTerminals(vec!["NS".to_string(), "NS".to_string()]),
                        1.0
                    )])
                )
            ]));
    }
}

use std::{collections::HashMap, io};

use parse_tree::ParseTree;

pub mod parse_tree;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Rhs {
    Terminal(String),
    NonTerminals(Vec<String>),
}

fn create_grammar(grammar: &mut HashMap<String, HashMap<Rhs, f64>>, tree: ParseTree<&str>) {
    tree.execute_for_nodes(&mut |node| {
        if node.is_leaf() {
            return;
        }
        let non_terminal = node.root.to_string();
        let body: &mut HashMap<Rhs, f64> = match grammar.get_mut(&non_terminal) {
            Some(rhs) => rhs,
            None => {
                grammar.insert(non_terminal.clone(), HashMap::new());
                grammar
                    .get_mut(&non_terminal)
                    .expect("just inserted the value that is asked")
            }
        };
        let child = node.children.first().expect("node should not be a leaf");
        let factor = (body.len() as f64) / (body.len() as f64 + 1.0);
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
        let probability: f64 = match body.get(&lhs) {
            Some(&probability) => probability * factor + 1.0 / (body.len() as f64 + 1.0),
            None => 1.0 / (body.len() as f64 + 1.0),
        };
        for v in (*body).values_mut() {
            *v *= factor;
        }
        body.insert(lhs, probability);
    });
}

pub fn induce_grammar() -> HashMap<String, HashMap<Rhs, f64>> {
    let mut grammar: HashMap<String, HashMap<Rhs, f64>> = HashMap::new();
    for line in io::stdin().lines() {
        let Ok(line) = line else { continue };
        let Ok((_, tree)) = parse_tree::element(&line) else {
            continue;
        };
        create_grammar(&mut grammar, tree);
    }
    grammar
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
        let mut grammar: HashMap<String, HashMap<Rhs, f64>> = HashMap::new();
        create_grammar(&mut grammar, tree);
        eprintln!("{:?}", grammar);
        assert_eq!(
            grammar,
            HashMap::from([
                (
                    "NS".to_string(),
                    HashMap::from([(Rhs::Terminal("hi".to_string()), 1.0)])
                ),
                (
                    "ROOT".to_string(),
                    HashMap::from([(Rhs::NonTerminals(vec!["NS".to_string()]), 1.0)])
                )
            ])
        );
    }

    #[test]
    fn create_grammar_probability_test() {
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
        let mut grammar: HashMap<String, HashMap<Rhs, f64>> = HashMap::new();
        create_grammar(&mut grammar, tree);
        eprintln!("{:?}", grammar);
        assert_eq!(
            grammar,
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
                    HashMap::from([(Rhs::NonTerminals(vec!["NS".to_string(), "NS".to_string()]), 1.0)])
                )
            ])
        );
    }
}

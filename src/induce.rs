use std::{collections::HashMap, io};

pub mod parse_tree;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Rhs {
    Terminal(String),
    NonTerminals(Vec<String>),
}

pub fn induce_grammar() -> HashMap<String, HashMap<Rhs, f64>> {
    let mut a: HashMap<String, HashMap<Rhs, u64>> = HashMap::new();
    io::stdin().lines().for_each(|line| {
        if let Ok(line) = line {
            if let Ok((_, tree)) = parse_tree::element(&line) {
                // println!("{}", line);
                tree.execute_for_nodes(&mut |node| {
                    if node.is_leaf() {
                        return;
                    }
                    let lhs = node.root.to_string();
                    let rhs: &mut HashMap<Rhs, u64> = match a.get_mut(&lhs) {
                        Some(rhs) => rhs,
                        None => {
                            a.insert(lhs.clone(), HashMap::new());
                            a.get_mut(&lhs)
                                .expect("just inserted the value that is asked")
                        }
                    };
                    let Some(child) = node.children.first() else {
                        return;
                    };
                    // assumes that if the child is a leaf it is also a terminal
                    if child.is_leaf() {
                        let lhs = Rhs::Terminal(child.root.to_string());
                        let count = match rhs.get(&lhs) {
                            Some(&count) => count + 1,
                            None => 1,
                        };
                        rhs.insert(lhs, count);
                    } else {
                        let child_names: Vec<String> = node
                            .children
                            .iter()
                            .map(|child| child.root.to_string())
                            .collect();
                        let lhs = Rhs::NonTerminals(child_names);
                        let count = match rhs.get(&lhs) {
                            Some(&count) => count + 1,
                            None => 1,
                        };
                        rhs.insert(lhs, count);
                    }
                });
            }
        }
    });
    todo!()
}

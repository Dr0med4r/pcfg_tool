use std::{io, process::exit};

use crate::induce::parse_tree::{ParseTree, element};

pub fn debinarise() {
    for (line_number, line) in io::stdin().lines().enumerate() {
        let Ok(line) = line else { continue };
        let transformed_line = debinarise_line(line);
        if let Some(transformed_line) = transformed_line {
            println!("{}", transformed_line);
        } else {
            eprintln!("error while transforming line {}", line_number + 1);
            exit(1);
        }
    }
}

fn debinarise_line(line: String) -> Option<String> {
    let Ok((remaining, tree)) = element(&line) else {
        return None;
    };
    if remaining.trim() != "" {
        return None;
    }

    let transformed_tree = debinarise_tree(tree);

    Some(transformed_tree.to_string())
}

/// returns the string without extras or `None` if it is already without
fn demarkovize(string: &str) -> Option<&str> {
    let without_parents = if string.find('^').is_some() {
        string.split('^').next().unwrap()
    } else {
        string
    };
    if without_parents.find('|').is_some() {
        // is always Some
        string.split('|').next()
    } else {
        None
    }
}

fn debinarise_tree(tree: ParseTree<&str>) -> ParseTree<String> {
    let root = if tree.root.find('^').is_some() {
        tree.root.split('^').next().unwrap().to_string()
    } else {
        tree.root.to_string()
    };
    let root = if root.find('|').is_some() {
        root.split('|').next().unwrap().to_string()
    } else {
        root
    };
    let mut transformed_tree = ParseTree::new(root);
    for child in tree.children {
        let demarkovize = demarkovize(child.root);
        if demarkovize == Some(&transformed_tree.root) {
            for child in debinarise_tree_remove(child, &transformed_tree.root) {
                transformed_tree.children.push(child);
            }
        } else {
            transformed_tree.children.push(debinarise_tree(child));
        }
    }
    transformed_tree
}

fn debinarise_tree_remove(tree: ParseTree<&str>, root: &str) -> Vec<ParseTree<String>> {
    let mut children = vec![];
    for child in tree.children {
        if demarkovize(child.root) == Some(root) {
            for child in debinarise_tree_remove(child, root) {
                children.push(child);
            }
        } else {
            children.push(debinarise_tree(child));
        }
    }
    children
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::induce::parse_tree::ParseTree;

    #[test]
    fn test_debinarise_tree() {
        let test_tree = ParseTree {
            root: "ROOT",
            children: vec![ParseTree {
                root: "FRAG^<ROOT>",
                children: vec![
                    ParseTree {
                        root: "RB",
                        children: vec![ParseTree::new("Not")],
                    },
                    ParseTree {
                        root: "FRAG|<NP-TMP,.>^<ROOT>",
                        children: vec![
                            ParseTree {
                                root: "NP-TMP^<FRAG,ROOT>",
                                children: vec![
                                    ParseTree {
                                        root: "DT",
                                        children: vec![ParseTree::new("this")],
                                    },
                                    ParseTree {
                                        root: "NN",
                                        children: vec![ParseTree::new("year")],
                                    },
                                ],
                            },
                            ParseTree {
                                root: ".",
                                children: vec![ParseTree::new(".")],
                            },
                        ],
                    },
                ],
            }],
        };
        let expected_tree = ParseTree {
            root: "ROOT".to_string(),
            children: vec![ParseTree {
                root: "FRAG".to_string(),
                children: vec![
                    ParseTree {
                        root: "RB".to_string(),
                        children: vec![ParseTree::new("Not".to_string())],
                    },
                    ParseTree {
                        root: "NP-TMP".to_string(),
                        children: vec![
                            ParseTree {
                                root: "DT".to_string(),
                                children: vec![ParseTree::new("this".to_string())],
                            },
                            ParseTree {
                                root: "NN".to_string(),
                                children: vec![ParseTree::new("year".to_string())],
                            },
                        ],
                    },
                    ParseTree {
                        root: ".".to_string(),
                        children: vec![ParseTree::new(".".to_string())],
                    },
                ],
            }],
        };
        let correct = debinarise_tree(test_tree);
        assert_eq!(correct, expected_tree);
    }
}

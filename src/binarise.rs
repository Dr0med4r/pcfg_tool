use std::{cmp::max, io, process::exit};

use crate::induce::parse_tree::{ParseTree, element};

pub fn binarise(horizontal: u64, vertical: u64) {
    for (line_number, line) in io::stdin().lines().enumerate() {
        let Ok(line) = line else { continue };
        let transformed_line = binarise_line(line, horizontal, vertical);
        if let Some(transformed_line) = transformed_line {
            println!("{}", transformed_line);
        } else {
            eprintln!("error while transforming line {}", line_number + 1);
            exit(1);
        }
    }
}

fn binarise_line(line: String, horizontal: u64, vertical: u64) -> Option<String> {
    let Ok((remaining, tree)) = element(&line) else {
        return None;
    };
    if remaining.trim() != "" {
        return None;
    }

    let transformed_tree = binarise_tree(tree, horizontal, vertical, &[], true);

    Some(transformed_tree.to_string())
}

fn binarise_tree(
    mut tree: ParseTree<&str>,
    horizontal: u64,
    vertical: u64,
    parents: &[String],
    original: bool,
) -> ParseTree<String> {
    let root = tree.root.to_string();
    let mut transformed_tree = ParseTree::new(root.clone());

    // node is preterminal
    if tree.children.len() == 1 && tree.children[0].is_leaf() {
        transformed_tree
            .children
            .push(ParseTree::new(tree.children[0].root.to_string()));
        return transformed_tree;
    }
    if original {
        transformed_tree.root += &add_parents(parents, vertical);
    } else {
        // when origin = false parents is at least one long because it is at least a child
        transformed_tree.root += &add_parents(&parents[..parents.len() - 1], vertical);
    }

    let mut parents = parents.to_owned();
    if original {
        parents.push(root.clone());
    }
    if tree.children.len() > 2 {
        let first = tree.children.remove(0);
        let remaining = tree.children;
        transformed_tree
            .children
            .push(binarise_tree(first, horizontal, vertical, &parents, true));
        transformed_tree
            .children
            .push(squash_children(remaining, horizontal, vertical, &parents));
    } else {
        for child in tree.children {
            transformed_tree
                .children
                .push(binarise_tree(child, horizontal, vertical, &parents, true));
        }
    }

    transformed_tree
}

fn squash_children(
    children: Vec<ParseTree<&str>>,
    horizontal: u64,
    vertical: u64,
    parents: &[String],
) -> ParseTree<String> {
    let start = max(children.len() as i64 - horizontal as i64, 0) as usize;
    let original = parents[parents.len() - 1].clone();
    let mut node = original.clone() + "|<";
    node += &children[start..]
        .iter()
        .map(|e| e.root.to_string())
        .reduce(|a, e| a + "," + &e)
        .unwrap();
    node += ">";
    let mut tree = if horizontal == 1 {
        ParseTree::new(&original[..])
    } else {
        ParseTree::new(&node[..])
    };
    tree.children = children;
    binarise_tree(tree, horizontal, vertical, parents, false)
}

fn add_parents(parents: &[String], vertical: u64) -> String {
    if parents.is_empty() || vertical == 1 {
        return "".into();
    }
    let start = max(parents.len() as i64 - vertical as i64, 0) as usize;
    let mut node = "^<".to_string();

    node += &parents[start..]
        .iter()
        .map(|e| e.to_owned())
        .rev()
        .reduce(|a, e| a + "," + &e)
        .unwrap();
    node += ">";
    node
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::induce::parse_tree::ParseTree;

    #[test]
    fn test_binarise_tree() {
        let test_tree = ParseTree {
            root: "ROOT",
            children: vec![ParseTree {
                root: "FRAG",
                children: vec![
                    ParseTree {
                        root: "RB",
                        children: vec![ParseTree::new("Not")],
                    },
                    ParseTree {
                        root: "NP-TMP",
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
            }],
        };
        let expected_tree = ParseTree {
            root: "ROOT".to_string(),
            children: vec![ParseTree {
                root: "FRAG^<ROOT>".to_string(),
                children: vec![
                    ParseTree {
                        root: "RB".to_string(),
                        children: vec![ParseTree::new("Not".to_string())],
                    },
                    ParseTree {
                        root: "FRAG|<NP-TMP,.>^<ROOT>".to_string(),
                        children: vec![
                            ParseTree {
                                root: "NP-TMP^<FRAG,ROOT>".to_string(),
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
                    },
                ],
            }],
        };
        let correct = binarise_tree(test_tree, 999, 3, &[], true);
        assert_eq!(correct, expected_tree);
    }
}

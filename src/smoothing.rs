use std::{char, io};

use foldhash::{HashMap, HashSet};

use crate::{induce::parse_tree::ParseTree, unk::get_tree_lines};

pub fn smooth(threshold: u64) {
    let mut word_count: HashMap<&str, u64> = HashMap::default();
    let string_lines: Vec<String> = io::stdin().lines().map_while(Result::ok).collect();
    let trees = get_tree_lines(&string_lines, &mut word_count);
    let unknown_words: HashSet<&str> = word_count
        .into_iter()
        .filter_map(
            |(word, count)| {
                if count <= threshold { Some(word) } else { None }
            },
        )
        .collect();
    for tree in trees {
        let smoothed_tree = smooth_tree(tree, &unknown_words, true);
        println!("{}", smoothed_tree);
    }
}

fn smooth_tree(
    tree: ParseTree<&str>,
    unknown_words: &HashSet<&str>,
    mut first: bool,
) -> ParseTree<String> {
    let mut smoothed_tree = ParseTree::new(tree.root.to_string());
    if tree.is_leaf() && unknown_words.contains(&tree.root) {
        smoothed_tree.root = smooth_word(tree.root, first);
    }

    for child in tree.children {
        smoothed_tree
            .children
            .push(smooth_tree(child, unknown_words, first));
        first = false
    }
    smoothed_tree
}

// checks the function test for every char of word
fn has_any<F>(word: &str, test: F) -> bool
where
    F: Fn(char) -> bool,
{
    word.chars().any(test)
}

pub fn smooth_word(word: &str, first: bool) -> String {
    let first_char = word.chars().next();

    // does this even happen?
    let letter_suffix = if word.is_empty() {
        return "UNK".to_string();
    } else if first_char.is_some_and(char::is_uppercase) && !has_any(word, char::is_lowercase) {
        // All Caps or numbers
        "-AC"
    } else if first_char.is_some_and(char::is_uppercase) {
        // Capital word
        if first { "-SC" } else { "-C" }
    } else if has_any(word, char::is_lowercase) {
        // has lowercase
        "-L"
    } else if has_any(word, char::is_alphabetic) {
        // has letters
        "-U"
    } else {
        // no letters
        "-S"
    };

    let number_suffix = if word.chars().all(char::is_numeric) {
        // is a number
        "-N"
    } else if has_any(word, char::is_numeric) {
        // contains digits
        "-n"
    } else {
        ""
    };

    let dash_suffix = if has_any(word, |e| e == '-') {
        // contains a dash
        "-H"
    } else {
        ""
    };
    let period_suffix = if has_any(word, |e| e == '.') {
        // contains a dot
        "-P"
    } else {
        ""
    };

    let comma_suffix = if has_any(word, |e| e == ',') {
        // contains a comma
        "-C"
    } else {
        ""
    };

    let last = word.chars().last();
    let word_suffix = if word.len() > 3 && last.unwrap().is_alphabetic() {
        // add the last character if it is a letter
        &("-".to_string() + &last.unwrap().to_lowercase().to_string())[..]
    } else {
        ""
    };

    "UNK".to_string()
        + letter_suffix
        + number_suffix
        + dash_suffix
        + period_suffix
        + comma_suffix
        + word_suffix
}

#[cfg(test)]
mod test {
    use super::*;
    const UNKNOWN_WORDS: [&str; 7] = ["test1", "1984", "CAPS", "a,", "a.", "long_word", "Capital"];
    const TRANSLATED_WORDS: [&str; 7] = [
        "UNK-L-n", "UNK-S-N", "UNK-AC-s", "UNK-L-C", "UNK-L-P", "UNK-L-d", "UNK-SC-l",
    ];

    #[test]
    fn test_some_input() {
        let unknown_words = HashSet::from_iter(UNKNOWN_WORDS);
        for (input, output) in UNKNOWN_WORDS.into_iter().zip(TRANSLATED_WORDS) {
            let input_tree = ParseTree {
                root: "ROOT",
                children: vec![ParseTree::new(input), ParseTree::new("test")],
            };
            let desired_tree = ParseTree {
                root: "ROOT".to_string(),
                children: vec![
                    ParseTree::new(output.to_string()),
                    ParseTree::new("test".to_string()),
                ],
            };

            let tree = smooth_tree(input_tree, &unknown_words, true);
            assert_eq!(desired_tree, tree);
        }
    }

    #[test]
    fn test_first_capital() {
        let unknown_words = HashSet::from_iter(["Try"]);
        let input_tree = ParseTree {
            root: "ROOT",
            children: vec![ParseTree::new("Try"), ParseTree::new("test")],
        };
        let desired_tree = ParseTree {
            root: "ROOT".to_string(),
            children: vec![
                ParseTree::new("UNK-SC".to_string()),
                ParseTree::new("test".to_string()),
            ],
        };
        let tree = smooth_tree(input_tree, &unknown_words, true);
        assert_eq!(desired_tree, tree);

        let input_tree = ParseTree {
            root: "ROOT",
            children: vec![ParseTree::new("test"), ParseTree::new("Try")],
        };
        let desired_tree = ParseTree {
            root: "ROOT".to_string(),
            children: vec![
                ParseTree::new("test".to_string()),
                ParseTree::new("UNK-C".to_string()),
            ],
        };
        let tree = smooth_tree(input_tree, &unknown_words, true);
        assert_eq!(desired_tree, tree);
    }
}

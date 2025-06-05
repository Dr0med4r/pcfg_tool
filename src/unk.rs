use std::{io, process::exit};

use foldhash::HashMap;

use crate::induce::parse_tree::element;

pub fn unk(threshold: u64) {
    let mut word_count: HashMap<&str, u64> = HashMap::default();
    let string_lines: Vec<String> = io::stdin().lines().map_while(Result::ok).collect();
    let mut lines = Vec::new();
    for (line_number, line) in string_lines.iter().enumerate() {
        let Ok((rem, tree)) = element(line) else {
            eprintln!("error while parsing line {}", line_number + 1);
            exit(1);
        };
        if rem.trim() != "" {
            eprintln!("line {} not completely parsed", line_number + 1);
            exit(1);
        }
        tree.execute_for_nodes(&mut |tree| {
            if tree.is_leaf() {
                let count: &mut u64 = word_count.entry(tree.root).or_default();
                *count += 1;
            }
        });
        lines.push(tree);
    }
    let unknown_words: Vec<&str> = word_count
        .into_iter()
        .filter_map(
            |(word, count)| {
                if count <= threshold { Some(word) } else { None }
            },
        )
        .collect();
    for mut line in lines {
        line.change_nodes(&mut |e| {
            if e.is_leaf() && unknown_words.contains(&e.root) {
                e.root = "UNK";
            }
        });
        println!("{}", line);
    }
}

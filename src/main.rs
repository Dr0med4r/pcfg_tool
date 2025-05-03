mod argparse;
mod induce;
mod parse;

use std::{
    fs::File,
    io::{self, Write},
    process::exit,
};

use argparse::{Args, Commands};
use clap::Parser;
use foldhash::{HashMap, HashMapExt};
use induce::{induce_grammar, write_grammar};
use parse::{
    deduce, parse_rules, string_lookup::StringLookup, transform_sentence, weight_map::Item,
};

fn main() {
    let args = Args::parse();
    match &args.command {
        Commands::Induce { grammar } => {
            induce(grammar);
        }
        Commands::Parse {
            rules,
            lexicon,
            paradigma,
            initial_nonterminal,
            unking,
            smoothing,
            threshold_beam,
            rank_beam,
            astar,
        } => {
            match paradigma {
                Some(paradigma) if paradigma == &"cyk".to_string() => exit(22),
                _ => {}
            }
            if *unking
                || *smoothing
                || threshold_beam.is_some()
                || rank_beam.is_some()
                || astar.is_some()
            {
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
            for (line_number, line) in io::stdin().lines().enumerate() {
                let Ok(line) = line else {
                    eprintln!("error reading line {}", line_number + 1);
                    exit(1);
                };
                let line_items = transform_sentence(&line, &string_lookup);
                let initial_nonterminal = Item::NonTerminal(
                    string_lookup
                        .get(initial_nonterminal)
                        .expect("initial nonterminal is not in the rules")
                        as u64,
                );
                rule_lookup.entry(initial_nonterminal).or_default();
                let rule_weights = deduce(
                    &line_items,
                    &rule_lookup,
                    initial_nonterminal,
                    string_lookup.len(),
                );
                if rule_weights.get_with_index(initial_nonterminal, 0, line_items.len() as u64)
                    == 0.0
                {
                    println!("(NOPARSE {})", line)
                } else {
                    let tree = rule_weights.convert_to_parse_tree(
                        initial_nonterminal,
                        0,
                        line_items.len() as u64,
                        &string_lookup,
                        &all_rules,
                        &mut line_items.into(),
                    );
                    println!("{tree}")
                }
            }
        }

        _ => {
            exit(22);
        }
    }
}

fn induce(grammar: &Option<String>) {
    let (mut rules, mut lexicon, mut words) = match grammar {
        Some(grammar_location) => {
            let rules_location = File::create(format!("{grammar_location}.rules"))
                .expect("GRAMMAR.rules is not a correct location");
            let lexicon_location = File::create(format!("{grammar_location}.lexicon"))
                .expect("GRAMMAR.lexicon is not a correct location");
            let words_location = File::create(format!("{grammar_location}.words"))
                .expect("GRAMMAR.words is not a correct location");
            (
                Box::new(rules_location) as Box<dyn Write>,
                Box::new(lexicon_location) as Box<dyn Write>,
                Box::new(words_location) as Box<dyn Write>,
            )
        }
        None => (
            Box::new(io::stdout()) as Box<dyn Write>,
            Box::new(io::stdout()) as Box<dyn Write>,
            Box::new(io::stdout()) as Box<dyn Write>,
        ),
    };
    let grammar = induce_grammar();
    write_grammar(&mut rules, &mut lexicon, &mut words, &grammar);
}

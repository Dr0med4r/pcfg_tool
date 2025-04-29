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
use induce::{induce_grammar, write_grammar};
use parse::{deduce, parse_rules};

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
            let mut all_rules = Vec::new();
            let rule_lookup = parse_rules(&mut all_rules, rules, true);
            let lexicon_lookup = parse_rules(&mut all_rules, lexicon, false);
            for (line_number, line) in io::stdin().lines().enumerate() {
                let Ok(line) = line else {
                    eprintln!("error in line: {}", line_number + 1);
                    exit(1);
                };
                let rule_weights = deduce(line, rule_lookup, lexicon_lookup);

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

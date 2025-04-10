mod argparse;
mod induce;

use std::{
    fs::File,
    io::{self, Write},
    process::exit,
};

use argparse::{Args, Commands};
use clap::Parser;
use induce::{induce_grammar, write_grammar};

fn main() {
    let args = Args::parse();
    match &args.command {
        Commands::Induce { grammar } => {
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
        _ => {
            exit(22);
        }
    }
}

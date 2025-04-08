mod argparse;
mod induce;

use std::process::exit;

use argparse::{Args, Commands};
use clap::Parser;
use induce::induce_grammar;

fn main() {
    let args = Args::parse();
    match &args.command {
        Commands::Induce { grammar } => {
            if let Some(grammar) = grammar {
                for x in induce_grammar() {
                    todo!();
                }
            }
            induce_grammar();
        }
        _ => {
            exit(22);
        }
    }
}

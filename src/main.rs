mod argparse;
mod induce;

use std::process::exit;

use argparse::{Args, Commands};
use clap::Parser;

fn main() {
    let args = Args::parse();
    match &args.command {
        Commands::Induce { grammar: _} => {
            println!("induce");               
        }
        _ => {
            exit(22);
        }
    }
}

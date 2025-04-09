use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    Induce {
        /// Sets the name of the Grammar files ([GRAMMAR].rules, [GRAMMAR].lexicon, [GRAMMAR].words)
        #[arg()]
        grammar: Option<String>,
    },
    Parse {
        #[arg(value_name = "RULES")]
        rules: PathBuf,
        #[arg(value_name = "LEXICON")]
        lexicon: PathBuf,
        #[arg(short, long)]
        paradigma: Option<String>,
        #[arg(short, long, default_value_t=String::from("ROOT"))]
        initial_nonterminal: String,
        #[arg(short, long)]
        unking: bool,
        #[arg(short, long)]
        smoothing: bool,
        #[arg(short, long)]
        threshold_beam: u64,
        #[arg(short, long)]
        rank_beam: u64,
        #[arg(short, long)]
        astar: PathBuf,
    },
    Binarise {},
    Debinarise {},
    Unk {},
    Smooth {},
    Outside {},
}

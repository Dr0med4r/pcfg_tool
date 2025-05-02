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

        /// how to parse: cyk or deductive
        #[arg(short, long)]
        paradigma: Option<String>,
        #[arg(short, long, default_value_t=String::from("ROOT"))]
        initial_nonterminal: String,
        /// replace unknown words
        #[arg(short, long)]
        unking: bool,
        /// replace unknown words with smoothing
        #[arg(short, long)]
        smoothing: bool,
        /// use beam search with threshhold
        #[arg(short, long)]
        threshold_beam: Option<u64>,
        /// use beam search with constant size
        #[arg(short, long)]
        rank_beam: Option<u64>,
        /// use a star search
        #[arg(short, long)]
        astar: Option<PathBuf>,
    },
    Binarise {},
    Debinarise {},
    Unk {},
    Smooth {},
    Outside {},
}

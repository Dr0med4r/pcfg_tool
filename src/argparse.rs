use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about=None, disable_help_flag=true)]
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
    Binarise {
        #[arg(short, long, default_value_t=999)]
        horizontal: u64,
        #[arg(short, long, default_value_t=1)]
        vertical: u64,
    },
    Debinarise {},
    Unk {
        #[arg(short, long)]
        threshold: u64,
    },
    Smooth {
        #[arg(short, long)]
        threshold: Option<u64>,
    },
    Outside {
        #[arg(value_name = "RULES")]
        rules: PathBuf,
        #[arg(value_name = "LEXICON")]
        lexicon: PathBuf,
        #[arg()]
        grammar: Option<String>,
        #[arg(short, long, default_value_t=String::from("ROOT"))]
        initial_nonterminal: String,
    },
}

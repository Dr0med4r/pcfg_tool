mod argparse;
mod debinarise;
mod induce;
mod parse;
mod unk;
mod binarise;
mod smoothing;
mod astar;


use argparse::{Args, Commands};
use clap::Parser;
use debinarise::debinarise;
use binarise::binarise;
use induce::induce;
use parse::parse;
use smoothing::smooth;
use unk::unk;
use astar::out;

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
            parse(
                rules,
                lexicon,
                paradigma,
                initial_nonterminal,
                unking,
                smoothing,
                threshold_beam,
                rank_beam,
                astar,
            );
        }

        Commands::Unk { threshold } => {
            unk(*threshold);
        }

        Commands::Debinarise {} => {
            debinarise();
        }

        Commands::Binarise { horizontal, vertical } => {
            binarise(*horizontal, *vertical);
        }

        Commands::Smooth { threshold } => {
            smooth(*threshold);
        }

        Commands::Outside { rules, lexicon, grammar, initial_nonterminal } => {
            out(rules, lexicon, grammar, initial_nonterminal);

        }
    }
}

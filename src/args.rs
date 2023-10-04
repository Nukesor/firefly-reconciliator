use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = "bois",
    about = "A configuration management tool for your system or user dotfiles.",
    author,
    version
)]
pub struct Arguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The path to the configuration file that should be used.
    #[clap(short, long)]
    pub config: Option<PathBuf>,
}

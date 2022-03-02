use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub const RHAI_TOML: &str = "rhai.toml";

/// Generate HTML documentation from Rhai script files.
#[derive(Parser, Debug)]
#[clap(name = "rhai-doc", about, version, author)]
pub struct Cli {
    /// Use multiple to set the level of verbosity: 1 = silent, 2 (default) = full, 3 = debug
    #[clap(long, short)]
    #[clap(parse(from_occurrences))]
    pub verbose: usize,
    /// Set the configuration file
    #[clap(long, short, value_name = "FILE", default_value = RHAI_TOML)]
    pub config: PathBuf,
    /// Generate documentation for all functions, including private ones
    #[clap(long, short)]
    pub all: bool,
    /// Set the Rhai scripts (*.rhai) directory
    #[clap(long = "dir", short, value_name = "DIR", default_value = ".")]
    pub directory: PathBuf,
    /// Set the directory where MarkDown (*.md) pages files are located
    #[clap(long, short, value_name = "DIR", default_value = "pages")]
    pub pages: PathBuf,
    /// Set the destination for the documentation output
    #[clap(long = "dest", short = 'D', value_name = "DIR", default_value = "dist")]
    pub destination: PathBuf,

    /// Sub-commands
    #[clap(subcommand)]
    pub command: Option<RhaiDocCommand>,
}

/// Sub-commands
#[derive(Subcommand, Debug)]
pub enum RhaiDocCommand {
    /// Generates a new configuration file
    New {
        /// Sets the configuration file to generate
        #[clap(long, short, value_name = "FILE", default_value = RHAI_TOML)]
        config: String,
    },
}

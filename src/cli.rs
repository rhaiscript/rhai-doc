use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

pub const RHAI_TOML: &str = "rhai.toml";

/// Generate HTML documentation from Rhai script files.
#[derive(Debug, Parser)]
#[command(name = "rhai-doc", about, version, author)]
pub struct Cli {
    /// Logging level (use multiple to set the level of verbosity)
    ///
    /// 1 = silent, 2 (default) = full, 3 = debug
    #[arg(long, short, action = ArgAction::Count)]
    pub verbose: u8,
    /// Use silent mode (overrides --verbose)
    #[arg(long, short)]
    pub quiet: bool,

    /// Generate documentation for all functions, including private ones
    #[arg(long, short)]
    pub all: bool,

    /// Set the configuration file
    #[arg(long, short, value_name = "FILE", default_value = RHAI_TOML)]
    pub config: PathBuf,
    /// Set the Rhai scripts (*.rhai) directory
    #[arg(long = "dir", short, value_name = "DIR", default_value = ".")]
    pub directory: PathBuf,
    /// Set the directory where MarkDown (*.md) pages files are located
    #[arg(long, short, value_name = "DIR", default_value = "pages")]
    pub pages: PathBuf,
    /// Set the destination for the documentation output
    #[arg(long = "dest", short = 'D', value_name = "DIR", default_value = "dist")]
    pub destination: PathBuf,

    /// Sub-commands
    #[command(subcommand)]
    pub command: Option<RhaiDocCommand>,
}

/// Sub-commands
#[derive(Subcommand, Debug)]
pub enum RhaiDocCommand {
    /// Generates a new configuration file
    New {
        /// Sets the configuration file to generate
        #[arg(long, short, value_name = "FILE", default_value = RHAI_TOML)]
        config: String,
    },
}

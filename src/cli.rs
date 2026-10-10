//! Command-line interface. Runtime behaviour is configured in `config`.

use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "cpc",
    version,
    about = "Copy code to your clipboard, with filenames and token counts",
    long_about = "Copy one or more files or directories as a single, labelled text bundle.\nAll processing is local. Files are never sent over the network."
)]
pub struct Cli {
    /// Files and directories to copy; mixed inputs are supported.
    #[arg(required = true, value_name = "PATH")]
    pub input_paths: Vec<PathBuf>,

    /// Use this TOML configuration instead of the default configuration.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Do not include the ==== filename ==== headers.
    #[arg(long)]
    pub no_headers: bool,

    /// Skip tokenisation altogether.
    #[arg(long, conflicts_with = "tokenizer")]
    pub no_tokens: bool,

    /// Token encoding: o200k_base, cl100k_base or off.
    #[arg(long, value_parser = ["o200k_base", "cl100k_base", "off"])]
    pub tokenizer: Option<String>,

    /// Write the bundle to stdout instead of the system clipboard.
    #[arg(long)]
    pub stdout: bool,

    /// Include hidden entries when traversing directories (not built-in exclusions).
    #[arg(long)]
    pub hidden: bool,

    /// Disable .gitignore/.ignore filtering (not built-in exclusions).
    #[arg(long)]
    pub no_ignore: bool,
}

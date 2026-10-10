//! cpc: deterministic file selection, formatting, measurement and clipboard output.

pub mod bundle;
pub mod cli;
pub mod clipboard;
pub mod config;
pub mod report;
pub mod selection;
pub mod tokens;

use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;

pub fn run() -> Result<()> {
    execute(cli::Cli::parse())
}

/// Run the application. The clipboard is only touched after the complete
/// selection, payload and optional token measurement succeed.
pub fn execute(args: cli::Cli) -> Result<()> {
    let settings = config::resolve_settings(config::load_config(args.config.as_deref())?, &args)?;
    let selection = selection::collect_files(&args.input_paths, &settings)?;
    let bundle = bundle::build_bundle(&selection, &settings)?;
    let summary = report::measure_bundle(&bundle, settings.report.tokenizer)?;
    if args.stdout {
        let stdout = std::io::stdout();
        let mut writer = stdout.lock();
        writer
            .write_all(bundle.text.as_bytes())
            .context("could not write bundle to stdout")?;
        writer.flush().context("could not flush stdout")?;
    } else {
        clipboard::write_clipboard(&bundle.text)?;
    }
    eprintln!("{}", report::format_summary(&summary, !args.stdout));
    Ok(())
}

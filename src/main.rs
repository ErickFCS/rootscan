//! rootscan — tracks the filesystem structure (paths only, never file
//! contents) of a root folder and its subdirectories, honoring
//! gitignore-like rules.
//!
//! Lifecycle: parse CLI input -> build config -> scan -> build tree ->
//! serialize -> write output -> report summary. Any failure aborts the
//! lifecycle and is reported with its full error chain.

mod adapters;
mod app;
mod domain;
mod error;
mod io;
mod ports;

use std::process::ExitCode;

use clap::Parser as _;
use io::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match app::run(cli) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            print_error_chain(&err);
            ExitCode::FAILURE
        }
    }
}

/// Prints an error followed by its whole `cause by` chain, so failures are
/// always diagnosed at the layer that produced them.
fn print_error_chain(err: &error::Error) {
    eprintln!("error: {err}");
    let mut source = std::error::Error::source(err);
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}

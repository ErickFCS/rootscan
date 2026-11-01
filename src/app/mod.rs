//! Coordination layer: owns the application lifecycle. Wires input to core
//! processing to output, without knowing how any of them is implemented
//! (it only uses ports).

pub mod config;

use std::fmt;
use std::time::Instant;

use crate::adapters::serializer;
use crate::adapters::walker::FsWalker;
use crate::domain::tree::{Tree, TreeOptions};
use crate::error::Result;
use crate::io::cli::Cli;
use crate::io::writer;
use crate::ports::scanner::Scanner;

/// Result of a complete, successful run.
#[derive(Debug)]
pub struct Summary {
    pub files: u64,
    pub directories: u64,
    pub skipped: u64,
    pub output: std::path::PathBuf,
    pub elapsed: std::time::Duration,
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tracked {} files and {} directories ({} skipped) -> {} in {}",
            self.files,
            self.directories,
            self.skipped,
            self.output.display(),
            humantime::format_duration(self.elapsed)
        )
    }
}

/// Runs the full lifecycle: configure -> scan -> build tree -> serialize ->
/// write -> report.
pub fn run(cli: Cli) -> Result<Summary> {
    let start = Instant::now();
    let config = config::build(cli)?;

    let (entries, stats) = FsWalker.scan(&config.scan)?;

    let tree = Tree::build(
        config.scan.root_name(),
        &entries,
        TreeOptions {
            sorted: config.sorted,
            dirs_only: config.dirs_only,
        },
    );

    let bytes = serializer::select(config.format).serialize(&tree)?;
    writer::write(&config.output, &bytes)?;

    Ok(Summary {
        files: stats.files,
        directories: stats.directories,
        skipped: stats.skipped,
        output: config.output,
        elapsed: start.elapsed(),
    })
}

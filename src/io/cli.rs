//! Input adapter: CLI definition and parsing.

use std::path::PathBuf;

use clap::Parser;

use crate::ports::serializer::OutputFormat;

#[derive(Debug, Parser)]
#[command(
    name = "rootscan",
    version,
    about = "Track every path under a root folder in a snapshot file, honoring gitignore-like rules.",
    after_help = "Examples:\n  rootscan ~/projects/app\n  rootscan . -f json -o /tmp/snapshot.json\n  rootscan . -e team.ignore --sorted -L 3"
)]
pub struct Cli {
    /// Root folder to scan.
    pub root: PathBuf,

    /// Output file path. Defaults to `<ROOT>/.rootscan.<ext>` based on the
    /// selected format. The output file is automatically excluded from the
    /// scan.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "yaml")]
    pub format: OutputFormat,

    /// Extra gitignore-like rule file, interpreted relative to every
    /// scanned directory (rules follow `.gitignore` semantics). Repeatable.
    #[arg(short = 'e', long = "exclude-file", value_name = "FILE")]
    pub exclude_files: Vec<PathBuf>,

    /// Do not read `.gitignore` and related ignore files.
    #[arg(long)]
    pub no_gitignore: bool,

    /// Include hidden entries (dotfiles and dot-directories).
    #[arg(short = 'H', long)]
    pub hidden: bool,

    /// Maximum recursion depth below the root.
    #[arg(short = 'L', long, value_name = "DEPTH")]
    pub max_depth: Option<usize>,

    /// Sort every directory's children (directories first).
    #[arg(long)]
    pub sorted: bool,

    /// List only directories, omitting file entries.
    #[arg(short = 'C', long)]
    pub dirs_only: bool,
}

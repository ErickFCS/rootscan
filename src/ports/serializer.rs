//! Serializer port: turns a domain tree into output bytes in a concrete
//! representation.

use crate::domain::tree::Tree;
use crate::error::Result;

/// Output format selector. Also drives the CLI value parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Indented, tree-shaped YAML (default).
    Yaml,
    /// Same tree shape as YAML, JSON-encoded.
    Json,
    /// Flat list of relative paths, one per line, directories with a
    /// trailing `/`.
    Text,
}

impl OutputFormat {
    /// Default output file name (relative to the scanned root).
    pub fn default_output_name(self) -> &'static str {
        match self {
            OutputFormat::Yaml => ".rootscan.yaml",
            OutputFormat::Json => ".rootscan.json",
            OutputFormat::Text => ".rootscan.txt",
        }
    }

    /// Lowercase name, used in messages.
    #[allow(dead_code)] // kept for consumers of the port API
    pub fn as_str(self) -> &'static str {
        match self {
            OutputFormat::Yaml => "yaml",
            OutputFormat::Json => "json",
            OutputFormat::Text => "text",
        }
    }
}

pub trait Serializer {
    fn serialize(&self, tree: &Tree) -> Result<Vec<u8>>;
}

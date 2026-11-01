//! Ports: the contracts the coordination layer depends on, decoupling it
//! from concrete adapters.
//!
//! Dependency direction is always inward: `cli -> app -> ports <- adapters`.
//! `domain` types are shared freely, as it has no dependencies itself.

pub mod scanner;
pub mod serializer;

/// Configuration for a filesystem scan.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Canonical root directory to scan.
    pub root: std::path::PathBuf,
    /// Include hidden entries (dotfiles/dot-directories).
    pub include_hidden: bool,
    /// Respect `.gitignore` and similar files found during the walk.
    pub respect_gitignore: bool,
    /// Extra gitignore-like rule files; interpreted relative to each
    /// directory visited (same semantics as `.gitignore`).
    pub exclude_files: Vec<std::path::PathBuf>,
    /// Paths that must be pruned from the walk entirely (e.g. the output
    /// file itself), compared exactly.
    pub exclude_paths: Vec<std::path::PathBuf>,
    /// Maximum recursion depth; `None` means unlimited.
    pub max_depth: Option<usize>,
}

impl ScanOptions {
    pub fn root_name(&self) -> String {
        self.root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.root.to_string_lossy().into_owned())
    }
}

/// Counters describing what was scanned.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScanStats {
    pub files: u64,
    pub directories: u64,
    /// Entries that reported a filesystem error (e.g. unreadable).
    pub skipped: u64,
}

impl ScanStats {
    /// Total tracked entries (files + directories).
    #[allow(dead_code)] // kept for consumers of the port API
    pub fn entries(&self) -> u64 {
        self.files + self.directories
    }
}

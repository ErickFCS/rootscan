//! Minimal domain model: what a filesystem entry *is*. Metadata such as
//! size or permissions is intentionally not tracked.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Path relative to the scan root.
    pub relative: PathBuf,
    pub kind: EntryKind,
}

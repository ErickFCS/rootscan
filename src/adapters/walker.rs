//! Filesystem adapter implementing the scanner port on top of the `ignore`
//! crate (the walker engine behind ripgrep).
//!
//! Features inherited from `ignore`:
//! - parallel directory traversal across all cores
//! - correct `.gitignore` semantics: nested rules, `!` negation, anchoring,
//!   `**`, `.git/info/exclude`, global gitignore and `.ignore` files
//! - hidden-file filtering
//! - symlinked directories are listed but never followed (cycle-safe)

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ignore::{DirEntry, WalkBuilder};

use crate::domain::model::{Entry, EntryKind};
use crate::error::{Error, Result};
use crate::ports::scanner::Scanner;
use crate::ports::{ScanOptions, ScanStats};

#[derive(Debug, Default, Clone, Copy)]
pub struct FsWalker;

impl Scanner for FsWalker {
    fn scan(&self, options: &ScanOptions) -> Result<(Vec<Entry>, ScanStats)> {
        let mut builder = WalkBuilder::new(&options.root);

        builder
            .hidden(!options.include_hidden)
            .standard_filters(options.respect_gitignore)
            .git_global(options.respect_gitignore)
            .follow_links(false);

        // Honor gitignore-like rules even outside a git repository.
        builder.require_git(false);

        if let Some(max_depth) = options.max_depth {
            builder.max_depth(Some(max_depth));
        }

        for rule_file in &options.exclude_files {
            builder.add_custom_ignore_filename(rule_file);
        }

        // Prune excluded paths (e.g. the output file itself) from the walk.
        for excluded in &options.exclude_paths {
            let excluded = excluded.clone();
            builder.filter_entry(move |entry| entry.path() != excluded.as_path());
        }

        let entries = Arc::new(Mutex::new(Vec::new()));
        let files = Arc::new(AtomicU64::new(0));
        let directories = Arc::new(AtomicU64::new(0));
        let skipped = Arc::new(AtomicU64::new(0));
        let first_error = Arc::new(Mutex::new(None::<Error>));
        let root = options.root.clone();

        builder.build_parallel().run(|| {
            let root = root.clone();
            let entries = Arc::clone(&entries);
            let files = Arc::clone(&files);
            let directories = Arc::clone(&directories);
            let skipped = Arc::clone(&skipped);
            let first_error = Arc::clone(&first_error);
            Box::new(move |result: std::result::Result<DirEntry, ignore::Error>| {
                Visitor {
                    root: root.clone(),
                    entries: &entries,
                    files: &files,
                    directories: &directories,
                    skipped: &skipped,
                    first_error: &first_error,
                }
                .visit(result)
            })
        });

        if let Some(err) = first_error.lock().expect("visitor error mutex poisoned").take() {
            return Err(err);
        }

        let collected = Mutex::into_inner(
            Arc::try_unwrap(entries).expect("all walker threads finished"),
        )
        .expect("entry mutex poisoned");

        let stats = ScanStats {
            files: files.load(Ordering::Relaxed),
            directories: directories.load(Ordering::Relaxed),
            skipped: skipped.load(Ordering::Relaxed),
        };
        Ok((collected, stats))
    }
}

/// Per-entry visitor logic for the parallel walk.
struct Visitor<'a> {
    root: PathBuf,
    entries: &'a Mutex<Vec<Entry>>,
    files: &'a AtomicU64,
    directories: &'a AtomicU64,
    skipped: &'a AtomicU64,
    first_error: &'a Mutex<Option<Error>>,
}

impl Visitor<'_> {
    fn visit(&mut self, result: std::result::Result<DirEntry, ignore::Error>) -> ignore::WalkState {
        use ignore::WalkState;

        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                self.skipped.fetch_add(1, Ordering::Relaxed);
                record_once(self.first_error, Error::Walk(err));
                return WalkState::Continue; // collect everything possible
            }
        };

        // Depth 0 is the root directory itself, not a tracked entry.
        if entry.depth() == 0 {
            return WalkState::Continue;
        }

        let Ok(relative) = entry.path().strip_prefix(&self.root) else {
            return WalkState::Continue;
        };

        let kind = if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            self.directories.fetch_add(1, Ordering::Relaxed);
            EntryKind::Directory
        } else {
            self.files.fetch_add(1, Ordering::Relaxed);
            EntryKind::File
        };

        self.entries
            .lock()
            .expect("entry mutex poisoned")
            .push(Entry { relative: relative.to_path_buf(), kind });

        WalkState::Continue
    }
}

/// Keeps the first error encountered; later ones would be duplicates of the
/// same underlying failure propagated once per worker thread.
fn record_once(slot: &Mutex<Option<Error>>, error: Error) {
    let mut guard = slot.lock().expect("visitor error mutex poisoned");
    if guard.is_none() {
        *guard = Some(error);
    }
}

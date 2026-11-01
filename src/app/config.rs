//! Translates CLI input into the configuration the rest of the application
//! consumes, validating as early as possible (fail fast, at the edge).

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::io::cli::Cli;
use crate::ports::serializer::OutputFormat;
use crate::ports::ScanOptions;

#[derive(Debug)]
pub struct Config {
    pub format: OutputFormat,
    pub output: PathBuf,
    pub sorted: bool,
    pub dirs_only: bool,
    pub scan: ScanOptions,
}

/// Builds and validates the application configuration from raw CLI input.
pub fn build(cli: Cli) -> Result<Config> {
    let Cli {
        root,
        output,
        format,
        exclude_files,
        no_gitignore,
        hidden,
        max_depth,
        sorted,
        dirs_only,
    } = cli;

    let root = canonicalize_dir(&root)?;

    let output = match output {
        Some(path) => absolute_output(path),
        None => root.join(format.default_output_name()),
    };

    let mut scan = ScanOptions {
        root: root.clone(),
        include_hidden: hidden,
        respect_gitignore: !no_gitignore,
        exclude_files,
        exclude_paths: vec![],
        max_depth,
    };

    // Exclude the output file from the scan, so re-running on the same root
    // never lets a stale snapshot appear in the fresh one.
    match std::fs::canonicalize(&output) {
        Ok(existing) => scan.exclude_paths.push(existing),
        Err(_) => scan.exclude_paths.push(output.clone()),
    }

    Ok(Config {
        format,
        output,
        sorted,
        dirs_only,
        scan,
    })
}

/// Canonicalizes `path` and verifies it is an existing directory.
fn canonicalize_dir(path: &Path) -> Result<PathBuf> {
    let canonical = std::fs::canonicalize(path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Error::RootNotFound(path.to_path_buf()),
        _ => Error::RootInspect(path.to_path_buf(), err),
    })?;

    let is_dir = std::fs::metadata(&canonical)
        .map(|meta| meta.is_dir())
        .map_err(|err| Error::RootInspect(canonical.clone(), err))?;

    if !is_dir {
        return Err(Error::RootNotDirectory(canonical));
    }
    Ok(canonical)
}

/// Resolves a user-supplied output path to an absolute path, anchoring
/// relative paths to the current directory and following symlinks of any
/// existing parent directories.
fn absolute_output(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    let fallback = std::path::absolute(&path).unwrap_or(path);
    match (fallback.parent(), fallback.file_name()) {
        (Some(parent), Some(file)) => match std::fs::canonicalize(parent) {
            Ok(parent) => parent.join(file),
            Err(_) => fallback,
        },
        _ => fallback,
    }
}

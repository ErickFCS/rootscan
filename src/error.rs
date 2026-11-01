//! Central error type for the whole application.
//!
//! Every layer maps its own failure modes onto this enum; the coordination
//! layer and `main` are the only places that consume it.

use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("root path `{0}` does not exist")]
    RootNotFound(PathBuf),

    #[error("root path `{0}` is not a directory")]
    RootNotDirectory(PathBuf),

    #[error("failed to inspect root path `{0}`")]
    RootInspect(PathBuf, #[source] std::io::Error),

    #[error("failed to serialize output as `{0}`")]
    Serialize(&'static str, #[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to write output to `{0}`")]
    OutputWrite(PathBuf, #[source] std::io::Error),

    #[error("filesystem walk failed")]
    Walk(#[source] ignore::Error),
}

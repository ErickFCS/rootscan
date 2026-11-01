//! Output adapter: writes the serialized snapshot to disk.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::{Error, Result};

/// Writes `bytes` to `path`, creating missing parent directories. Any
/// pre-existing file is replaced atomically enough for this tool's purpose
/// (create + truncate + write).
pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| Error::OutputWrite(path.to_path_buf(), err))?;
    }

    let mut file = File::create(path).map_err(|err| Error::OutputWrite(path.to_path_buf(), err))?;
    file.write_all(bytes)
        .and_then(|_| file.flush())
        .map_err(|err| Error::OutputWrite(path.to_path_buf(), err))?;
    Ok(())
}

//! Scanner port: anything that can produce the filesystem entries of a scan,
//! plus statistics about the run.

use crate::domain::model::Entry;
use crate::error::Result;
use crate::ports::{ScanOptions, ScanStats};

pub trait Scanner {
    /// Scans the filesystem and returns all entries plus run statistics.
    ///
    /// Implementations may walk in parallel; the returned entry order is
    /// therefore unspecified unless the adapter documents otherwise.
    fn scan(&self, options: &ScanOptions) -> Result<(Vec<Entry>, ScanStats)>;
}

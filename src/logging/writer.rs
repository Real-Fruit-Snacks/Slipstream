use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use tempfile::NamedTempFile;
use fs2::FileExt;

/// Write `content` atomically to `path` using a tempfile + rename.
pub fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.flush()?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

/// Append `content` to `path` with an exclusive flock, then flush.
pub fn locked_write(path: &Path, content: &[u8]) -> io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.lock_exclusive()?;
    let mut file = file;
    file.write_all(content)?;
    file.flush()?;
    fs2::FileExt::unlock(&file)?;
    Ok(())
}

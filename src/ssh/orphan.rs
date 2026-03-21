use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct OrphanSocket {
    pub path: PathBuf,
    pub pid: Option<u32>,
}

pub struct OrphanDetector;

impl OrphanDetector {
    /// Scan directory for socket files matching "slipstream_*.sock" pattern.
    /// Extract PID from filename and check if /proc/PID exists.
    /// Return only orphans where the PID doesn't exist.
    pub fn scan(sessions_dir: &Path) -> Result<Vec<OrphanSocket>, io::Error> {
        let mut orphans = Vec::new();

        // If directory doesn't exist, return empty list
        if !sessions_dir.exists() {
            return Ok(orphans);
        }

        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Check if filename matches "ssh-*-*.sock" pattern
            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.starts_with("ssh-") && filename_str.ends_with(".sock") {
                        // Extract PID from filename: "ssh-host-12345.sock"
                        // PID is the last segment after the final '-'
                        let without_ext = &filename_str[..filename_str.len() - 5]; // Strip ".sock"
                        let pid_str = without_ext.rsplit('-').next().unwrap_or("");
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            // Check if /proc/PID exists
                            let proc_path = PathBuf::from(format!("/proc/{}", pid));
                            if !proc_path.exists() {
                                orphans.push(OrphanSocket {
                                    path,
                                    pid: Some(pid),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(orphans)
    }

    /// Remove the socket file associated with an orphan.
    pub fn cleanup(orphan: &OrphanSocket) -> Result<(), io::Error> {
        if orphan.path.exists() {
            fs::remove_file(&orphan.path)?;
        }
        Ok(())
    }

    /// Print information about an orphan socket to stderr.
    pub fn prompt_user(orphan: &OrphanSocket) {
        match &orphan.pid {
            Some(pid) => {
                eprintln!(
                    "Found orphaned socket: {} (PID {} no longer exists)",
                    orphan.path.display(),
                    pid
                );
            }
            None => {
                eprintln!(
                    "Found orphaned socket: {} (unable to determine PID)",
                    orphan.path.display()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_no_orphans_in_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let orphans = OrphanDetector::scan(tmp.path()).unwrap();
        assert!(orphans.is_empty());
    }

    #[test]
    fn test_detect_stale_socket_file() {
        let tmp = TempDir::new().unwrap();
        let sock_path = tmp.path().join("ssh-testhost-99999999.sock");
        fs::write(&sock_path, "").unwrap();
        let orphans = OrphanDetector::scan(tmp.path()).unwrap();
        assert_eq!(orphans.len(), 1);
    }

    #[test]
    fn test_cleanup_removes_socket() {
        let tmp = TempDir::new().unwrap();
        let sock_path = tmp.path().join("ssh-testhost-99999999.sock");
        fs::write(&sock_path, "").unwrap();
        let orphans = OrphanDetector::scan(tmp.path()).unwrap();
        OrphanDetector::cleanup(&orphans[0]).unwrap();
        assert!(!sock_path.exists());
    }
}

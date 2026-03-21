use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshDiscoveryError {
    #[error("configured SSH path does not exist: {0}")]
    ConfiguredPathMissing(String),
    #[error("ssh binary not found in PATH or fallback locations")]
    NotFound,
}

pub struct SshDiscovery;

impl SshDiscovery {
    pub fn find_ssh(
        config_path: Option<&str>,
        own_exe: Option<&str>,
    ) -> Result<PathBuf, SshDiscoveryError> {
        // Strategy 1: explicit config path
        if let Some(path) = config_path {
            if !path.is_empty() {
                let p = PathBuf::from(path);
                if p.exists() {
                    return Ok(p);
                } else {
                    return Err(SshDiscoveryError::ConfiguredPathMissing(
                        path.to_string(),
                    ));
                }
            }
        }

        // Resolve own_exe to a canonical path for comparison
        let own_canonical = own_exe.and_then(|e| std::fs::canonicalize(e).ok());

        // Strategy 2: search $PATH for `ssh`
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                let candidate = PathBuf::from(dir).join("ssh");
                if candidate.exists() {
                    // Check if this resolves to our own exe (prevent recursion)
                    if let Some(ref own) = own_canonical {
                        if let Ok(canonical) = std::fs::canonicalize(&candidate) {
                            if &canonical == own {
                                continue;
                            }
                        }
                    }
                    return Ok(candidate);
                }
            }
        }

        // Strategy 3: fallback to /usr/bin/ssh
        let fallback = PathBuf::from("/usr/bin/ssh");
        if fallback.exists() {
            return Ok(fallback);
        }

        Err(SshDiscoveryError::NotFound)
    }
}

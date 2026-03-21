use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub identity: Identity,
    pub addresses: Vec<Address>,
    pub saved_tunnels: Vec<SavedTunnel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub fingerprint: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub ip: String,
    pub port: u16,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTunnel {
    #[serde(rename = "type")]
    pub tunnel_type: String,
    pub port: Option<u16>,
    pub source: Option<String>,
    pub dest_host: Option<String>,
    pub dest_port: Option<u16>,
    pub auto_restore: bool,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("TOML deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("target not found: {0}")]
    NotFound(String),
}

pub struct TargetStorage {
    pub base_dir: PathBuf,
}

impl TargetStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Replace `:`, `/`, `\`, `+` so fingerprints are filesystem-safe directory names.
    pub fn fingerprint_to_dirname(fingerprint: &str) -> String {
        fingerprint.replace([':', '/', '\\', '+'], "-")
    }

    pub fn target_dir(&self, fingerprint: &str) -> PathBuf {
        self.base_dir.join(Self::fingerprint_to_dirname(fingerprint))
    }

    /// Create the target directory (and a `logs` subdir) with 0700 permissions on Unix.
    pub fn ensure_target_dir(&self, fingerprint: &str) -> Result<PathBuf, StorageError> {
        let dir = self.target_dir(fingerprint);
        fs::create_dir_all(&dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        }

        let logs_dir = dir.join("logs");
        fs::create_dir_all(&logs_dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&logs_dir, fs::Permissions::from_mode(0o700))?;
        }

        Ok(dir)
    }

    /// Atomically write a TargetInfo to disk via a temp file.
    pub fn save_target(&self, target: &TargetInfo) -> Result<(), StorageError> {
        let dir = self.ensure_target_dir(&target.identity.fingerprint)?;
        let target_file = dir.join("target.toml");

        let contents = toml::to_string_pretty(target)?;

        // Write atomically: write to temp file in the same directory, then rename.
        let mut tmp = NamedTempFile::new_in(&dir)?;
        std::io::Write::write_all(&mut tmp, contents.as_bytes())?;
        tmp.persist(&target_file)
            .map_err(|e| StorageError::Io(e.error))?;

        Ok(())
    }

    pub fn load_target(&self, fingerprint: &str) -> Result<TargetInfo, StorageError> {
        let target_file = self.target_dir(fingerprint).join("target.toml");
        self.load_target_from_path(&target_file)
    }

    pub fn load_target_from_path(&self, path: &Path) -> Result<TargetInfo, StorageError> {
        if !path.exists() {
            return Err(StorageError::NotFound(path.display().to_string()));
        }
        let contents = fs::read_to_string(path)?;
        let target: TargetInfo = toml::from_str(&contents)?;
        Ok(target)
    }

    /// Create a timestamped session directory inside the target's logs dir.
    pub fn create_session_dir(&self, fingerprint: &str) -> Result<PathBuf, StorageError> {
        let logs_dir = self.target_dir(fingerprint).join("logs");
        fs::create_dir_all(&logs_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();
        let session_dir = logs_dir.join(timestamp);
        fs::create_dir_all(&session_dir)?;

        Ok(session_dir)
    }

    /// Rename the target directory to dir_archived_TIMESTAMP.
    pub fn archive_target(&self, fingerprint: &str) -> Result<(), StorageError> {
        let old_dir = self.target_dir(fingerprint);
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S").to_string();
        let new_name = format!(
            "{}_archived_{}",
            Self::fingerprint_to_dirname(fingerprint),
            timestamp
        );
        let new_dir = self.base_dir.join(new_name);
        fs::rename(&old_dir, &new_dir)?;
        Ok(())
    }

    /// Rename old_fp directory to new_fp directory and update fingerprint in target.toml.
    pub fn rename_target(&self, old_fp: &str, new_fp: &str) -> Result<(), StorageError> {
        let old_dir = self.target_dir(old_fp);
        let new_dir = self.target_dir(new_fp);
        fs::rename(&old_dir, &new_dir)?;

        let target_file = new_dir.join("target.toml");
        if target_file.exists() {
            let contents = fs::read_to_string(&target_file)?;
            let mut target: TargetInfo = toml::from_str(&contents)?;
            target.identity.fingerprint = new_fp.to_string();
            let new_contents = toml::to_string_pretty(&target)?;
            use tempfile::NamedTempFile;
            let mut tmp = NamedTempFile::new_in(&new_dir)?;
            std::io::Write::write_all(&mut tmp, new_contents.as_bytes())?;
            tmp.persist(&target_file)
                .map_err(|e| StorageError::Io(e.error))?;
        }

        Ok(())
    }

    pub fn ensure_dir_secure(path: &std::path::Path) -> Result<(), StorageError> {
        std::fs::create_dir_all(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(())
    }

    /// Return a list of all known targets by reading target.toml files in base_dir subdirs.
    pub fn list_targets(&self) -> Result<Vec<TargetInfo>, StorageError> {
        if !self.base_dir.exists() {
            return Ok(vec![]);
        }

        let mut targets = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let target_file = path.join("target.toml");
                if target_file.exists() {
                    if let Ok(t) = self.load_target_from_path(&target_file) {
                        targets.push(t);
                    }
                }
            }
        }
        Ok(targets)
    }
}

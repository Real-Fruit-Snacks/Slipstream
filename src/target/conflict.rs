use crate::target::storage::{StorageError, TargetInfo, TargetStorage};
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictAction {
    Archive,
    Keep,
    Ignore,
}

pub struct ConflictPrompt;

impl ConflictPrompt {
    pub fn prompt(old_target: &TargetInfo, new_fingerprint: &str) -> ConflictAction {
        let stderr = io::stderr();
        let mut err = stderr.lock();

        writeln!(err, "Fingerprint conflict detected!").ok();
        writeln!(
            err,
            "  Existing target fingerprint: {}",
            old_target.identity.fingerprint
        )
        .ok();
        writeln!(err, "  New fingerprint:             {}", new_fingerprint).ok();
        writeln!(err, "  Hostname: {}", old_target.identity.hostname).ok();
        writeln!(err).ok();
        writeln!(err, "Choose action:").ok();
        writeln!(err, "  [A] Archive  - rename old target dir with timestamp suffix").ok();
        writeln!(err, "  [K] Keep     - keep old target, use new fingerprint dir (default)").ok();
        writeln!(err, "  [I] Ignore   - rename old dir to new fingerprint (merge)").ok();
        write!(err, "Selection [A/K/I] (default: K): ").ok();
        err.flush().ok();

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).ok();

        match line.trim().to_uppercase().as_str() {
            "A" => ConflictAction::Archive,
            "I" => ConflictAction::Ignore,
            _ => ConflictAction::Keep,
        }
    }

    pub fn execute_action(
        action: ConflictAction,
        storage: &TargetStorage,
        old_target: &TargetInfo,
        new_fingerprint: &str,
    ) -> Result<(), StorageError> {
        let old_fp = &old_target.identity.fingerprint;
        match action {
            ConflictAction::Archive => {
                storage.archive_target(old_fp)?;
                storage.ensure_target_dir(new_fingerprint)?;
            }
            ConflictAction::Keep => {
                storage.ensure_target_dir(new_fingerprint)?;
            }
            ConflictAction::Ignore => {
                storage.rename_target(old_fp, new_fingerprint)?;
            }
        }
        Ok(())
    }
}

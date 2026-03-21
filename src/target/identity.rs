use crate::target::storage::{StorageError, TargetInfo, TargetStorage};

pub enum Resolution {
    NewTarget,
    ExistingTarget { target: TargetInfo },
    ExistingTargetNewIp { target: TargetInfo },
    FingerprintChanged { old_target: TargetInfo },
}

pub struct TargetResolver<'a> {
    storage: &'a TargetStorage,
}

impl<'a> TargetResolver<'a> {
    pub fn new(storage: &'a TargetStorage) -> Self {
        Self { storage }
    }

    /// Resolve the identity of a target given its fingerprint, IP, and port.
    ///
    /// Logic:
    /// 1. Check all stored targets for a matching fingerprint.
    ///    - Found: check if the (ip, port) is already in the addresses list.
    ///      - Yes → ExistingTarget
    ///      - No  → ExistingTargetNewIp
    /// 2. No fingerprint match: check all stored targets for an (ip, port) match.
    ///    - Found: the fingerprint changed → FingerprintChanged
    /// 3. Neither → NewTarget
    pub fn resolve(
        &self,
        fingerprint: &str,
        ip: &str,
        port: u16,
    ) -> Result<Resolution, StorageError> {
        let targets = self.storage.list_targets()?;

        // Step 1: look for a matching fingerprint
        for target in &targets {
            if target.identity.fingerprint == fingerprint {
                let ip_match = target
                    .addresses
                    .iter()
                    .any(|a| a.ip == ip && a.port == port);
                if ip_match {
                    return Ok(Resolution::ExistingTarget {
                        target: target.clone(),
                    });
                } else {
                    return Ok(Resolution::ExistingTargetNewIp {
                        target: target.clone(),
                    });
                }
            }
        }

        // Step 2: look for a matching (ip, port) with a different fingerprint
        for target in &targets {
            let ip_match = target
                .addresses
                .iter()
                .any(|a| a.ip == ip && a.port == port);
            if ip_match {
                return Ok(Resolution::FingerprintChanged {
                    old_target: target.clone(),
                });
            }
        }

        // Step 3: completely new target
        Ok(Resolution::NewTarget)
    }
}

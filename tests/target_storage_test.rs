use chrono::Utc;
use slipstream::target::storage::{Address, Identity, SavedTunnel, TargetInfo, TargetStorage};
use tempfile::TempDir;

fn make_storage() -> (TempDir, TargetStorage) {
    let tmp = TempDir::new().unwrap();
    let storage = TargetStorage::new(tmp.path().to_path_buf());
    (tmp, storage)
}

fn make_target(fingerprint: &str, ip: &str) -> TargetInfo {
    TargetInfo {
        identity: Identity {
            fingerprint: fingerprint.to_string(),
            hostname: "test-host".to_string(),
        },
        addresses: vec![Address {
            ip: ip.to_string(),
            port: 22,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }],
        saved_tunnels: vec![],
    }
}

#[test]
fn test_create_target_directory() {
    let (_tmp, storage) = make_storage();
    let fingerprint = "SHA256:abc123";

    let dir = storage.ensure_target_dir(fingerprint).unwrap();
    assert!(dir.exists(), "target dir should exist");
    assert!(dir.join("logs").exists(), "logs subdir should exist");
}

#[test]
fn test_save_and_load_target_toml() {
    let (_tmp, storage) = make_storage();
    let target = make_target("SHA256:deadbeef", "10.0.0.1");

    storage.save_target(&target).unwrap();

    let loaded = storage.load_target("SHA256:deadbeef").unwrap();
    assert_eq!(loaded.identity.fingerprint, "SHA256:deadbeef");
    assert_eq!(loaded.identity.hostname, "test-host");
    assert_eq!(loaded.addresses.len(), 1);
    assert_eq!(loaded.addresses[0].ip, "10.0.0.1");
    assert_eq!(loaded.addresses[0].port, 22);
}

#[test]
fn test_create_session_log_dir() {
    let (_tmp, storage) = make_storage();
    let fingerprint = "SHA256:session_test";

    // Ensure the target dir exists first
    storage.ensure_target_dir(fingerprint).unwrap();

    let session_dir = storage.create_session_dir(fingerprint).unwrap();
    assert!(session_dir.exists(), "session dir should be created");

    // The session dir should be inside logs/
    let logs_dir = storage.target_dir(fingerprint).join("logs");
    assert!(
        session_dir.starts_with(&logs_dir),
        "session dir should be under logs/"
    );
}

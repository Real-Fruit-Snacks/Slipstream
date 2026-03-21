use chrono::Utc;
use slipstream::target::conflict::{ConflictAction, ConflictPrompt};
use slipstream::target::storage::{Address, Identity, TargetInfo, TargetStorage};
use tempfile::TempDir;

fn make_storage(tmp: &TempDir) -> TargetStorage {
    TargetStorage::new(tmp.path().to_path_buf())
}

fn make_target(fingerprint: &str) -> TargetInfo {
    TargetInfo {
        identity: Identity {
            fingerprint: fingerprint.to_string(),
            hostname: "test-host".to_string(),
        },
        addresses: vec![Address {
            ip: "192.168.1.1".to_string(),
            port: 22,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }],
        saved_tunnels: vec![],
    }
}

#[test]
fn test_archive_action() {
    let tmp = TempDir::new().unwrap();
    let storage = make_storage(&tmp);

    let old_fp = "SHA256:oldfingerprint";
    let new_fp = "SHA256:newfingerprint";

    let target = make_target(old_fp);
    storage.save_target(&target).unwrap();

    // Verify old dir exists
    let old_dir = storage.target_dir(old_fp);
    assert!(old_dir.exists(), "old target dir should exist before archive");

    ConflictPrompt::execute_action(ConflictAction::Archive, &storage, &target, new_fp).unwrap();

    // Old dir should be gone, new dir should exist
    assert!(!old_dir.exists(), "old target dir should be gone after archive");
    assert!(
        storage.target_dir(new_fp).exists(),
        "new target dir should exist after archive"
    );

    // An archived dir should exist somewhere in base_dir
    let archived = std::fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .contains("_archived_")
        });
    assert!(archived, "an archived directory should be present");
}

#[test]
fn test_keep_action() {
    let tmp = TempDir::new().unwrap();
    let storage = make_storage(&tmp);

    let old_fp = "SHA256:keepold";
    let new_fp = "SHA256:keepnew";

    let target = make_target(old_fp);
    storage.save_target(&target).unwrap();

    let old_dir = storage.target_dir(old_fp);
    assert!(old_dir.exists());

    ConflictPrompt::execute_action(ConflictAction::Keep, &storage, &target, new_fp).unwrap();

    // Old dir must still exist
    assert!(old_dir.exists(), "old target dir should still exist after Keep");
    // New dir should also exist
    assert!(
        storage.target_dir(new_fp).exists(),
        "new target dir should be created for Keep"
    );
}

#[test]
fn test_ignore_action() {
    let tmp = TempDir::new().unwrap();
    let storage = make_storage(&tmp);

    let old_fp = "SHA256:ignoreme";
    let new_fp = "SHA256:ignorenew";

    let target = make_target(old_fp);
    storage.save_target(&target).unwrap();

    let old_dir = storage.target_dir(old_fp);
    assert!(old_dir.exists());

    ConflictPrompt::execute_action(ConflictAction::Ignore, &storage, &target, new_fp).unwrap();

    // Old dir should be gone, data moved to new_fp dir
    assert!(!old_dir.exists(), "old target dir should be gone after Ignore");
    let new_dir = storage.target_dir(new_fp);
    assert!(new_dir.exists(), "new target dir should exist after Ignore (data moved)");

    // target.toml fingerprint should be updated
    let loaded = storage.load_target(new_fp).unwrap();
    assert_eq!(
        loaded.identity.fingerprint, new_fp,
        "fingerprint in target.toml should be updated to new_fp"
    );
}

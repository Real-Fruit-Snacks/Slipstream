use chrono::Utc;
use slipstream::target::identity::{Resolution, TargetResolver};
use slipstream::target::storage::{Address, Identity, TargetInfo, TargetStorage};
use tempfile::TempDir;

fn make_storage() -> (TempDir, TargetStorage) {
    let tmp = TempDir::new().unwrap();
    let storage = TargetStorage::new(tmp.path().to_path_buf());
    (tmp, storage)
}

fn make_target(fingerprint: &str, ip: &str, port: u16) -> TargetInfo {
    TargetInfo {
        identity: Identity {
            fingerprint: fingerprint.to_string(),
            hostname: "test-host".to_string(),
        },
        addresses: vec![Address {
            ip: ip.to_string(),
            port,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }],
        saved_tunnels: vec![],
    }
}

#[test]
fn test_resolve_new_target() {
    let (_tmp, storage) = make_storage();
    let resolver = TargetResolver::new(&storage);

    let result = resolver.resolve("SHA256:new", "192.168.1.1", 22).unwrap();
    assert!(matches!(result, Resolution::NewTarget));
}

#[test]
fn test_resolve_existing_target_same_ip() {
    let (_tmp, storage) = make_storage();
    let target = make_target("SHA256:existing", "10.0.0.1", 22);
    storage.save_target(&target).unwrap();

    let resolver = TargetResolver::new(&storage);
    let result = resolver
        .resolve("SHA256:existing", "10.0.0.1", 22)
        .unwrap();

    assert!(matches!(result, Resolution::ExistingTarget { .. }));
}

#[test]
fn test_resolve_existing_target_new_ip() {
    let (_tmp, storage) = make_storage();
    let target = make_target("SHA256:existing", "10.0.0.1", 22);
    storage.save_target(&target).unwrap();

    let resolver = TargetResolver::new(&storage);
    // Same fingerprint, different IP
    let result = resolver
        .resolve("SHA256:existing", "10.0.0.2", 22)
        .unwrap();

    assert!(matches!(result, Resolution::ExistingTargetNewIp { .. }));
}

#[test]
fn test_resolve_same_ip_different_fingerprint() {
    let (_tmp, storage) = make_storage();
    let target = make_target("SHA256:old_fp", "10.0.0.1", 22);
    storage.save_target(&target).unwrap();

    let resolver = TargetResolver::new(&storage);
    // Different fingerprint, same IP/port → fingerprint changed
    let result = resolver
        .resolve("SHA256:new_fp", "10.0.0.1", 22)
        .unwrap();

    assert!(matches!(result, Resolution::FingerprintChanged { .. }));
}

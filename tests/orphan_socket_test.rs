use slipstream::ssh::orphan::OrphanDetector;
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

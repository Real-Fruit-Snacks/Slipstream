use slipstream::ssh::discovery::SshDiscovery;

#[test]
fn test_find_ssh_from_path() {
    let result = SshDiscovery::find_ssh(None, None);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.exists());
}

#[test]
fn test_find_ssh_with_config_override() {
    let result = SshDiscovery::find_ssh(Some("/usr/bin/ssh"), None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_str().unwrap(), "/usr/bin/ssh");
}

#[test]
fn test_find_ssh_config_override_missing_binary() {
    let result = SshDiscovery::find_ssh(Some("/nonexistent/ssh"), None);
    assert!(result.is_err());
}

#[test]
fn test_find_ssh_skips_self() {
    let result = SshDiscovery::find_ssh(None, Some("/usr/local/bin/slipstream"));
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(!path.to_str().unwrap().contains("slipstream"));
}

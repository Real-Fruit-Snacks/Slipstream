use slipstream::transfer::fallback::{FallbackChain, TransferMethod};

#[test]
fn test_default_fallback_chain() {
    let chain = FallbackChain::default();
    let methods = chain.methods();
    assert_eq!(methods.len(), 4);
    assert_eq!(methods[0], TransferMethod::Sftp);
    assert_eq!(methods[1], TransferMethod::Scp);
    assert_eq!(methods[2], TransferMethod::Cat);
    assert_eq!(methods[3], TransferMethod::Base64);
}

#[test]
fn test_fallback_chain_from_config() {
    let chain = FallbackChain::from_strings(&["scp", "base64"]);
    let methods = chain.methods();
    assert_eq!(methods.len(), 2);
    assert_eq!(methods[0], TransferMethod::Scp);
    assert_eq!(methods[1], TransferMethod::Base64);
}

#[test]
fn test_parse_method() {
    assert_eq!(TransferMethod::from_str("sftp"), Some(TransferMethod::Sftp));
    assert_eq!(TransferMethod::from_str("scp"), Some(TransferMethod::Scp));
    assert_eq!(TransferMethod::from_str("cat"), Some(TransferMethod::Cat));
    assert_eq!(TransferMethod::from_str("base64"), Some(TransferMethod::Base64));
    assert_eq!(TransferMethod::from_str("unknown"), None);
}

#[test]
fn test_build_sftp_command() {
    let cmd = TransferMethod::Sftp.upload_command(
        "/tmp/ctrl.sock",
        "user@host",
        "/local/file.txt",
        "/remote/file.txt",
    );
    assert!(cmd.contains("ControlPath"), "missing ControlPath: {}", cmd);
    assert!(cmd.contains("put"), "missing 'put': {}", cmd);
}

#[test]
fn test_build_scp_upload_command() {
    let cmd = TransferMethod::Scp.upload_command(
        "/tmp/ctrl.sock",
        "user@host",
        "/local/file.txt",
        "/remote/file.txt",
    );
    assert!(cmd.contains("scp"), "missing 'scp': {}", cmd);
    assert!(cmd.contains("ControlPath"), "missing ControlPath: {}", cmd);
}

#[test]
fn test_build_cat_upload_command() {
    let cmd = TransferMethod::Cat.upload_command(
        "/tmp/ctrl.sock",
        "user@host",
        "/local/file.txt",
        "/remote/file.txt",
    );
    assert!(cmd.contains("ssh"), "missing 'ssh': {}", cmd);
    assert!(cmd.contains("-S"), "missing '-S': {}", cmd);
    assert!(cmd.contains("cat >"), "missing 'cat >': {}", cmd);
}

#[test]
fn test_build_base64_upload_command() {
    let cmd = TransferMethod::Base64.upload_command(
        "/tmp/ctrl.sock",
        "user@host",
        "/local/file.txt",
        "/remote/file.txt",
    );
    assert!(cmd.contains("base64"), "missing 'base64': {}", cmd);
    assert!(cmd.contains("ssh"), "missing 'ssh': {}", cmd);
    assert!(cmd.contains("-S"), "missing '-S': {}", cmd);
}

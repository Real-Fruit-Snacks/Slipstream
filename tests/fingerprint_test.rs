use slipstream::ssh::fingerprint::FingerprintParser;

#[test]
fn test_parse_ed25519_fingerprint() {
    let line = "debug1: Server host key: ssh-ed25519 SHA256:abcdef123456ghijkl";
    assert_eq!(FingerprintParser::parse_line(line), Some("SHA256:abcdef123456ghijkl".to_string()));
}

#[test]
fn test_parse_rsa_fingerprint() {
    let line = "debug1: Server host key: ssh-rsa SHA256:xyz789abcdef000111";
    assert_eq!(FingerprintParser::parse_line(line), Some("SHA256:xyz789abcdef000111".to_string()));
}

#[test]
fn test_parse_non_fingerprint_line() {
    assert_eq!(FingerprintParser::parse_line("debug1: Connection established."), None);
}

#[test]
fn test_parse_from_multiple_lines() {
    let stderr = "debug1: Connecting to 10.10.10.5\ndebug1: Server host key: ssh-ed25519 SHA256:realfp123\ndebug1: Auth succeeded.";
    assert_eq!(FingerprintParser::parse_from_output(stderr), Some("SHA256:realfp123".to_string()));
}

#[test]
fn test_parse_no_fingerprint() {
    assert_eq!(FingerprintParser::parse_from_output("debug1: Connected."), None);
}

#[test]
fn test_fallback_known_hosts_command() {
    assert_eq!(FingerprintParser::known_hosts_lookup_command("10.10.10.5"), vec!["ssh-keygen", "-F", "10.10.10.5"]);
}

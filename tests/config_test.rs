use slipstream::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.sessions.escape_prefix, "!");
    assert_eq!(config.transfers.default_method, "sftp");
    assert_eq!(config.transfers.fallback_chain, vec!["sftp", "scp", "cat", "base64"]);
    assert!(config.logging.enabled);
    assert!(config.map.enabled);
    assert!(config.map.parse_ls);
}

#[test]
fn test_config_from_toml_string() {
    let toml_str = r#"
[sessions]
escape_prefix = "@"
notify_disconnect = false

[transfers]
default_method = "scp"
"#;
    let config = Config::from_str(toml_str).unwrap();
    assert_eq!(config.sessions.escape_prefix, "@");
    assert!(!config.sessions.notify_disconnect);
    assert_eq!(config.transfers.default_method, "scp");
    assert!(config.logging.enabled);
}

#[test]
fn test_config_load_missing_file_returns_defaults() {
    let config = Config::load_from("/nonexistent/path/config.toml");
    assert_eq!(config.sessions.escape_prefix, "!");
}

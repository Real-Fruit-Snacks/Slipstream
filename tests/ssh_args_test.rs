use slipstream::ssh::args::SshArgs;

#[test]
fn test_parse_simple_target() {
    let args = SshArgs::parse(&["user@10.10.10.5".to_string()]);
    assert_eq!(args.user, Some("user".to_string()));
    assert_eq!(args.host, "10.10.10.5");
    assert_eq!(args.port, 22);
    assert_eq!(args.passthrough, vec!["user@10.10.10.5"]);
}

#[test]
fn test_parse_with_port_flag() {
    let args = SshArgs::parse(&["-p".to_string(), "2222".to_string(), "root@target".to_string()]);
    assert_eq!(args.user, Some("root".to_string()));
    assert_eq!(args.host, "target");
    assert_eq!(args.port, 2222);
}

#[test]
fn test_parse_with_identity_and_options() {
    let args = SshArgs::parse(&[
        "-i".to_string(), "/root/.ssh/id_rsa".to_string(),
        "-o".to_string(), "StrictHostKeyChecking=no".to_string(),
        "admin@10.10.10.20".to_string(),
    ]);
    assert_eq!(args.user, Some("admin".to_string()));
    assert_eq!(args.host, "10.10.10.20");
    assert!(args.passthrough.contains(&"-i".to_string()));
}

#[test]
fn test_parse_host_only_no_user() {
    let args = SshArgs::parse(&["10.10.10.5".to_string()]);
    assert_eq!(args.user, None);
    assert_eq!(args.host, "10.10.10.5");
}

#[test]
fn test_parse_with_jump_host() {
    let args = SshArgs::parse(&[
        "-J".to_string(), "pivot@10.10.10.1".to_string(),
        "user@192.168.1.50".to_string(),
    ]);
    assert_eq!(args.user, Some("user".to_string()));
    assert_eq!(args.host, "192.168.1.50");
    assert!(args.passthrough.contains(&"-J".to_string()));
}

#[test]
fn test_passthrough_preserves_all_args() {
    let input = vec![
        "-o".to_string(), "StrictHostKeyChecking=no".to_string(),
        "-L".to_string(), "8080:localhost:80".to_string(),
        "user@host".to_string(),
    ];
    let args = SshArgs::parse(&input);
    assert_eq!(args.passthrough, input);
}

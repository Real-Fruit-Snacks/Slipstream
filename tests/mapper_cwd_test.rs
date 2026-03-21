use slipstream::mapper::cwd::CwdTracker;
use slipstream::mapper::parser::OutputParser;
use slipstream::target_os::TargetOS;

#[test]
fn test_cwd_tracker_initial() {
    let tracker = CwdTracker::new(TargetOS::Unix);
    assert_eq!(tracker.current(), "/");
}

#[test]
fn test_cwd_update_from_pwd() {
    let mut tracker = CwdTracker::new(TargetOS::Unix);
    tracker.update_from_pwd("/home/user/projects\n");
    assert_eq!(tracker.current(), "/home/user/projects");
}

#[test]
fn test_cwd_update_from_cd_absolute() {
    let mut tracker = CwdTracker::new(TargetOS::Unix);
    tracker.update_from_cd("cd /var/log");
    assert_eq!(tracker.current(), "/var/log");
}

#[test]
fn test_cwd_cd_relative() {
    let mut tracker = CwdTracker::new(TargetOS::Unix);
    tracker.update_from_pwd("/home/user\n");
    tracker.update_from_cd("cd projects");
    assert_eq!(tracker.current(), "/home/user/projects");
}

#[test]
fn test_cwd_cd_parent() {
    let mut tracker = CwdTracker::new(TargetOS::Unix);
    tracker.update_from_pwd("/home/user/projects\n");
    tracker.update_from_cd("cd ..");
    assert_eq!(tracker.current(), "/home/user");
}

#[test]
fn test_cwd_cd_home_no_change() {
    let mut tracker = CwdTracker::new(TargetOS::Unix);
    tracker.update_from_pwd("/home/user\n");
    tracker.update_from_cd("cd ~");
    // ~ is ignored (no shell expansion), cwd remains
    assert_eq!(tracker.current(), "/home/user");
}

#[test]
fn test_parse_tree_output() {
    let output = "\
.
├── src
│   ├── main.rs
│   └── lib.rs
└── Cargo.toml
";
    let entries = OutputParser::parse_tree(output, "/project");
    // Should have non-empty entries for each file/dir name
    assert!(!entries.is_empty());
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"src"), "expected 'src' in {:?}", names);
    assert!(names.contains(&"main.rs"), "expected 'main.rs' in {:?}", names);
    assert!(names.contains(&"lib.rs"), "expected 'lib.rs' in {:?}", names);
    assert!(names.contains(&"Cargo.toml"), "expected 'Cargo.toml' in {:?}", names);
    // All paths should start with /project/
    for entry in &entries {
        assert!(entry.path.starts_with("/project/"), "unexpected path: {}", entry.path);
    }
}

#[test]
fn test_parse_network_output() {
    let output = "\
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
    inet6 ::1/128 scope host
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP group default
    link/ether aa:bb:cc:dd:ee:ff brd ff:ff:ff:ff:ff:ff
    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0
    inet6 fe80::1/64 scope link
";
    let interfaces = OutputParser::parse_ip_a(output);
    assert_eq!(interfaces.len(), 2, "expected 2 interfaces, got {}", interfaces.len());

    let lo = &interfaces[0];
    assert_eq!(lo.name, "lo");
    assert_eq!(lo.ipv4.as_deref(), Some("127.0.0.1/8"));
    assert_eq!(lo.ipv6.as_deref(), Some("::1/128"));

    let eth0 = &interfaces[1];
    assert_eq!(eth0.name, "eth0");
    assert_eq!(eth0.ipv4.as_deref(), Some("192.168.1.100/24"));
    assert_eq!(eth0.ipv6.as_deref(), Some("fe80::1/64"));
}

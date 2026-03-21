use slipstream::mapper::parser::{EntryType, OutputParser};

#[test]
fn test_parse_ls_simple() {
    let output = "foo.txt\nbar.txt\nbaz.txt\n";
    let entries = OutputParser::parse_ls(output, "/home/user");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "/home/user/foo.txt");
    assert_eq!(entries[0].name, "foo.txt");
    assert_eq!(entries[1].path, "/home/user/bar.txt");
    assert_eq!(entries[2].path, "/home/user/baz.txt");
}

#[test]
fn test_parse_ls_la() {
    let output = r#"total 24
drwxr-xr-x  2 root root 4096 Jan  1 12:00 .
drwxr-xr-x 20 root root 4096 Jan  1 12:00 ..
-rw-r--r--  1 root root  220 Jan  1 12:00 hosts
-rwsr-xr-x  1 root root 8192 Jan  1 12:00 sudo
drwxr-xr-x  2 root root 4096 Jan  1 12:00 etc
"#;
    let entries = OutputParser::parse_ls_la(output, "/etc");
    // Should skip total, ., ..
    assert_eq!(entries.len(), 3);

    let hosts = entries.iter().find(|e| e.name == "hosts").unwrap();
    assert_eq!(hosts.permissions.as_deref(), Some("-rw-r--r--"));
    assert_eq!(hosts.owner.as_deref(), Some("root"));
    assert_eq!(hosts.size, Some(220));
    assert!(!hosts.is_suid());

    let sudo = entries.iter().find(|e| e.name == "sudo").unwrap();
    assert!(sudo.is_suid(), "sudo should be SUID");
    assert_eq!(sudo.entry_type, EntryType::File);

    let etc = entries.iter().find(|e| e.name == "etc").unwrap();
    assert_eq!(etc.entry_type, EntryType::Directory);
}

#[test]
fn test_parse_find_output() {
    let output = "/etc/passwd\n/etc/shadow\n/var/log/auth.log\n";
    let entries = OutputParser::parse_find(output);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "/etc/passwd");
    assert_eq!(entries[0].name, "passwd");
    assert_eq!(entries[1].path, "/etc/shadow");
    assert_eq!(entries[2].path, "/var/log/auth.log");
}

#[test]
fn test_parse_passwd() {
    let output = r#"root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
www-data:x:33:33:www-data:/var/www:/usr/sbin/nologin
"#;
    let users = OutputParser::parse_passwd(output);
    assert_eq!(users.len(), 3);

    assert_eq!(users[0].username, "root");
    assert_eq!(users[0].uid, 0);
    assert_eq!(users[0].gid, 0);
    assert_eq!(users[0].home, "/root");
    assert_eq!(users[0].shell, "/bin/bash");

    assert_eq!(users[1].username, "daemon");
    assert_eq!(users[1].uid, 1);

    assert_eq!(users[2].username, "www-data");
    assert_eq!(users[2].uid, 33);
    assert_eq!(users[2].home, "/var/www");
}

#[test]
fn test_detect_command_type() {
    assert_eq!(OutputParser::detect_command("ls -la /etc"), Some("ls_la"));
    assert_eq!(OutputParser::detect_command("ls /home"), Some("ls"));
    assert_eq!(OutputParser::detect_command("find / -name '*.conf'"), Some("find"));
    assert_eq!(OutputParser::detect_command("cat /etc/passwd"), Some("passwd"));
    assert_eq!(OutputParser::detect_command("pwd"), Some("pwd"));
    assert_eq!(OutputParser::detect_command("vim /etc/hosts"), None);
}

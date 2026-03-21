use slipstream::mapper::parser::{EntryType, ParsedEntry};
use slipstream::mapper::store::MapStore;
use slipstream::mapper::query::MapQuery;

fn make_entry(path: &str, name: &str, entry_type: EntryType) -> ParsedEntry {
    ParsedEntry {
        path: path.to_string(),
        name: name.to_string(),
        entry_type,
        permissions: None,
        owner: None,
        size: None,
    }
}

fn make_entry_with_perms(path: &str, name: &str, perms: &str) -> ParsedEntry {
    ParsedEntry {
        path: path.to_string(),
        name: name.to_string(),
        entry_type: EntryType::File,
        permissions: Some(perms.to_string()),
        owner: None,
        size: None,
    }
}

#[test]
fn test_query_directory() {
    let mut store = MapStore::new_empty();
    store.add_entry(make_entry("/etc/hosts", "hosts", EntryType::File));
    store.add_entry(make_entry("/etc/passwd", "passwd", EntryType::File));
    store.add_entry(make_entry("/var/log/auth.log", "auth.log", EntryType::File));
    store.add_entry(make_entry("/etc/ssh/sshd_config", "sshd_config", EntryType::File));

    let results = MapQuery::list_directory(&store, "/etc");
    // Only direct children of /etc, not /etc/ssh/sshd_config
    assert_eq!(results.len(), 2);
    let paths: Vec<&str> = results.iter().map(|e| e.path.as_str()).collect();
    assert!(paths.contains(&"/etc/hosts"));
    assert!(paths.contains(&"/etc/passwd"));
}

#[test]
fn test_query_find_pattern() {
    let mut store = MapStore::new_empty();
    store.add_entry(make_entry("/etc/nginx.conf", "nginx.conf", EntryType::File));
    store.add_entry(make_entry("/etc/ssh/sshd_config", "sshd_config", EntryType::File));
    store.add_entry(make_entry("/etc/hosts", "hosts", EntryType::File));
    store.add_entry(make_entry("/var/app.conf", "app.conf", EntryType::File));

    let results = MapQuery::find(&store, "*.conf");
    assert_eq!(results.len(), 2);
    let names: Vec<&str> = results.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"nginx.conf"));
    assert!(names.contains(&"app.conf"));
}

#[test]
fn test_query_find_suid() {
    let mut store = MapStore::new_empty();
    store.add_entry(make_entry_with_perms("/usr/bin/sudo", "sudo", "-rwsr-xr-x"));
    store.add_entry(make_entry_with_perms("/usr/bin/passwd", "passwd", "-rwsr-xr-x"));
    store.add_entry(make_entry_with_perms("/etc/hosts", "hosts", "-rw-r--r--"));

    let results = MapQuery::find(&store, "suid");
    assert_eq!(results.len(), 2);
    let names: Vec<&str> = results.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"sudo"));
    assert!(names.contains(&"passwd"));
}

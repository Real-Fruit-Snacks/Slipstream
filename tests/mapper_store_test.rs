use slipstream::mapper::parser::{EntryType, ParsedEntry};
use slipstream::mapper::store::MapStore;
use tempfile::tempdir;

fn make_entry(path: &str, name: &str) -> ParsedEntry {
    ParsedEntry {
        path: path.to_string(),
        name: name.to_string(),
        entry_type: EntryType::File,
        permissions: None,
        owner: None,
        size: None,
    }
}

#[test]
fn test_new_empty_map() {
    let store = MapStore::new_empty();
    assert!(store.entries().is_empty());
    assert!(store.users().is_empty());
}

#[test]
fn test_add_entries_and_save() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("map.json");

    let mut store = MapStore::new_empty();
    store.add_entry(make_entry("/etc/hosts", "hosts"));
    store.add_entry(make_entry("/etc/passwd", "passwd"));
    store.save(&path).unwrap();

    let loaded = MapStore::load_or_create(&path);
    assert_eq!(loaded.entries().len(), 2);
    assert_eq!(loaded.entries()[0].path, "/etc/hosts");
    assert_eq!(loaded.entries()[1].path, "/etc/passwd");
}

#[test]
fn test_merge_does_not_duplicate() {
    let mut store = MapStore::new_empty();
    store.add_entry(make_entry("/etc/hosts", "hosts"));
    store.add_entry(make_entry("/etc/hosts", "hosts")); // duplicate
    assert_eq!(store.entries().len(), 1);
}

#[test]
fn test_reset_clears_map() {
    let mut store = MapStore::new_empty();
    store.add_entry(make_entry("/etc/hosts", "hosts"));
    store.add_entry(make_entry("/etc/passwd", "passwd"));
    assert_eq!(store.entries().len(), 2);

    store.reset();
    assert!(store.entries().is_empty());
    assert!(store.users().is_empty());
}

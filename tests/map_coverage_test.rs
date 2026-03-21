use slipstream::mapper::store::MapStore;
use slipstream::mapper::parser::{ParsedEntry, EntryType};
use slipstream::mapper::query::MapQuery;

#[test]
fn test_map_coverage() {
    let mut store = MapStore::new_empty();
    store.add_entry(ParsedEntry { path: "/etc/passwd".to_string(), name: "passwd".to_string(), entry_type: EntryType::File, permissions: None, owner: None, size: None });
    store.add_entry(ParsedEntry { path: "/var/log/auth.log".to_string(), name: "auth.log".to_string(), entry_type: EntryType::File, permissions: None, owner: None, size: None });
    let coverage = MapQuery::coverage(&store);
    assert!(coverage.contains("/etc"));
    assert!(coverage.contains("/var/log"));
    assert!(coverage.contains("2 entries"));
}

#[test]
fn test_map_export_json() {
    let mut store = MapStore::new_empty();
    store.add_entry(ParsedEntry { path: "/tmp/test".to_string(), name: "test".to_string(), entry_type: EntryType::File, permissions: None, owner: None, size: None });
    let json = MapQuery::export_json(&store);
    assert!(json.contains("/tmp/test"));
}

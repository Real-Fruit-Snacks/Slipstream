use slipstream::session::manager::{Session, SessionManager, SessionState};

fn make_session(user: &str, host: &str, hostname: Option<&str>, port: u16) -> Session {
    Session {
        user: user.to_string(),
        host: host.to_string(),
        hostname: hostname.map(|s| s.to_string()),
        port,
        state: SessionState::Connected,
        label: None,
    }
}

#[test]
fn test_create_session() {
    let mut mgr = SessionManager::new();
    let id = mgr.create(make_session("root", "10.0.0.1", Some("victim01"), 22));
    assert_eq!(id, 1);
    assert_eq!(mgr.list().len(), 1);
    let s = mgr.get(id).unwrap();
    assert_eq!(s.user, "root");
    assert_eq!(s.host, "10.0.0.1");
    assert_eq!(s.hostname.as_deref(), Some("victim01"));
}

#[test]
fn test_active_session() {
    let mut mgr = SessionManager::new();
    assert_eq!(mgr.active_id(), None);
    let id = mgr.create(make_session("admin", "192.168.1.1", None, 22));
    assert_eq!(mgr.active_id(), Some(id));
    let id2 = mgr.create(make_session("user", "192.168.1.2", None, 22));
    // first session stays active after second is created
    assert_eq!(mgr.active_id(), Some(id));
    // set_active changes it
    mgr.set_active(id2);
    assert_eq!(mgr.active_id(), Some(id2));
}

#[test]
fn test_switch_session() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create(make_session("root", "10.0.0.1", None, 22));
    let id2 = mgr.create(make_session("root", "10.0.0.2", None, 22));
    assert_eq!(mgr.active_id(), Some(id1));
    assert!(mgr.switch_to(id2));
    assert_eq!(mgr.active_id(), Some(id2));
    assert!(!mgr.switch_to(999));
    // active unchanged after failed switch
    assert_eq!(mgr.active_id(), Some(id2));
}

#[test]
fn test_kill_session() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create(make_session("root", "10.0.0.1", None, 22));
    let id2 = mgr.create(make_session("root", "10.0.0.2", None, 22));
    // kill the active session — active should fall to first remaining
    assert!(mgr.kill(id1));
    assert_eq!(mgr.list().len(), 1);
    assert_eq!(mgr.active_id(), Some(id2));
    // kill nonexistent
    assert!(!mgr.kill(999));
    // kill last remaining
    assert!(mgr.kill(id2));
    assert_eq!(mgr.list().len(), 0);
    assert_eq!(mgr.active_id(), None);
}

#[test]
fn test_rename_session() {
    let mut mgr = SessionManager::new();
    let id = mgr.create(make_session("root", "10.0.0.1", None, 22));
    assert!(mgr.rename(id, "pivot-box".to_string()));
    assert_eq!(mgr.get(id).unwrap().label.as_deref(), Some("pivot-box"));
    assert!(!mgr.rename(999, "nope".to_string()));
}

#[test]
fn test_format_sessions_list() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create(make_session("root", "10.10.10.5", Some("victim01"), 22));
    let id2 = mgr.create(make_session("admin", "10.10.10.6", Some("victim02"), 22));
    mgr.rename(id1, "main-pivot".to_string());
    // id1 is active by default
    let output = mgr.format_list();
    assert!(output.contains("victim01"), "should contain hostname victim01");
    assert!(output.contains("\u{25c4} active"), "should contain active marker");
    assert!(output.contains("main-pivot"), "should contain label text");
    assert!(output.contains("victim02"), "should contain second session hostname");
    // switch active and verify marker moves
    mgr.switch_to(id2);
    let output2 = mgr.format_list();
    assert!(output2.contains("victim02"));
    assert!(output2.contains("\u{25c4} active"));
}

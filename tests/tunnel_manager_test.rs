use slipstream::tunnel::manager::{Tunnel, TunnelManager, TunnelType};

fn make_local_tunnel(session_id: u32) -> Tunnel {
    Tunnel {
        tunnel_type: TunnelType::Local,
        source_port: 8080,
        dest_host: Some("10.10.10.50".to_string()),
        dest_port: Some(80),
        session_id,
    }
}

fn make_socks_tunnel(session_id: u32) -> Tunnel {
    Tunnel {
        tunnel_type: TunnelType::Socks,
        source_port: 1080,
        dest_host: None,
        dest_port: None,
        session_id,
    }
}

#[test]
fn test_add_local_tunnel() {
    let mut mgr = TunnelManager::new();
    let id = mgr.add(make_local_tunnel(1));
    assert_eq!(id, 1);
    assert_eq!(mgr.list().len(), 1);
}

#[test]
fn test_add_socks_tunnel() {
    let mut mgr = TunnelManager::new();
    let id = mgr.add(make_socks_tunnel(1));
    assert_eq!(id, 1);
    assert_eq!(mgr.list().len(), 1);
    assert_eq!(mgr.list()[0].1.tunnel_type, TunnelType::Socks);
}

#[test]
fn test_delete_tunnel() {
    let mut mgr = TunnelManager::new();
    let id = mgr.add(make_local_tunnel(1));
    assert!(mgr.delete(id));
    assert_eq!(mgr.list().len(), 0);
}

#[test]
fn test_delete_nonexistent_tunnel() {
    let mut mgr = TunnelManager::new();
    assert!(!mgr.delete(999));
}

#[test]
fn test_delete_by_session() {
    let mut mgr = TunnelManager::new();
    mgr.add(make_local_tunnel(42));
    mgr.add(make_socks_tunnel(42));
    mgr.add(make_local_tunnel(99));
    let removed = mgr.delete_by_session(42);
    assert_eq!(removed, 2);
    assert_eq!(mgr.list().len(), 1);
    assert_eq!(mgr.list()[0].1.session_id, 99);
}

#[test]
fn test_flush() {
    let mut mgr = TunnelManager::new();
    mgr.add(make_local_tunnel(1));
    mgr.add(make_socks_tunnel(2));
    mgr.flush();
    assert_eq!(mgr.list().len(), 0);
}

#[test]
fn test_build_ssh_forward_args_local() {
    let t = make_local_tunnel(1);
    assert_eq!(t.to_ssh_forward_arg(), "8080:10.10.10.50:80");
}

#[test]
fn test_build_ssh_forward_args_socks() {
    let t = make_socks_tunnel(1);
    assert_eq!(t.to_ssh_dynamic_arg(), "1080");
}

#[test]
fn test_parse_tunnel_add_args_local() {
    let t = Tunnel::parse_add_args("--type local -s 8080 -d 10.10.10.50 -p 80", 1).unwrap();
    assert_eq!(t.tunnel_type, TunnelType::Local);
    assert_eq!(t.source_port, 8080);
    assert_eq!(t.dest_host.as_deref(), Some("10.10.10.50"));
    assert_eq!(t.dest_port, Some(80));
    assert_eq!(t.session_id, 1);
}

#[test]
fn test_parse_tunnel_add_args_socks() {
    let t = Tunnel::parse_add_args("--type socks -p 1080", 2).unwrap();
    assert_eq!(t.tunnel_type, TunnelType::Socks);
    assert_eq!(t.source_port, 1080);
    assert_eq!(t.dest_host, None);
    assert_eq!(t.dest_port, None);
    assert_eq!(t.session_id, 2);
}

#[test]
fn test_parse_tunnel_add_args_reverse() {
    let t = Tunnel::parse_add_args("--type reverse -s 9090 -d 127.0.0.1 -p 3000", 3).unwrap();
    assert_eq!(t.tunnel_type, TunnelType::Reverse);
    assert_eq!(t.source_port, 9090);
    assert_eq!(t.dest_host.as_deref(), Some("127.0.0.1"));
    assert_eq!(t.dest_port, Some(3000));
    assert_eq!(t.session_id, 3);
}

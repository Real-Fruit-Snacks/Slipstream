use slipstream::tunnel::manager::{Tunnel, TunnelManager, TunnelType};

#[test]
fn test_export_tunnels_to_saved() {
    let mut mgr = TunnelManager::new();
    mgr.add(Tunnel { tunnel_type: TunnelType::Socks, source_port: 1080, dest_host: None, dest_port: None, session_id: 1 });
    mgr.add(Tunnel { tunnel_type: TunnelType::Local, source_port: 8080, dest_host: Some("10.10.10.50".to_string()), dest_port: Some(80), session_id: 1 });
    let saved = mgr.export_as_saved(1);
    assert_eq!(saved.len(), 2);
}

#[test]
fn test_import_from_saved() {
    use slipstream::target::storage::SavedTunnel;
    let saved = vec![SavedTunnel {
        tunnel_type: "socks".to_string(), port: Some(1080), source: None,
        dest_host: None, dest_port: None, auto_restore: false,
    }];
    let tunnels = TunnelManager::import_from_saved(&saved, 1);
    assert_eq!(tunnels.len(), 1);
    assert_eq!(tunnels[0].source_port, 1080);
}

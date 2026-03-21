use crate::tunnel::manager::{Tunnel, TunnelManager, TunnelType};

fn tunnel_type_str(t: &TunnelType) -> &'static str {
    match t {
        TunnelType::Local => "local",
        TunnelType::Socks => "socks",
        TunnelType::Reverse => "reverse",
    }
}

fn tunnel_source(t: &Tunnel) -> String {
    format!(":{}", t.source_port)
}

fn tunnel_dest(t: &Tunnel) -> String {
    match (&t.dest_host, t.dest_port) {
        (Some(h), Some(p)) => format!("{}:{}", h, p),
        (Some(h), None) => h.clone(),
        (None, Some(p)) => format!(":{}", p),
        (None, None) => "-".to_string(),
    }
}

/// Format the tunnel list as a table.
///
/// Non-verbose columns: #, TYPE, SOURCE, DEST, STATUS
/// Verbose adds:        SESSION, CREATED (CREATED is always "-" as no timestamp is stored)
pub fn format_tunnel_list(mgr: &TunnelManager, verbose: bool) -> String {
    let tunnels = mgr.list();

    if tunnels.is_empty() {
        return "No tunnels active.\n".to_string();
    }

    if verbose {
        let header = format!(
            "{:<4} {:<8} {:<16} {:<22} {:<8} {:<10} {:<8}\n",
            "#", "TYPE", "SOURCE", "DEST", "STATUS", "SESSION", "CREATED"
        );
        let sep = format!(
            "{} {} {} {} {} {} {}\n",
            "-".repeat(4),
            "-".repeat(8),
            "-".repeat(16),
            "-".repeat(22),
            "-".repeat(8),
            "-".repeat(10),
            "-".repeat(8),
        );
        let mut out = header;
        out.push_str(&sep);
        for (id, t) in tunnels {
            out.push_str(&format!(
                "{:<4} {:<8} {:<16} {:<22} {:<8} {:<10} {:<8}\n",
                id,
                tunnel_type_str(&t.tunnel_type),
                tunnel_source(t),
                tunnel_dest(t),
                "active",
                t.session_id,
                "-",
            ));
        }
        out
    } else {
        let header = format!(
            "{:<4} {:<8} {:<16} {:<22} {:<8}\n",
            "#", "TYPE", "SOURCE", "DEST", "STATUS"
        );
        let sep = format!(
            "{} {} {} {} {}\n",
            "-".repeat(4),
            "-".repeat(8),
            "-".repeat(16),
            "-".repeat(22),
            "-".repeat(8),
        );
        let mut out = header;
        out.push_str(&sep);
        for (id, t) in tunnels {
            out.push_str(&format!(
                "{:<4} {:<8} {:<16} {:<22} {:<8}\n",
                id,
                tunnel_type_str(&t.tunnel_type),
                tunnel_source(t),
                tunnel_dest(t),
                "active",
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mgr() -> TunnelManager {
        let mut mgr = TunnelManager::new();
        mgr.add(Tunnel {
            tunnel_type: TunnelType::Local,
            source_port: 8080,
            dest_host: Some("127.0.0.1".to_string()),
            dest_port: Some(80),
            session_id: 1,
        });
        mgr.add(Tunnel {
            tunnel_type: TunnelType::Socks,
            source_port: 1080,
            dest_host: None,
            dest_port: None,
            session_id: 2,
        });
        mgr
    }

    #[test]
    fn empty_manager() {
        let mgr = TunnelManager::new();
        let out = format_tunnel_list(&mgr, false);
        assert_eq!(out, "No tunnels active.\n");
    }

    #[test]
    fn non_verbose_contains_columns() {
        let mgr = make_mgr();
        let out = format_tunnel_list(&mgr, false);
        assert!(out.contains("TYPE"));
        assert!(out.contains("SOURCE"));
        assert!(out.contains("DEST"));
        assert!(out.contains("STATUS"));
        assert!(!out.contains("SESSION"));
        assert!(out.contains("local"));
        assert!(out.contains("socks"));
    }

    #[test]
    fn verbose_contains_session_column() {
        let mgr = make_mgr();
        let out = format_tunnel_list(&mgr, true);
        assert!(out.contains("SESSION"));
        assert!(out.contains("CREATED"));
    }
}

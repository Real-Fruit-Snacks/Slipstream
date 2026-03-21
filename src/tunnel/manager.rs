use crate::target::storage::SavedTunnel;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("missing required flag: {0}")]
    MissingFlag(String),
    #[error("invalid tunnel type: {0}")]
    InvalidType(String),
    #[error("invalid port: {0}")]
    InvalidPort(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TunnelType {
    Local,
    Socks,
    Reverse,
}

#[derive(Debug, Clone)]
pub struct Tunnel {
    pub tunnel_type: TunnelType,
    pub source_port: u16,
    pub dest_host: Option<String>,
    pub dest_port: Option<u16>,
    pub session_id: u32,
}

impl Tunnel {
    pub fn parse_add_args(args: &str, session_id: u32) -> Result<Self, TunnelError> {
        let tokens: Vec<&str> = args.split_whitespace().collect();
        let mut tunnel_type: Option<TunnelType> = None;
        let mut source_port: Option<u16> = None;
        let mut dest_host: Option<String> = None;
        let mut dest_port: Option<u16> = None;

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "--type" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(TunnelError::MissingFlag("--type value".to_string()));
                    }
                    tunnel_type = Some(match tokens[i] {
                        "local" => TunnelType::Local,
                        "socks" => TunnelType::Socks,
                        "reverse" => TunnelType::Reverse,
                        other => return Err(TunnelError::InvalidType(other.to_string())),
                    });
                }
                "-s" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(TunnelError::MissingFlag("-s value".to_string()));
                    }
                    source_port = Some(
                        tokens[i]
                            .parse::<u16>()
                            .map_err(|_| TunnelError::InvalidPort(tokens[i].to_string()))?,
                    );
                }
                "-d" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(TunnelError::MissingFlag("-d value".to_string()));
                    }
                    dest_host = Some(tokens[i].to_string());
                }
                "-p" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(TunnelError::MissingFlag("-p value".to_string()));
                    }
                    dest_port = Some(
                        tokens[i]
                            .parse::<u16>()
                            .map_err(|_| TunnelError::InvalidPort(tokens[i].to_string()))?,
                    );
                }
                _ => {}
            }
            i += 1;
        }

        let ttype = tunnel_type.ok_or_else(|| TunnelError::MissingFlag("--type".to_string()))?;

        let (final_source_port, final_dest_port) = match ttype {
            TunnelType::Socks => {
                // For socks: use -p or -s as source_port, no dest needed
                let sp = dest_port
                    .or(source_port)
                    .ok_or_else(|| TunnelError::MissingFlag("-p or -s".to_string()))?;
                (sp, None)
            }
            TunnelType::Local | TunnelType::Reverse => {
                let sp = source_port
                    .ok_or_else(|| TunnelError::MissingFlag("-s".to_string()))?;
                (sp, dest_port)
            }
        };

        Ok(Tunnel {
            tunnel_type: ttype,
            source_port: final_source_port,
            dest_host,
            dest_port: final_dest_port,
            session_id,
        })
    }

    pub fn to_ssh_forward_arg(&self) -> String {
        format!(
            "{}:{}:{}",
            self.source_port,
            self.dest_host.as_deref().unwrap_or(""),
            self.dest_port.map(|p| p.to_string()).unwrap_or_default()
        )
    }

    pub fn to_ssh_dynamic_arg(&self) -> String {
        self.source_port.to_string()
    }

    pub fn to_ssh_reverse_arg(&self) -> String {
        format!(
            "{}:{}:{}",
            self.source_port,
            self.dest_host.as_deref().unwrap_or(""),
            self.dest_port.map(|p| p.to_string()).unwrap_or_default()
        )
    }
}

pub struct TunnelManager {
    tunnels: Vec<(u32, Tunnel)>,
    next_id: u32,
}

impl TunnelManager {
    pub fn new() -> Self {
        TunnelManager {
            tunnels: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, tunnel: Tunnel) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.tunnels.push((id, tunnel));
        id
    }

    pub fn delete(&mut self, id: u32) -> bool {
        let before = self.tunnels.len();
        self.tunnels.retain(|(tid, _)| *tid != id);
        self.tunnels.len() < before
    }

    pub fn delete_by_session(&mut self, session_id: u32) -> usize {
        let before = self.tunnels.len();
        self.tunnels.retain(|(_, t)| t.session_id != session_id);
        before - self.tunnels.len()
    }

    pub fn flush(&mut self) {
        self.tunnels.clear();
    }

    pub fn list(&self) -> &[(u32, Tunnel)] {
        &self.tunnels
    }

    pub fn get(&self, id: u32) -> Option<&Tunnel> {
        self.tunnels
            .iter()
            .find(|(tid, _)| *tid == id)
            .map(|(_, t)| t)
    }

    pub fn export_as_saved(&self, session_id: u32) -> Vec<SavedTunnel> {
        self.tunnels
            .iter()
            .filter(|(_, t)| t.session_id == session_id)
            .map(|(_, t)| {
                let type_str = match t.tunnel_type {
                    TunnelType::Local => "local",
                    TunnelType::Socks => "socks",
                    TunnelType::Reverse => "reverse",
                }
                .to_string();
                SavedTunnel {
                    tunnel_type: type_str,
                    port: Some(t.source_port),
                    source: None,
                    dest_host: t.dest_host.clone(),
                    dest_port: t.dest_port,
                    auto_restore: false,
                }
            })
            .collect()
    }

    pub fn import_from_saved(saved: &[SavedTunnel], session_id: u32) -> Vec<Tunnel> {
        saved
            .iter()
            .filter_map(|s| {
                let tunnel_type = match s.tunnel_type.as_str() {
                    "local" => TunnelType::Local,
                    "socks" => TunnelType::Socks,
                    "reverse" => TunnelType::Reverse,
                    _ => return None,
                };
                let source_port = s.port?;
                Some(Tunnel {
                    tunnel_type,
                    source_port,
                    dest_host: s.dest_host.clone(),
                    dest_port: s.dest_port,
                    session_id,
                })
            })
            .collect()
    }
}

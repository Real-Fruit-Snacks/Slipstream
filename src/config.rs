use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub sessions: SessionsConfig,
    pub logging: LoggingConfig,
    pub transfers: TransfersConfig,
    pub tunnels: TunnelsConfig,
    pub map: MapConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            sessions: SessionsConfig::default(),
            logging: LoggingConfig::default(),
            transfers: TransfersConfig::default(),
            tunnels: TunnelsConfig::default(),
            map: MapConfig::default(),
        }
    }
}

impl Config {
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(path) {
            Ok(contents) => Self::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn load() -> Self {
        let config_path = dirs::home_dir()
            .map(|h| h.join(".config").join("slipstream").join("config.toml"));
        match config_path {
            Some(p) => Self::load_from(p),
            None => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub ssh_binary: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            ssh_binary: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionsConfig {
    pub escape_prefix: String,
    pub notify_disconnect: bool,
    pub auto_reconnect: bool,
    pub reconnect_max_attempts: u32,
    pub reconnect_backoff_max_secs: u32,
}

impl Default for SessionsConfig {
    fn default() -> Self {
        Self {
            escape_prefix: "!".to_string(),
            notify_disconnect: true,
            auto_reconnect: false,
            reconnect_max_attempts: 5,
            reconnect_backoff_max_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub timestamp_format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timestamp_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TransfersConfig {
    pub default_method: String,
    pub fallback_chain: Vec<String>,
}

impl Default for TransfersConfig {
    fn default() -> Self {
        Self {
            default_method: "sftp".to_string(),
            fallback_chain: vec![
                "sftp".to_string(),
                "scp".to_string(),
                "cat".to_string(),
                "base64".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TunnelsConfig {
    pub auto_restore: String,
}

impl Default for TunnelsConfig {
    fn default() -> Self {
        Self {
            auto_restore: "false".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MapConfig {
    pub enabled: bool,
    pub parse_ls: bool,
    pub parse_ps: bool,
    pub parse_netstat: bool,
    pub parse_id: bool,
    pub parse_uname: bool,
    pub parse_env: bool,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            parse_ls: true,
            parse_ps: true,
            parse_netstat: true,
            parse_id: true,
            parse_uname: true,
            parse_env: true,
        }
    }
}

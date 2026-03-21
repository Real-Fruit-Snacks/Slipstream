use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Unknown,
}

impl Default for EntryType {
    fn default() -> Self {
        EntryType::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedEntry {
    pub path: String,
    pub name: String,
    pub entry_type: EntryType,
    pub permissions: Option<String>,
    pub owner: Option<String>,
    pub size: Option<u64>,
}

impl ParsedEntry {
    pub fn is_suid(&self) -> bool {
        self.permissions
            .as_ref()
            .map(|p| p.contains('s'))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedUser {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home: String,
    pub shell: String,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

pub struct OutputParser;

impl OutputParser {
    pub fn detect_command(cmd: &str) -> Option<&'static str> {
        let cmd = cmd.trim();
        if cmd == "pwd" {
            return Some("pwd");
        }
        if cmd.starts_with("cd ") || cmd == "cd" {
            return Some("cd");
        }
        if cmd.starts_with("cat") && cmd.contains("/etc/passwd") {
            return Some("passwd");
        }
        if cmd.starts_with("find ") {
            return Some("find");
        }
        if cmd.starts_with("tree ") || cmd == "tree" {
            return Some("tree");
        }
        if cmd.starts_with("ls -l") || cmd.starts_with("ls -al") || cmd.starts_with("ls -la") {
            return Some("ls_la");
        }
        if cmd.starts_with("ls") {
            return Some("ls");
        }
        if cmd.starts_with("ip a") || cmd.starts_with("ifconfig") {
            return Some("network");
        }
        // Windows commands
        if cmd.starts_with("dir") { return Some("dir"); }
        if cmd == "net user" || cmd.starts_with("net user ") { return Some("net_user"); }
        if cmd.starts_with("ipconfig") { return Some("ipconfig"); }
        None
    }

    pub fn join_path(cwd: &str, name: &str, separator: char) -> String {
        if cwd.is_empty() { return name.to_string(); }
        if cwd.ends_with(separator) || cwd.ends_with('/') || cwd.ends_with('\\') {
            format!("{}{}", cwd, name)
        } else {
            format!("{}{}{}", cwd, separator, name)
        }
    }

    pub fn parse_ls(output: &str, cwd: &str) -> Vec<ParsedEntry> {
        output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|name| {
                let name = name.trim().to_string();
                let path = Self::join_path(cwd, &name, '/');
                ParsedEntry {
                    path,
                    name,
                    entry_type: EntryType::Unknown,
                    permissions: None,
                    owner: None,
                    size: None,
                }
            })
            .collect()
    }

    pub fn parse_ls_la(output: &str, cwd: &str) -> Vec<ParsedEntry> {
        let mut entries = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("total ") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 {
                continue;
            }
            let name = parts[8..].join(" ");
            if name == "." || name == ".." {
                continue;
            }

            let perms_raw = parts[0];
            let entry_type = match perms_raw.chars().next() {
                Some('d') => EntryType::Directory,
                Some('l') => EntryType::Symlink,
                Some('-') => EntryType::File,
                _ => EntryType::Unknown,
            };
            let perms = Some(perms_raw.to_string());

            let owner = Some(parts[2].to_string());
            let size = parts[4].parse::<u64>().ok();

            let path = Self::join_path(cwd, &name, '/');

            entries.push(ParsedEntry {
                path,
                name,
                entry_type,
                permissions: perms,
                owner,
                size,
            });
        }
        entries
    }

    pub fn parse_find(output: &str) -> Vec<ParsedEntry> {
        output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|path| {
                let path = path.trim().to_string();
                let name = path.rsplit('/').next().unwrap_or(&path).to_string();
                ParsedEntry {
                    path,
                    name,
                    entry_type: EntryType::Unknown,
                    permissions: None,
                    owner: None,
                    size: None,
                }
            })
            .collect()
    }

    pub fn parse_tree(output: &str, cwd: &str) -> Vec<ParsedEntry> {
        let mut entries = Vec::new();
        for line in output.lines() {
            // Strip tree drawing characters: ├── └── │ and leading spaces
            let stripped = line
                .replace("├── ", "")
                .replace("└── ", "")
                .replace("│   ", "")
                .replace("│", "")
                .replace("    ", "");
            let name = stripped.trim().to_string();
            if name.is_empty() {
                continue;
            }
            // Skip summary lines like "N directories, M files"
            if name.contains(" director") || name.contains(" file") {
                continue;
            }
            let path = Self::join_path(cwd, &name, '/');
            entries.push(ParsedEntry {
                path,
                name,
                entry_type: EntryType::Unknown,
                permissions: None,
                owner: None,
                size: None,
            });
        }
        entries
    }

    pub fn parse_ip_a(output: &str) -> Vec<NetworkInterface> {
        let mut interfaces: Vec<NetworkInterface> = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim();
            // Interface header line: "1: lo: <LOOPBACK,...>"
            if let Some(rest) = trimmed.split_once(": ") {
                // Check if the first part is a digit (interface index)
                if rest.0.chars().all(|c| c.is_ascii_digit()) {
                    // rest.1 is "ifname: <FLAGS...>" or similar
                    let iface_name = rest.1
                        .split(':')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !iface_name.is_empty() {
                        interfaces.push(NetworkInterface {
                            name: iface_name,
                            ipv4: None,
                            ipv6: None,
                        });
                        continue;
                    }
                }
            }
            // inet line: "    inet 192.168.1.1/24 brd ..."
            if trimmed.starts_with("inet ") && !trimmed.starts_with("inet6 ") {
                if let Some(iface) = interfaces.last_mut() {
                    let addr = trimmed
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .to_string();
                    if !addr.is_empty() {
                        iface.ipv4 = Some(addr);
                    }
                }
            }
            // inet6 line: "    inet6 fe80::1/64 scope link"
            if trimmed.starts_with("inet6 ") {
                if let Some(iface) = interfaces.last_mut() {
                    let addr = trimmed
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .to_string();
                    if !addr.is_empty() {
                        iface.ipv6 = Some(addr);
                    }
                }
            }
        }
        interfaces
    }

    pub fn parse_passwd(output: &str) -> Vec<ParsedUser> {
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 7 {
                    Some(ParsedUser {
                        username: parts[0].to_string(),
                        uid: parts[2].parse().unwrap_or(0),
                        gid: parts[3].parse().unwrap_or(0),
                        home: parts[5].to_string(),
                        shell: parts[6].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn strip_ansi(input: &str) -> String {
        let stripped = strip_ansi_escapes::strip(input);
        String::from_utf8_lossy(&stripped).to_string()
    }

    /// Parse Windows `dir` output
    pub fn parse_dir(output: &str, cwd: &str) -> Vec<ParsedEntry> {
        // Extract "Directory of X:\path" line as the authoritative cwd
        let mut effective_cwd = cwd.to_string();
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Directory of ") {
                effective_cwd = trimmed.strip_prefix("Directory of ").unwrap_or(cwd).trim().to_string();
                break;
            }
        }
        let cwd = effective_cwd.as_str();

        let mut entries = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            // Skip empty, header lines, summary lines
            if line.is_empty() || line.starts_with("Volume") || line.starts_with("Directory of")
                || line.contains("File(s)") || line.contains("Dir(s)")
                || line.contains("Serial Number") {
                continue;
            }
            // Format: "MM/DD/YYYY  HH:MM AM/PM    <DIR>          name"
            // or:     "MM/DD/YYYY  HH:MM AM/PM         123,456 name"
            let parts: Vec<&str> = line.splitn(4, char::is_whitespace).collect();
            if parts.len() < 4 { continue; }
            // Find the name - it's after the size or <DIR> marker
            let rest = line;
            let (entry_type, name) = if rest.contains("<DIR>") {
                if let Some(pos) = rest.find("<DIR>") {
                    let after = rest[pos + 5..].trim();
                    if after == "." || after == ".." { continue; }
                    (EntryType::Directory, after.to_string())
                } else { continue; }
            } else {
                // File entry - find the filename after the size
                // Match pattern: date time size name
                let cols: Vec<&str> = rest.split_whitespace().collect();
                if cols.len() < 4 { continue; }
                // Last element(s) are the filename, size is cols[3] or second-to-last
                // Better: everything after the 3rd whitespace group
                let name = cols[3..].join(" ");
                let size = cols.get(2).and_then(|s| s.replace(",", "").parse::<u64>().ok());
                let path = Self::join_path(cwd, &name, '\\');
                entries.push(ParsedEntry {
                    path, name, entry_type: EntryType::File,
                    permissions: None, owner: None, size,
                });
                continue;
            };
            let path = Self::join_path(cwd, &name, '\\');
            entries.push(ParsedEntry {
                path, name, entry_type, permissions: None, owner: None, size: None,
            });
        }
        entries
    }

    /// Parse Windows `net user` output
    pub fn parse_net_user(output: &str) -> Vec<ParsedUser> {
        let mut users = Vec::new();
        let mut past_separator = false;
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("---") { past_separator = true; continue; }
            if !past_separator { continue; }
            if line.starts_with("The command completed") || line.is_empty() { continue; }
            // Skip prompt lines that leak into output
            if line.contains('>') || line.contains('@') || line.contains('\\') { continue; }
            // Usernames are space-separated, up to 3 per line
            for name in line.split_whitespace() {
                if !name.is_empty() {
                    users.push(ParsedUser {
                        username: name.to_string(),
                        uid: 0, gid: 0,
                        home: String::new(), shell: String::new(),
                    });
                }
            }
        }
        users
    }

    /// Parse Windows `ipconfig` output
    pub fn parse_ipconfig(output: &str) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();
        let mut current_name = String::new();
        let mut current_ipv4 = None;
        for line in output.lines() {
            let trimmed = line.trim();
            // Adapter header: "Ethernet adapter Ethernet0:"
            if !line.starts_with(' ') && line.contains("adapter") && line.ends_with(':') {
                if !current_name.is_empty() {
                    interfaces.push(NetworkInterface {
                        name: current_name.clone(), ipv4: current_ipv4.take(), ipv6: None,
                    });
                }
                current_name = line.split("adapter").nth(1)
                    .unwrap_or("").trim().trim_end_matches(':').to_string();
            }
            // IPv4 line: "   IPv4 Address. . . . . . . . . . . : 192.168.1.100"
            if trimmed.starts_with("IPv4 Address") || trimmed.starts_with("IP Address") {
                if let Some(addr) = trimmed.rsplit(": ").next() {
                    current_ipv4 = Some(addr.trim().to_string());
                }
            }
        }
        if !current_name.is_empty() {
            interfaces.push(NetworkInterface {
                name: current_name, ipv4: current_ipv4, ipv6: None,
            });
        }
        interfaces
    }
}

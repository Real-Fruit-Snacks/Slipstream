use std::path::PathBuf;

use crate::commands::help;
use crate::config::Config;
use crate::input::line_buffer::LineBuffer;
use crate::input::mode::{ModeTracker, TerminalMode};
use crate::input::router::{CommandRouter, RouteResult};
use crate::logging::engine::LogEngine;
use crate::mapper::cwd::CwdTracker;
use crate::mapper::parser::OutputParser;
use crate::mapper::query::MapQuery;
use crate::mapper::store::MapStore;
use crate::session::manager::SessionManager;
use crate::transfer::fallback::{FallbackChain, TransferMethod};
use crate::target_os::TargetOS;
use crate::tunnel::manager::{Tunnel, TunnelManager, TunnelType};

pub struct PtyLoop {
    config: Config,
    session_manager: SessionManager,
    tunnel_manager: TunnelManager,
    router: CommandRouter,
    line_buffer: LineBuffer,
    mode_tracker: ModeTracker,
    control_path: Option<PathBuf>,
    ssh_user_host: String,
    log_engine: Option<LogEngine>,
    map_store: MapStore,
    cwd_tracker: CwdTracker,
    last_command: Option<String>,
    captured_fingerprint: Option<String>,
    child_pid: Option<nix::unistd::Pid>,
    target_os: TargetOS,
}

impl PtyLoop {
    pub fn new(config: Config, control_path: Option<PathBuf>, ssh_user_host: String) -> Self {
        let prefix = config.sessions.escape_prefix.clone();
        PtyLoop {
            config,
            session_manager: SessionManager::new(),
            tunnel_manager: TunnelManager::new(),
            router: CommandRouter::with_prefix(prefix),
            line_buffer: LineBuffer::new(),
            mode_tracker: ModeTracker::new(),
            control_path,
            ssh_user_host,
            log_engine: None,
            map_store: MapStore::new_empty(),
            cwd_tracker: CwdTracker::new(TargetOS::Unknown),
            last_command: None,
            captured_fingerprint: None,
            child_pid: None,
            target_os: TargetOS::Unknown,
        }
    }

    pub fn router(&self) -> &CommandRouter {
        &self.router
    }

    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    pub fn session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    pub fn tunnel_manager(&self) -> &TunnelManager {
        &self.tunnel_manager
    }

    pub fn tunnel_manager_mut(&mut self) -> &mut TunnelManager {
        &mut self.tunnel_manager
    }

    pub fn line_buffer(&self) -> &LineBuffer {
        &self.line_buffer
    }

    pub fn line_buffer_mut(&mut self) -> &mut LineBuffer {
        &mut self.line_buffer
    }

    pub fn mode_tracker(&self) -> &ModeTracker {
        &self.mode_tracker
    }

    pub fn mode_tracker_mut(&mut self) -> &mut ModeTracker {
        &mut self.mode_tracker
    }

    /// Set the captured host fingerprint for tunnel save/restore.
    pub fn set_fingerprint(&mut self, fp: String) {
        self.captured_fingerprint = Some(fp);
    }

    /// Set the child PID (SSH process).
    pub fn set_child_pid(&mut self, pid: nix::unistd::Pid) {
        self.child_pid = Some(pid);
    }

    pub fn target_os(&self) -> TargetOS {
        self.target_os
    }

    pub fn set_target_os(&mut self, os: TargetOS) {
        self.target_os = os;
        self.cwd_tracker.set_target_os(os);
    }

    /// Initialize the logging engine with a session directory path.
    pub fn init_logging(&mut self, session_dir: PathBuf) {
        self.log_engine = Some(LogEngine::new(session_dir, true));
    }

    /// Feed PTY output data to the log engine if initialized.
    pub fn log_output(&mut self, data: &str) {
        if let Some(ref mut engine) = self.log_engine {
            engine.append_continuous_output(data);
        }
        self.process_output_for_mapper(data);
    }

    /// Process PTY output through the filesystem mapper.
    fn process_output_for_mapper(&mut self, output: &str) {
        if !self.config.map.enabled {
            return;
        }
        let clean = OutputParser::strip_ansi(output);
        if let Some(ref cmd) = self.last_command.clone() {
            if let Some(cmd_type) = OutputParser::detect_command(cmd) {
                match cmd_type {
                    "ls" => {
                        let ls_path = if let Some(ref lc) = self.last_command.clone() {
                            let parts: Vec<&str> = lc.trim().splitn(2, char::is_whitespace).collect();
                            if parts.len() > 1 {
                                let arg = parts[1].trim();
                                if arg.starts_with('/') || (arg.len() >= 2 && arg.as_bytes()[1] == b':') {
                                    arg.to_string()
                                } else {
                                    self.cwd_tracker.current().to_string()
                                }
                            } else {
                                self.cwd_tracker.current().to_string()
                            }
                        } else {
                            self.cwd_tracker.current().to_string()
                        };
                        let entries = OutputParser::parse_ls(&clean, &ls_path);
                        self.map_store.add_entries(entries);
                    }
                    "ls_la" => {
                        let ls_path = if let Some(ref lc) = self.last_command.clone() {
                            let parts: Vec<&str> = lc.trim().splitn(2, char::is_whitespace).collect();
                            if parts.len() > 1 {
                                let arg = parts[1].trim();
                                if arg.starts_with('/') || (arg.len() >= 2 && arg.as_bytes()[1] == b':') {
                                    arg.to_string()
                                } else {
                                    self.cwd_tracker.current().to_string()
                                }
                            } else {
                                self.cwd_tracker.current().to_string()
                            }
                        } else {
                            self.cwd_tracker.current().to_string()
                        };
                        let entries = OutputParser::parse_ls_la(&clean, &ls_path);
                        self.map_store.add_entries(entries);
                    }
                    "find" => {
                        let entries = OutputParser::parse_find(&clean);
                        self.map_store.add_entries(entries);
                    }
                    "pwd" => {
                        self.cwd_tracker.update_from_pwd(&clean);
                    }
                    "cd" => {
                        self.cwd_tracker.update_from_cd(cmd);
                    }
                    "passwd" => {
                        let users = OutputParser::parse_passwd(&clean);
                        self.map_store.add_users(users);
                    }
                    "tree" => {
                        let entries = OutputParser::parse_tree(&clean, self.cwd_tracker.current());
                        self.map_store.add_entries(entries);
                    }
                    "dir" => {
                        let dir_path = if let Some(ref lc) = self.last_command.clone() {
                            let parts: Vec<&str> = lc.trim().splitn(2, char::is_whitespace).collect();
                            if parts.len() > 1 {
                                parts[1].trim().to_string()
                            } else {
                                self.cwd_tracker.current().to_string()
                            }
                        } else {
                            self.cwd_tracker.current().to_string()
                        };
                        let entries = OutputParser::parse_dir(&clean, &dir_path);
                        self.map_store.add_entries(entries);
                    }
                    "net_user" => {
                        let users = OutputParser::parse_net_user(&clean);
                        self.map_store.add_users(users);
                    }
                    "ipconfig" => {
                        // Store network interfaces — for now just log it
                        // NetworkInterface data could be stored separately
                    }
                    _ => {}
                }
            }
        }
    }

    /// Log a command being sent to SSH.
    pub fn log_command(&mut self, cmd: &str) {
        self.last_command = Some(cmd.to_string());
        if let Some(ref mut engine) = self.log_engine {
            engine.start_command(cmd);
        }
    }

    /// Log a slipstream event (e.g. ! command execution).
    pub fn log_event(&mut self, event: &str) {
        if let Some(ref mut engine) = self.log_engine {
            engine.log_event(event);
        }
    }

    /// Dispatch a recognised command and return the response text.
    pub fn handle_command(&mut self, command: &str, args: &str) -> String {
        // Global --help flag on any command
        if args.split_whitespace().any(|w| w == "--help") {
            return help::command_help(command);
        }

        match command {
            "help" | "?" => {
                if args.trim().is_empty() {
                    help::general_help()
                } else {
                    help::command_help(args.trim())
                }
            }

            "sessions" => {
                let list = self.session_manager.format_list();
                if list.is_empty() {
                    "No active sessions\n".to_string()
                } else {
                    list + "\n"
                }
            }

            "switch" => {
                let id_str = args.trim();
                match id_str.parse::<u32>() {
                    Ok(id) => {
                        if self.session_manager.switch_to(id) {
                            format!("Switched to session #{}\n", id)
                        } else {
                            format!("No session with id #{}\n", id)
                        }
                    }
                    Err(_) => format!("Invalid session id: '{}'\n", id_str),
                }
            }

            "kill" => {
                let id_str = args.trim();
                match id_str.parse::<u32>() {
                    Ok(id) => {
                        let tunnels_removed = self.tunnel_manager.delete_by_session(id);
                        if self.session_manager.kill(id) {
                            format!(
                                "Session #{} killed ({} tunnel(s) removed)\n",
                                id, tunnels_removed
                            )
                        } else {
                            format!("No session with id #{}\n", id)
                        }
                    }
                    Err(_) => format!("Invalid session id: '{}'\n", id_str),
                }
            }

            "rename" => {
                let mut parts = args.splitn(2, ' ');
                let id_str = parts.next().unwrap_or("").trim();
                let label = parts.next().unwrap_or("").trim();
                match id_str.parse::<u32>() {
                    Ok(id) => {
                        if label.is_empty() {
                            "Usage: rename <id> <label>\n".to_string()
                        } else if self.session_manager.rename(id, label.to_string()) {
                            format!("Session #{} renamed to '{}'\n", id, label)
                        } else {
                            format!("No session with id #{}\n", id)
                        }
                    }
                    Err(_) => format!("Invalid session id: '{}'\n", id_str),
                }
            }

            "tunnel" => self.handle_tunnel_command(args),

            "map" => self.handle_map_command(args),

            "upload" => self.execute_transfer(args, true),

            "download" => self.execute_transfer(args, false),

            "transfer-method" => {
                let method = args.trim();
                if method.is_empty() {
                    format!(
                        "Current transfer method: {}\n",
                        self.config.transfers.default_method
                    )
                } else {
                    self.config.transfers.default_method = method.to_string();
                    format!("Transfer method set to: {}\n", method)
                }
            }

            "bg" => {
                "Slipstream runs a single session per process.\n\
                 To background: press Ctrl+Z to suspend, or use tmux/screen.\n\
                 Use !connect to open a new session in a tmux window.\n".to_string()
            }

            "loot" => self.handle_loot_command(args),

            "note" => self.handle_note_command(args),

            "exec" => self.handle_exec_command(args),

            "connect" => {
                if args.trim().is_empty() {
                    return "Usage: !connect [ssh-flags] user@host\n".to_string();
                }

                let slipstream_bin = std::env::current_exe()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "slipstream".to_string());

                // Try tmux first
                let tmux_check = std::process::Command::new("tmux")
                    .args(["list-sessions"])
                    .output();

                if tmux_check.is_ok() {
                    let window_name = format!(
                        "slip-{}",
                        args.trim().split('@').last().unwrap_or("target")
                    );
                    let cmd = format!("{} ssh {}", slipstream_bin, args.trim());
                    match std::process::Command::new("tmux")
                        .args(["new-window", "-n", &window_name, &cmd])
                        .output()
                    {
                        Ok(output) if output.status.success() => {
                            format!("Opened new tmux window: {}\n", window_name)
                        }
                        _ => {
                            format!(
                                "Failed to open tmux window. Run manually:\n  {} ssh {}\n",
                                slipstream_bin,
                                args.trim()
                            )
                        }
                    }
                } else {
                    format!(
                        "tmux not detected. Run in a new terminal:\n  {} ssh {}\n",
                        slipstream_bin,
                        args.trim()
                    )
                }
            }

            _ => format!("Unknown command: '{}'\n", command),
        }
    }

    /// Execute a file transfer (upload or download) via the control socket.
    fn execute_transfer(&self, args: &str, is_upload: bool) -> String {
        let cp = match self.control_path.as_ref() {
            Some(p) => p,
            None => return "No control socket available\n".to_string(),
        };
        let cp_str = match cp.to_str() {
            Some(s) => s,
            None => return "Invalid control socket path\n".to_string(),
        };

        // Parse args: optional --method <m>, then source, then destination
        let mut words: Vec<&str> = args.split_whitespace().collect();
        let mut method_override: Option<TransferMethod> = None;

        if words.len() >= 2 && words[0] == "--method" {
            if let Some(m) = TransferMethod::from_str(words[1]) {
                method_override = Some(m);
                words = words[2..].to_vec();
            } else {
                return format!("Unknown transfer method: '{}'\n", words[1]);
            }
        }

        if words.len() < 2 {
            return format!(
                "Usage: {} [--method <method>] <source> <destination>\n",
                if is_upload { "upload" } else { "download" }
            );
        }

        let source = words[0];
        let destination = words[1];

        // Build the fallback chain
        let methods: Vec<TransferMethod> = if let Some(m) = method_override {
            vec![m]
        } else {
            let chain_strs: Vec<&str> = self
                .config
                .transfers
                .fallback_chain
                .iter()
                .map(|s| s.as_str())
                .collect();
            let chain = FallbackChain::from_strings(&chain_strs);
            chain.methods().to_vec()
        };

        for method in &methods {
            let cmd_string = if is_upload {
                if self.target_os.is_windows() {
                    method.upload_command_windows(cp_str, &self.ssh_user_host, source, destination)
                } else {
                    method.upload_command(cp_str, &self.ssh_user_host, source, destination)
                }
            } else {
                if self.target_os.is_windows() {
                    method.download_command_windows(cp_str, &self.ssh_user_host, source, destination)
                } else {
                    method.download_command(cp_str, &self.ssh_user_host, source, destination)
                }
            };

            // For downloads, use status() so shell redirection (> file) works.
            // For uploads, use output() to capture errors.
            let cmd_result = if is_upload {
                std::process::Command::new("sh")
                    .args(["-c", &cmd_string])
                    .output()
                    .map(|o| o.status.success())
            } else {
                std::process::Command::new("sh")
                    .args(["-c", &cmd_string])
                    .status()
                    .map(|s| s.success())
            };

            match cmd_result {
                Ok(true) => {
                    // Get file size from local filesystem
                    let size_str = if is_upload {
                        std::fs::metadata(source)
                            .ok()
                            .map(|m| format_bytes(m.len()))
                            .unwrap_or_default()
                    } else {
                        std::fs::metadata(destination)
                            .ok()
                            .map(|m| format_bytes(m.len()))
                            .unwrap_or_default()
                    };
                    let direction = if is_upload { "uploaded" } else { "downloaded" };
                    let arrow = if is_upload {
                        format!("{} → {}", source, destination)
                    } else {
                        format!("{} → {}", source, destination)
                    };
                    if size_str.is_empty() {
                        return format!(
                            "\u{2713} {} via {} ({})\n",
                            direction,
                            method.name(),
                            arrow
                        );
                    }
                    return format!(
                        "\u{2713} {} via {} ({}, {})\n",
                        direction,
                        method.name(),
                        arrow,
                        size_str
                    );
                }
                Ok(_) => continue, // non-zero exit, try next method
                Err(_) => continue, // command failed to run, try next method
            }
        }

        format!(
            "Transfer failed: all methods exhausted ({})\n",
            methods
                .iter()
                .map(|m| m.name())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    /// Handle the `map` command dispatch.
    fn handle_map_command(&mut self, args: &str) -> String {
        let args = args.trim();
        if args.is_empty() {
            let tree = MapQuery::format_tree(&self.map_store);
            if tree.is_empty() {
                return "No entries mapped yet. Run ls, find, or tree on the remote host.\n"
                    .to_string();
            }
            return tree + "\n";
        }

        let mut parts = args.splitn(2, ' ');
        let subcmd = parts.next().unwrap_or("");
        let rest = parts.next().unwrap_or("").trim();

        match subcmd {
            "find" => {
                if rest.is_empty() {
                    return "Usage: map find <pattern>\n".to_string();
                }
                let results = MapQuery::find(&self.map_store, rest);
                if results.is_empty() {
                    return format!("No entries matching '{}'\n", rest);
                }
                let lines: Vec<String> = results.iter().map(|e| e.path.clone()).collect();
                lines.join("\n") + "\n"
            }
            "users" => {
                let users = self.map_store.users();
                if users.is_empty() {
                    return if self.target_os.is_windows() {
                        "No users captured yet. Run: net user\n".to_string()
                    } else {
                        "No users captured yet. Run: cat /etc/passwd\n".to_string()
                    };
                }
                let lines: Vec<String> = users
                    .iter()
                    .map(|u| {
                        format!(
                            "{}  uid={} gid={} home={} shell={}",
                            u.username, u.uid, u.gid, u.home, u.shell
                        )
                    })
                    .collect();
                lines.join("\n") + "\n"
            }
            "coverage" => MapQuery::coverage(&self.map_store) + "\n",
            "export" => MapQuery::export_json(&self.map_store) + "\n",
            "reset" => {
                self.map_store.reset();
                "Map data reset\n".to_string()
            }
            _ => {
                // Treat as a path: !map /some/path or !map C:\some\path
                let is_path = subcmd.starts_with('/')
                    || (subcmd.len() >= 2 && subcmd.as_bytes()[1] == b':');
                if is_path {
                    let entries = MapQuery::list_directory(&self.map_store, subcmd);
                    if entries.is_empty() {
                        return format!("No entries under '{}'\n", subcmd);
                    }
                    let lines: Vec<String> = entries
                        .iter()
                        .map(|e| {
                            let perms = e
                                .permissions
                                .as_deref()
                                .unwrap_or("----------");
                            let owner = e.owner.as_deref().unwrap_or("?");
                            let size = e
                                .size
                                .map(|s| format_bytes(s))
                                .unwrap_or_else(|| "-".to_string());
                            format!("{} {} {} {}", perms, owner, size, e.name)
                        })
                        .collect();
                    lines.join("\n") + "\n"
                } else {
                    format!("Unknown map subcommand: '{}'\n", subcmd)
                }
            }
        }
    }

    /// Execute an SSH control socket forward command for a tunnel.
    fn execute_tunnel_forward(&self, tunnel: &Tunnel) -> Result<std::process::Output, String> {
        let cp = self
            .control_path
            .as_ref()
            .ok_or_else(|| "No control socket available".to_string())?;
        let cp_str = cp.to_str().unwrap_or("");

        match tunnel.tunnel_type {
            TunnelType::Local => std::process::Command::new("ssh")
                .args([
                    "-S", cp_str, "-O", "forward", "-L",
                    &tunnel.to_ssh_forward_arg(),
                    &self.ssh_user_host,
                ])
                .output()
                .map_err(|e| e.to_string()),
            TunnelType::Socks => std::process::Command::new("ssh")
                .args([
                    "-S", cp_str, "-O", "forward", "-D",
                    &tunnel.to_ssh_dynamic_arg(),
                    &self.ssh_user_host,
                ])
                .output()
                .map_err(|e| e.to_string()),
            TunnelType::Reverse => std::process::Command::new("ssh")
                .args([
                    "-S", cp_str, "-O", "forward", "-R",
                    &tunnel.to_ssh_reverse_arg(),
                    &self.ssh_user_host,
                ])
                .output()
                .map_err(|e| e.to_string()),
        }
    }

    /// Execute an SSH control socket cancel command to remove a tunnel.
    fn execute_tunnel_cancel(&self, tunnel: &Tunnel) -> Result<std::process::Output, String> {
        let cp = self
            .control_path
            .as_ref()
            .ok_or_else(|| "No control socket available".to_string())?;
        let cp_str = cp.to_str().unwrap_or("");

        let (flag, arg) = match tunnel.tunnel_type {
            TunnelType::Local => ("-L", tunnel.to_ssh_forward_arg()),
            TunnelType::Socks => ("-D", tunnel.to_ssh_dynamic_arg()),
            TunnelType::Reverse => ("-R", tunnel.to_ssh_reverse_arg()),
        };

        std::process::Command::new("ssh")
            .args([
                "-S", cp_str, "-O", "cancel", flag, &arg, &self.ssh_user_host,
            ])
            .output()
            .map_err(|e| e.to_string())
    }

    /// Handle the `tunnel` sub-command dispatch.
    pub fn handle_tunnel_command(&mut self, args: &str) -> String {
        let mut parts = args.splitn(2, ' ');
        let subcmd = parts.next().unwrap_or("").trim();
        let rest = parts.next().unwrap_or("").trim();

        match subcmd {
            "add" => {
                let session_id = self.session_manager.active_id().unwrap_or(0);
                match Tunnel::parse_add_args(rest, session_id) {
                    Ok(tunnel) => {
                        let id = self.tunnel_manager.add(tunnel);
                        // Execute the actual SSH forward via control socket
                        let tunnel_clone = match self.tunnel_manager.get(id) {
                            Some(t) => t.clone(),
                            None => return format!("Internal error: tunnel #{} not found\n", id),
                        };
                        match self.execute_tunnel_forward(&tunnel_clone) {
                            Ok(output) => {
                                if output.status.success() {
                                    self.log_event(&format!("tunnel #{} added and forwarded", id));
                                    format!("Tunnel #{} added\n", id)
                                } else {
                                    let stderr =
                                        String::from_utf8_lossy(&output.stderr).to_string();
                                    self.log_event(&format!(
                                        "tunnel #{} forward failed: {}",
                                        id,
                                        stderr.trim()
                                    ));
                                    self.tunnel_manager.delete(id);
                                    format!(
                                        "Tunnel add failed: {}\n",
                                        stderr.trim()
                                    )
                                }
                            }
                            Err(e) => {
                                self.log_event(&format!(
                                    "tunnel #{} forward error: {}",
                                    id, e
                                ));
                                self.tunnel_manager.delete(id);
                                format!("Tunnel add failed: {}\n", e)
                            }
                        }
                    }
                    Err(e) => format!("tunnel add error: {}\n", e),
                }
            }

            "list" => {
                let verbose = rest.split_whitespace().any(|w| w == "-v");
                let tunnels = self.tunnel_manager.list();
                if tunnels.is_empty() {
                    return "No tunnels configured\n".to_string();
                }
                let mut lines = Vec::new();
                for (id, t) in tunnels {
                    let type_str = match t.tunnel_type {
                        TunnelType::Local => "local",
                        TunnelType::Socks => "socks",
                        TunnelType::Reverse => "reverse",
                    };
                    if verbose {
                        let dest = match (&t.dest_host, t.dest_port) {
                            (Some(h), Some(p)) => format!("{}:{}", h, p),
                            (Some(h), None) => h.clone(),
                            _ => String::new(),
                        };
                        lines.push(format!(
                            "#{} type={} src={} dest={} session=#{}",
                            id, type_str, t.source_port, dest, t.session_id
                        ));
                    } else {
                        lines.push(format!("#{} {} :{}", id, type_str, t.source_port));
                    }
                }
                lines.join("\n") + "\n"
            }

            "del" => {
                let arg = rest.trim();
                if let Some(session_str) = arg.strip_prefix("--session ") {
                    match session_str.trim().parse::<u32>() {
                        Ok(sid) => {
                            // Cancel all tunnels for this session before removing
                            let tunnels_to_cancel: Vec<Tunnel> = self
                                .tunnel_manager
                                .list()
                                .iter()
                                .filter(|(_, t)| t.session_id == sid)
                                .map(|(_, t)| t.clone())
                                .collect();
                            for t in &tunnels_to_cancel {
                                let _ = self.execute_tunnel_cancel(t);
                            }
                            let n = self.tunnel_manager.delete_by_session(sid);
                            self.log_event(&format!(
                                "removed {} tunnel(s) for session #{}",
                                n, sid
                            ));
                            format!("Removed {} tunnel(s) for session #{}\n", n, sid)
                        }
                        Err(_) => format!("Invalid session id: '{}'\n", session_str.trim()),
                    }
                } else {
                    match arg.parse::<u32>() {
                        Ok(id) => {
                            // Cancel the tunnel via control socket before removing
                            if let Some(tunnel) = self.tunnel_manager.get(id) {
                                let tunnel_clone = tunnel.clone();
                                let _ = self.execute_tunnel_cancel(&tunnel_clone);
                            }
                            if self.tunnel_manager.delete(id) {
                                self.log_event(&format!("tunnel #{} removed", id));
                                format!("Tunnel #{} removed\n", id)
                            } else {
                                format!("No tunnel with id #{}\n", id)
                            }
                        }
                        Err(_) => format!("Invalid tunnel id: '{}'\n", arg),
                    }
                }
            }

            "flush" => {
                // Cancel all tunnels before flushing
                let all_tunnels: Vec<Tunnel> = self
                    .tunnel_manager
                    .list()
                    .iter()
                    .map(|(_, t)| t.clone())
                    .collect();
                for t in &all_tunnels {
                    let _ = self.execute_tunnel_cancel(t);
                }
                self.tunnel_manager.flush();
                self.log_event("all tunnels flushed");
                "All tunnels flushed\n".to_string()
            }

            "save" => {
                if let Some(ref fp) = self.captured_fingerprint {
                    let targets_dir = dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                        .join(".config/slipstream/targets");
                    let storage = crate::target::storage::TargetStorage::new(targets_dir);
                    let session_id = self.session_manager.active_id().unwrap_or(0);
                    let saved = self.tunnel_manager.export_as_saved(session_id);

                    match storage.load_target(fp) {
                        Ok(mut target) => {
                            target.saved_tunnels = saved.clone();
                            match storage.save_target(&target) {
                                Ok(_) => format!("Saved {} tunnel(s) to target config\n", saved.len()),
                                Err(e) => format!("Failed to save tunnels: {}\n", e),
                            }
                        }
                        Err(_) => "No target data found — connect first to establish target identity\n".to_string(),
                    }
                } else {
                    "No fingerprint captured — cannot save tunnel config\n".to_string()
                }
            }

            "restore" => {
                if let Some(ref fp) = self.captured_fingerprint {
                    let targets_dir = dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                        .join(".config/slipstream/targets");
                    let storage = crate::target::storage::TargetStorage::new(targets_dir);
                    let session_id = self.session_manager.active_id().unwrap_or(0);

                    match storage.load_target(fp) {
                        Ok(target) => {
                            if target.saved_tunnels.is_empty() {
                                return "No saved tunnels for this target\n".to_string();
                            }
                            let tunnels = crate::tunnel::manager::TunnelManager::import_from_saved(
                                &target.saved_tunnels, session_id
                            );
                            let mut results = Vec::new();
                            for tunnel in tunnels {
                                let id = self.tunnel_manager.add(tunnel);
                                let t = match self.tunnel_manager.get(id) {
                                    Some(t) => t.clone(),
                                    None => {
                                        results.push(format!("Tunnel #{}: internal error — not found after add", id));
                                        continue;
                                    }
                                };
                                let exec_result = self.execute_tunnel_forward(&t);
                                results.push(format!("Tunnel #{}: {}", id,
                                    match exec_result {
                                        Ok(output) if output.status.success() => "forwarded".to_string(),
                                        Ok(output) => format!("forward failed: {}",
                                            String::from_utf8_lossy(&output.stderr).trim()),
                                        Err(e) => format!("error: {}", e),
                                    }
                                ));
                            }
                            results.join("\n") + "\n"
                        }
                        Err(_) => "No target data found\n".to_string(),
                    }
                } else {
                    "No fingerprint captured — cannot restore tunnel config\n".to_string()
                }
            }

            "" => help::command_help("tunnel"),

            _ => format!("Unknown tunnel subcommand: '{}'\n", subcmd),
        }
    }

    fn handle_loot_command(&self, args: &str) -> String {
        let cp = match self.control_path.as_ref() {
            Some(p) => p.to_str().unwrap_or(""),
            None => return "No control socket available\n".to_string(),
        };

        let loot_dir = args.trim();
        let loot_dir = if loot_dir.is_empty() { "./loot" } else { loot_dir };

        // Create loot directory
        std::fs::create_dir_all(loot_dir).ok();

        let files = if self.target_os.is_windows() {
            vec![
                ("C:/Users", "users_dir.txt", "dir C:\\Users"),
                ("C:/Windows/System32/config", "config_dir.txt", "dir C:\\Windows\\System32\\config"),
                ("", "systeminfo.txt", "systeminfo"),
                ("", "whoami_priv.txt", "whoami /priv"),
                ("", "whoami_groups.txt", "whoami /groups"),
                ("", "netstat.txt", "netstat -ano"),
                ("", "net_users.txt", "net user"),
                ("", "net_localgroup_admin.txt", "net localgroup Administrators"),
                ("", "ipconfig.txt", "ipconfig /all"),
                ("", "arp.txt", "arp -a"),
                ("", "tasklist.txt", "tasklist /v"),
                ("", "installed_software.txt", "reg query HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall /s"),
            ]
        } else {
            vec![
                ("/etc/passwd", "passwd.txt", "cat /etc/passwd"),
                ("/etc/shadow", "shadow.txt", "cat /etc/shadow"),
                ("/etc/hosts", "hosts.txt", "cat /etc/hosts"),
                ("/etc/crontab", "crontab.txt", "cat /etc/crontab"),
                ("", "id.txt", "id"),
                ("", "whoami.txt", "whoami"),
                ("", "uname.txt", "uname -a"),
                ("", "ifconfig.txt", "ip a"),
                ("", "netstat.txt", "ss -tlnp"),
                ("", "processes.txt", "ps aux"),
                ("", "suid.txt", "find / -perm -4000 -type f 2>/dev/null"),
                ("", "sudo_l.txt", "sudo -l 2>/dev/null"),
            ]
        };

        let mut results = Vec::new();
        for (_path, filename, cmd) in &files {
            let output_path = format!("{}/{}", loot_dir, filename);
            let ssh_cmd = format!(
                "timeout 30 ssh -S {} {} \"{}\" > {} 2>/dev/null",
                cp, self.ssh_user_host, cmd, output_path
            );
            match std::process::Command::new("sh")
                .args(["-c", &ssh_cmd])
                .status()
            {
                Ok(s) if s.success() => {
                    let size = std::fs::metadata(&output_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    if size > 0 {
                        results.push(format!("  \u{2713} {} ({}B)", filename, size));
                    } else {
                        results.push(format!("  \u{2717} {} (empty/denied)", filename));
                    }
                }
                _ => results.push(format!("  \u{2717} {} (failed)", filename)),
            }
        }

        format!("Loot saved to {}:\n{}\n", loot_dir, results.join("\n"))
    }

    fn handle_note_command(&self, args: &str) -> String {
        let note = args.trim();
        if note.is_empty() {
            // Show existing notes
            if let Some(ref fp) = self.captured_fingerprint {
                let targets_dir = dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                    .join(".config/slipstream/targets");
                let notes_path = targets_dir
                    .join(fp.replace([':', '/', '\\', '+'], "-"))
                    .join("notes.txt");
                if notes_path.exists() {
                    match std::fs::read_to_string(&notes_path) {
                        Ok(content) => return format!("Notes:\n{}\n", content),
                        Err(_) => return "Could not read notes\n".to_string(),
                    }
                }
                return "No notes for this target\n".to_string();
            }
            return "No target identified yet\n".to_string();
        }

        // Append note
        if let Some(ref fp) = self.captured_fingerprint {
            let targets_dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                .join(".config/slipstream/targets");
            let notes_path = targets_dir
                .join(fp.replace([':', '/', '\\', '+'], "-"))
                .join("notes.txt");
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
            let entry = format!("[{}] {}\n", timestamp, note);
            use std::io::Write;
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&notes_path)
            {
                Ok(mut f) => {
                    f.write_all(entry.as_bytes()).ok();
                    format!("Note saved: {}\n", note)
                }
                Err(e) => format!("Failed to save note: {}\n", e),
            }
        } else {
            "No target identified yet\n".to_string()
        }
    }

    fn handle_exec_command(&self, args: &str) -> String {
        let cmd = args.trim();
        if cmd.is_empty() {
            return "Usage: !exec <command>\nRuns command via SSH control socket (not through PTY)\n".to_string();
        }

        let cp = match self.control_path.as_ref() {
            Some(p) => p.to_str().unwrap_or(""),
            None => return "No control socket available\n".to_string(),
        };

        let ssh_cmd = format!("ssh -S {} {} \"{}\"", cp, self.ssh_user_host, cmd);
        match std::process::Command::new("sh")
            .args(["-c", &ssh_cmd])
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    result.push_str(&format!("STDERR: {}", stderr));
                }
                if result.is_empty() {
                    result = "(no output)\n".to_string();
                }
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result
            }
            Err(e) => format!("exec failed: {}\n", e),
        }
    }

    /// Route a raw input line and, if it is a Slipstream command, dispatch it.
    /// Returns Some(response) for commands, None for passthrough.
    pub fn process_line(&mut self, line: &str) -> Option<String> {
        match self.router.route(line) {
            RouteResult::SlipstreamCommand { command, args } => {
                Some(self.handle_command(&command, &args))
            }
            RouteResult::Passthrough => None,
        }
    }

    /// Current terminal mode.
    pub fn terminal_mode(&self) -> TerminalMode {
        self.mode_tracker.current()
    }

    /// Get the configured escape prefix.
    pub fn escape_prefix(&self) -> &str {
        &self.config.sessions.escape_prefix
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{}KB", bytes / 1024)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Return the general help text listing all available commands.
pub fn general_help() -> String {
    let lines = vec![
        "Slipstream commands (prefix with escape sequence, default: !):",
        "",
        "  sessions              List active sessions",
        "  switch <id>           Switch to session by id",
        "  kill <id>             Kill session (and its tunnels)",
        "  rename <id> <label>   Rename a session",
        "  connect <target>      Connect to a target",
        "  bg                    Background the current session",
        "",
        "  tunnel add   --type <local|socks|reverse> -s <port> [-d <host>] [-p <port>]",
        "  tunnel list  [-v]",
        "  tunnel del   <id>|--session <session_id>",
        "  tunnel flush          Remove all tunnels",
        "  tunnel save           Save tunnel config",
        "  tunnel restore        Restore saved tunnels",
        "",
        "  upload <remote>       Upload a file",
        "  download <remote>     Download a file",
        "  transfer-method [m]   Show or set transfer method",
        "",
        "  map                   Show filesystem map",
        "  help [cmd]            Show help (optionally for a specific command)",
        "  ?                     Alias for help",
        "",
        "  exec <cmd>            Run command via control socket",
        "  loot [dir]            Auto-grab common recon files",
        "  note [text]           View or add target notes",
    ];
    lines.join("\n") + "\n"
}

/// Return help text for a specific command, or general help if unknown.
pub fn command_help(cmd: &str) -> String {
    let cmd = cmd.trim();
    match cmd {
        "tunnel" => {
            [
                "tunnel - Manage SSH port-forwarding tunnels",
                "",
                "  tunnel add --type <local|socks|reverse> -s <src_port> [-d <dest_host>] [-p <dest_port>]",
                "  tunnel list [-v]",
                "  tunnel del <id>",
                "  tunnel del --session <session_id>",
                "  tunnel flush",
                "  tunnel save",
                "  tunnel restore",
            ]
            .join("\n")
                + "\n"
        }
        "sessions" => "sessions - List all active sessions\n".to_string(),
        "switch" => "switch <id> - Switch active session to the given id\n".to_string(),
        "kill" => "kill <id> - Kill a session and remove its tunnels\n".to_string(),
        "rename" => "rename <id> <label> - Assign a human-readable label to a session\n".to_string(),
        "connect" => "connect <target> - Connect to a new target\n".to_string(),
        "bg" => "bg - Background the current session\n".to_string(),
        "upload" => "upload <remote_path> - Upload a file to the remote host\n".to_string(),
        "download" => "download <remote_path> - Download a file from the remote host\n".to_string(),
        "transfer-method" => {
            "transfer-method [method] - Show or set the active file transfer method\n".to_string()
        }
        "map" => "map - Display the filesystem/process map for the active session\n".to_string(),
        "loot" => {
            [
                "loot [dir] - Auto-grab common recon files from the remote host",
                "",
                "  Usage: !loot [output_dir]",
                "  Collects common enumeration data (passwd, shadow, id, netstat, etc.)",
                "  and saves each to a file in the specified directory (default: ./loot).",
                "  Adapts collected files based on detected OS (Linux vs Windows).",
            ]
            .join("\n")
                + "\n"
        }
        "note" => {
            [
                "note [text] - View or add notes for the current target",
                "",
                "  Usage: !note           - Show saved notes for this target",
                "         !note <text>    - Append a timestamped note",
                "  Notes are stored per-target in ~/.config/slipstream/targets/<fp>/notes.txt",
            ]
            .join("\n")
                + "\n"
        }
        "exec" => {
            [
                "exec <command> - Run a command via the SSH control socket",
                "",
                "  Usage: !exec <command>",
                "  Executes the command out-of-band (not through the PTY).",
                "  Output is captured and returned directly to the Slipstream prompt.",
            ]
            .join("\n")
                + "\n"
        }
        "help" | "?" | "" => general_help(),
        _ => format!("No help available for '{}'\n", cmd),
    }
}

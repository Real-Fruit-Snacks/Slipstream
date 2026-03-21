/// The set of commands that Slipstream recognises.
static KNOWN_COMMANDS: &[&str] = &[
    "tunnel",
    "sessions",
    "switch",
    "connect",
    "bg",
    "kill",
    "rename",
    "upload",
    "download",
    "transfer-method",
    "map",
    "help",
    "?",
    "loot",
    "note",
    "exec",
];

/// Result of routing an input line.
#[derive(Debug, PartialEq)]
pub enum RouteResult {
    /// A recognised Slipstream command.
    SlipstreamCommand { command: String, args: String },
    /// The line should be forwarded to the remote shell unchanged.
    Passthrough,
}

/// Routes input lines to Slipstream commands or the remote shell.
pub struct CommandRouter {
    prefix: String,
}

impl CommandRouter {
    /// Create a router with the default prefix `"!"`.
    pub fn new() -> Self {
        CommandRouter {
            prefix: "!".to_string(),
        }
    }

    /// Create a router with a custom prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        CommandRouter {
            prefix: prefix.into(),
        }
    }

    /// Route a single input line.
    pub fn route(&self, line: &str) -> RouteResult {
        // Must start with the prefix to be considered a command.
        if !line.starts_with(&*self.prefix) {
            return RouteResult::Passthrough;
        }

        // Strip the prefix.
        let rest = &line[self.prefix.len()..];

        // Bare prefix (e.g. "!") → passthrough.
        if rest.is_empty() {
            return RouteResult::Passthrough;
        }

        // Special alias: prefix + "?" is equivalent to "?" command.
        if rest == "?" {
            return RouteResult::SlipstreamCommand {
                command: "?".to_string(),
                args: "".to_string(),
            };
        }

        // Extract the first word as the command name.
        let mut parts = rest.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("").trim();
        let args = parts.next().unwrap_or("").trim().to_string();

        if KNOWN_COMMANDS.contains(&cmd) {
            RouteResult::SlipstreamCommand {
                command: cmd.to_string(),
                args,
            }
        } else {
            // Unknown → passthrough (preserves bash !!, !$, etc.)
            RouteResult::Passthrough
        }
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

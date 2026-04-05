<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/Real-Fruit-Snacks/Slipstream/main/docs/assets/logo-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/Real-Fruit-Snacks/Slipstream/main/docs/assets/logo-light.svg">
  <img alt="Slipstream" src="https://raw.githubusercontent.com/Real-Fruit-Snacks/Slipstream/main/docs/assets/logo-dark.svg" width="520">
</picture>

![Rust](https://img.shields.io/badge/language-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20targets-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**SSH wrapper for red team operations — drop-in replacement with tunnel management, file transfers, passive filesystem mapping, and per-command session logging**

Wraps the system's real `ssh` binary via PTY, intercepting `!` commands to provide session management, tunnel management, file transfers, passive filesystem mapping, and per-command session logging — while passing all SSH functionality through unchanged. Targets identified by SSH host key fingerprint. Works against both Linux and Windows targets.

> **Authorization Required**: This tool is designed exclusively for authorized security testing with explicit written permission. Unauthorized access to computer systems is illegal and may result in criminal prosecution.

[Quick Start](#quick-start) • [Commands](#commands) • [Architecture](#architecture) • [Internals](#internals) • [Configuration](#configuration) • [Security](#security)

</div>

---

## Highlights

<table>
<tr>
<td width="50%">

**Drop-In SSH Replacement**
All SSH flags, `-o` options, and `~/.ssh/config` work as normal. Slipstream parses what it needs (host, user, port) and passes everything else unchanged to the real `ssh` binary. Your workflow doesn't change.

**Tunnel Management**
iptables-style syntax for SSH tunnels: `!tunnel add --type socks -p 1080`. Real forwarding via `ssh -O forward` over the master socket. Add, delete, list, flush, save, and restore tunnel configurations per target.

**File Transfers**
`!upload` and `!download` with an automatic fallback chain: SFTP, SCP, cat-over-SSH, base64. Windows paths with backslashes are handled transparently via forward-slash conversion. No separate SCP session needed.

</td>
<td width="50%">

**Passive Filesystem Mapper**
Slipstream watches your commands and parses the output. Run `ls`, `dir`, `find`, `net user`, `ipconfig` — the mapper builds a searchable map of the remote filesystem without sending extra commands.

**Per-Command Session Logging**
Every command gets its own timestamped log file. A session index tracks what you ran and when. Built for OSCP exam proof and engagement reporting — no more lost terminal history.

**Target Identity by Fingerprint**
Targets are identified by SSH host key fingerprint, not IP. Reconnect after DHCP changes, access dual-homed hosts, or reuse lab IPs — Slipstream knows it's the same (or different) machine.

</td>
</tr>
</table>

---

## Quick Start

### Prerequisites

<table>
<tr>
<th>Requirement</th>
<th>Version</th>
<th>Purpose</th>
</tr>
<tr>
<td>Rust</td>
<td>1.70+</td>
<td>Compiler toolchain</td>
</tr>
<tr>
<td>OpenSSH</td>
<td>6.8+</td>
<td>SSH client with ControlMaster support</td>
</tr>
<tr>
<td>Target</td>
<td>Any</td>
<td>Any SSH server (Linux or Windows)</td>
</tr>
</table>

### Build

```bash
# Clone repository
git clone https://github.com/Real-Fruit-Snacks/Slipstream.git
cd Slipstream

# Build
cargo build --release

# Binary at target/release/slipstream (~2.4 MB)
```

### Usage

```bash
# Connect — same as ssh, all flags work
slipstream ssh user@10.10.10.5
slipstream ssh -i ~/.ssh/key -o StrictHostKeyChecking=no admin@target

# Inside the session — ! commands
!help                                    # Show all commands
!tunnel add --type socks -p 1080         # SOCKS proxy
!tunnel add --type local -s 8080 -d 10.10.10.50 -p 80  # Port forward
!upload linpeas.sh /tmp/                 # Upload file
!download /etc/shadow ./loot/            # Download file
!loot                                    # Auto-grab common recon files
!exec whoami                             # Run command via control socket
!note This is the DC                     # Annotate the target
!map                                     # Show mapped filesystem
!sessions                                # List sessions

# Clean up engagement data
slipstream clean
slipstream clean --target SHA256:abc123
```

> Bash history expansion (`!!`, `!$`, `!-1`) passes through to SSH — only known Slipstream commands are intercepted.

---

## Commands

### Session

| Command | Description |
|---------|-------------|
| `!sessions` | List active sessions with labels |
| `!switch <id>` | Switch to session by ID |
| `!connect <target>` | Open new session in tmux window |
| `!rename <id> <label>` | Label a session (e.g., "DC", "web-server") |
| `!kill <id>` | Kill session and its tunnels |
| `!bg` | Background info (Ctrl+Z / tmux) |

### Tunnels

| Command | Description |
|---------|-------------|
| `!tunnel add --type socks -p 1080` | SOCKS proxy |
| `!tunnel add --type local -s 8080 -d host -p 80` | Local port forward |
| `!tunnel add --type reverse -s 9090 -d 127.0.0.1 -p 4444` | Reverse forward |
| `!tunnel list [-v]` | List active tunnels |
| `!tunnel del <id>` | Remove tunnel (executes `ssh -O cancel`) |
| `!tunnel flush` | Remove all tunnels |
| `!tunnel save` | Save tunnel config to target data |
| `!tunnel restore` | Restore saved tunnels on reconnect |

### File Transfer

| Command | Description |
|---------|-------------|
| `!upload <local> <remote>` | Upload file (SFTP > SCP > cat > base64) |
| `!download <remote> <local>` | Download file (same fallback chain) |
| `!upload --method scp <local> <remote>` | Force specific method |
| `!transfer-method [method]` | Get or set default transfer method |

### Filesystem Mapper

| Command | Description |
|---------|-------------|
| `!map` | Show mapped filesystem tree |
| `!map /path` or `!map C:\path` | Browse specific directory |
| `!map find *.conf` | Search by pattern |
| `!map find suid` | Find SUID binaries |
| `!map users` | Show discovered users |
| `!map coverage` | Show explored directories |
| `!map export` | Export map as JSON |
| `!map reset` | Clear map data |

### Red Team QOL

| Command | Description |
|---------|-------------|
| `!loot [dir]` | Auto-grab common recon files (passwd, shadow, SAM, ipconfig, etc.) |
| `!exec <cmd>` | Run command via control socket (doesn't pollute interactive session) |
| `!note <text>` | Add timestamped note to target |
| `!note` | View all notes for current target |

### Help

| Command | Description |
|---------|-------------|
| `!help` | Show all commands |
| `!help <command>` | Detailed help for a command |
| `!?` | Alias for `!help` |
| `!<command> --help` | Help for any command |

---

## Architecture

```
+---------------------------------------------------------+
|                     slipstream                          |
|                                                         |
|  +------------+    +-------------------------------+    |
|  | PTY Layer  |<-->| Input Interceptor             |    |
|  |            |    | (prompt-aware, cooked/raw)     |    |
|  +-----+------+    +-------------+-----------------+    |
|        |                         |                      |
|  +-----v------+    +------------v-----------------+     |
|  | SSH Child   |    | Command Router               |    |
|  | Process     |    |                              |    |
|  | (real ssh)  |    | !tunnel  !upload  !exec      |    |
|  +-----+------+    | !map     !loot    !note       |    |
|        |           | !sessions !connect !help ...   |    |
|  +-----v------+    +------------------------------+     |
|  | Master      |                                        |
|  | Socket      |    +-------------+  +-----------+      |
|  | (Control)   |    | Log Engine  |  | FS Mapper |      |
|  +-------------+    +-------------+  +-----------+      |
+---------------------------------------------------------+
```

| Layer | Implementation |
|-------|----------------|
| **Transport** | Wraps real `ssh` binary via PTY — all SSH features pass through |
| **Multiplexing** | `ControlMaster=auto` with Slipstream-owned socket path |
| **Tunnels** | `ssh -O forward` / `ssh -O cancel` via master socket |
| **Transfers** | SFTP / SCP / cat / base64 fallback chain over master socket |
| **Logging** | Per-command files + session index with timestamps |
| **Mapping** | Passive output parsing — `ls`, `dir`, `find`, `net user`, `ipconfig` |
| **Identity** | SSH host key fingerprint as primary key, conflict detection |
| **OS Detection** | Auto-detects Linux vs Windows from SSH output |

---

## Internals

### Input Interception

Slipstream uses a **prompt-aware interception model**. In cooked mode (normal shell), it buffers input and checks for `!<known_command>`. Unknown `!` sequences (`!!`, `!$`, `!foobar`) pass through to SSH for bash history expansion. In raw mode (vim, top, less, tmux), all input passes through unmodified — detected via alternate screen buffer escape sequences.

### Target Identity

Targets are identified by SSH host key fingerprint, captured from `ssh -v` output during handshake. The fingerprint is the primary key — IP addresses are metadata. This handles DHCP changes, dual-homed hosts, and lab IP reuse. When a fingerprint changes for a known IP, Slipstream prompts: Archive, Keep, or Ignore.

### OS Detection

Slipstream auto-detects the target OS from SSH output (presence of "Windows", "Microsoft", `C:\`, etc.) and adapts:

<table>
<tr>
<th>Feature</th>
<th>Linux</th>
<th>Windows</th>
</tr>
<tr>
<td>Mapper parsers</td>
<td><code>ls</code>, <code>find</code>, <code>tree</code>, <code>/etc/passwd</code></td>
<td><code>dir</code>, <code>net user</code>, <code>ipconfig</code></td>
</tr>
<tr>
<td>CWD tracking</td>
<td><code>/</code> separator</td>
<td><code>\</code> separator</td>
</tr>
<tr>
<td>Transfer paths</td>
<td>Forward slashes</td>
<td>Auto-converted forward slashes</td>
</tr>
<tr>
<td>Boundary detection</td>
<td><code>PROMPT_COMMAND</code></td>
<td>PowerShell <code>prompt</code> function</td>
</tr>
<tr>
<td>User hint</td>
<td>"Run: cat /etc/passwd"</td>
<td>"Run: net user"</td>
</tr>
<tr>
<td>Loot targets</td>
<td>shadow, crontab, suid</td>
<td>SAM, systeminfo, tasklist</td>
</tr>
</table>

### Data Organization

```
~/.config/slipstream/
+-- config.toml                         # Global config (optional)
+-- targets/
|   +-- SHA256-fingerprint/
|       +-- target.toml                 # Identity, addresses, saved tunnels
|       +-- notes.txt                   # Target annotations
|       +-- map.json                    # Filesystem map
|       +-- logs/
|           +-- 2026-03-20_14-30-00/
|               +-- session.log         # Command index with timestamps
|               +-- 001_whoami.log      # Per-command output
|               +-- 002_ls_-la.log
+-- sessions/
    +-- ssh-host-pid.sock               # Master socket (runtime only)
```

---

## Project Structure

```
slipstream/
+-- src/
|   +-- lib.rs                # Library root
|   +-- main.rs               # Entry point, PTY spawn, I/O loop
|   +-- config.rs             # TOML config with sane defaults
|   +-- pty_loop.rs           # Command dispatch, transfer/mapper/tunnel wiring
|   +-- target_os.rs          # TargetOS enum with auto-detection
|   +-- signals.rs            # SIGWINCH/SIGHUP/SIGTERM handlers
|   +-- ssh/
|   |   +-- discovery.rs      # Find real ssh binary on $PATH
|   |   +-- args.rs           # Parse SSH arguments
|   |   +-- process.rs        # Build SSH command with ControlMaster
|   |   +-- fingerprint.rs    # Parse host key from ssh -v
|   |   +-- orphan.rs         # Stale socket detection
|   +-- target/
|   |   +-- identity.rs       # Fingerprint-based target resolution
|   |   +-- storage.rs        # target.toml read/write
|   |   +-- conflict.rs       # Archive/Keep/Ignore prompts
|   +-- input/
|   |   +-- mode.rs           # Cooked/raw terminal mode
|   |   +-- line_buffer.rs    # Keystroke buffer
|   |   +-- router.rs         # ! command routing
|   +-- logging/
|   |   +-- engine.rs         # Session + per-command logs
|   |   +-- boundary.rs       # PROMPT_COMMAND marker detection
|   |   +-- writer.rs         # Atomic file writes with flock
|   +-- session/
|   |   +-- manager.rs        # Session lifecycle
|   +-- tunnel/
|   |   +-- manager.rs        # Tunnel CRUD + SSH forward execution
|   +-- transfer/
|   |   +-- fallback.rs       # Transfer methods + fallback chain
|   +-- mapper/
|   |   +-- parser.rs         # Output parsers (ls, dir, find, net user, ipconfig)
|   |   +-- store.rs          # map.json with merge-on-add
|   |   +-- query.rs          # find, coverage, export, tree
|   |   +-- cwd.rs            # Working directory tracker
|   +-- commands/
|       +-- help.rs           # Help system
|       +-- tunnel_cmd.rs     # Tunnel display formatting
+-- tests/                    # 118 integration tests
+-- docs/
    +-- banner.svg
    +-- assets/
        +-- logo-dark.svg
        +-- logo-light.svg
    +-- index.html
```

~6,400 lines of Rust. 38 source files. 118 tests. 2.4 MB binary.

---

## Platform Support

<table>
<tr>
<th>Capability</th>
<th>Linux Targets</th>
<th>Windows Targets</th>
</tr>
<tr>
<td>SSH Connection</td>
<td>Full</td>
<td>Full (OpenSSH for Windows)</td>
</tr>
<tr>
<td>Tunnel Management</td>
<td>Full (SOCKS, local, reverse)</td>
<td>Full (SOCKS, local, reverse)</td>
</tr>
<tr>
<td>File Transfer</td>
<td>SFTP / SCP / cat / base64</td>
<td>SFTP / SCP / cat / base64</td>
</tr>
<tr>
<td>Mapper Parsers</td>
<td><code>ls</code>, <code>find</code>, <code>tree</code>, <code>/etc/passwd</code></td>
<td><code>dir</code>, <code>net user</code>, <code>ipconfig</code></td>
</tr>
<tr>
<td>CWD Tracking</td>
<td><code>PROMPT_COMMAND</code> injection</td>
<td>PowerShell <code>prompt</code> function</td>
</tr>
<tr>
<td>Path Handling</td>
<td>Forward slashes (native)</td>
<td>Auto backslash-to-slash conversion</td>
</tr>
<tr>
<td>Loot Targets</td>
<td>passwd, shadow, crontab, SUID</td>
<td>SAM, systeminfo, tasklist, net user</td>
</tr>
<tr>
<td>OS Detection</td>
<td>Automatic</td>
<td>Automatic</td>
</tr>
<tr>
<td>Session Logging</td>
<td>Full</td>
<td>Full</td>
</tr>
</table>

---

## Configuration

### Config File

Optional persistent configuration at `~/.config/slipstream/config.toml`:

```toml
[defaults]
transfer_method = "sftp"    # sftp, scp, cat, base64
auto_loot = false           # Run !loot on connect
auto_map = true             # Enable passive mapper
```

CLI flags always override config values.

### Data Directories

```
~/.config/slipstream/
+-- config.toml                 # Global config
+-- targets/
|   +-- SHA256-fingerprint/     # Per-target data keyed by host key
+-- sessions/
    +-- *.sock                  # Runtime master sockets
```

---

## Security

### Vulnerability Reporting

**Report security issues via:**
- GitHub Security Advisories (preferred)
- Private disclosure to maintainers
- Responsible disclosure timeline (90 days)

**Do NOT:**
- Open public GitHub issues for vulnerabilities
- Disclose before coordination with maintainers
- Exploit vulnerabilities in unauthorized contexts

### Threat Model

**In scope:**
- Operator-side session management and logging
- Tunneling over existing SSH connections
- Passive reconnaissance from command output

**Out of scope:**
- Hiding SSH connections from network monitoring
- Evading endpoint detection on the target
- Bypassing SSH authentication mechanisms

### What Slipstream Does NOT Do

Slipstream is an **SSH wrapper**, not an exploitation framework:

- **Not a C2 framework** — No implant management, beaconing, or tasking
- **Not a vulnerability scanner** — Does not probe for exploits
- **Not an exploit framework** — No payload generation or injection
- **Not anti-forensics** — Logs everything by design, does not tamper with target logs

---

## License

MIT License

Copyright &copy; 2026 Real-Fruit-Snacks

```
THIS SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND.
THE AUTHORS ARE NOT LIABLE FOR ANY DAMAGES ARISING FROM USE.
USE AT YOUR OWN RISK AND ONLY WITH PROPER AUTHORIZATION.
```

---

## Resources

- **GitHub**: [github.com/Real-Fruit-Snacks/Slipstream](https://github.com/Real-Fruit-Snacks/Slipstream)
- **Issues**: [Report a Bug](https://github.com/Real-Fruit-Snacks/Slipstream/issues)
- **Security**: [SECURITY.md](SECURITY.md)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

---

<div align="center">

**Part of the Real-Fruit-Snacks water-themed security toolkit**

[Aquifer](https://github.com/Real-Fruit-Snacks/Aquifer) • [Cascade](https://github.com/Real-Fruit-Snacks/Cascade) • [Conduit](https://github.com/Real-Fruit-Snacks/Conduit) • [Deadwater](https://github.com/Real-Fruit-Snacks/Deadwater) • [Deluge](https://github.com/Real-Fruit-Snacks/Deluge) • [Depth](https://github.com/Real-Fruit-Snacks/Depth) • [Dew](https://github.com/Real-Fruit-Snacks/Dew) • [Droplet](https://github.com/Real-Fruit-Snacks/Droplet) • [Fathom](https://github.com/Real-Fruit-Snacks/Fathom) • [Flux](https://github.com/Real-Fruit-Snacks/Flux) • [Grotto](https://github.com/Real-Fruit-Snacks/Grotto) • [HydroShot](https://github.com/Real-Fruit-Snacks/HydroShot) • [Maelstrom](https://github.com/Real-Fruit-Snacks/Maelstrom) • [Rapids](https://github.com/Real-Fruit-Snacks/Rapids) • [Ripple](https://github.com/Real-Fruit-Snacks/Ripple) • [Riptide](https://github.com/Real-Fruit-Snacks/Riptide) • [Runoff](https://github.com/Real-Fruit-Snacks/Runoff) • [Seep](https://github.com/Real-Fruit-Snacks/Seep) • [Shallows](https://github.com/Real-Fruit-Snacks/Shallows) • [Siphon](https://github.com/Real-Fruit-Snacks/Siphon) • **Slipstream** • [Spillway](https://github.com/Real-Fruit-Snacks/Spillway) • [Surge](https://github.com/Real-Fruit-Snacks/Surge) • [Tidemark](https://github.com/Real-Fruit-Snacks/Tidemark) • [Tidepool](https://github.com/Real-Fruit-Snacks/Tidepool) • [Undercurrent](https://github.com/Real-Fruit-Snacks/Undercurrent) • [Undertow](https://github.com/Real-Fruit-Snacks/Undertow) • [Vapor](https://github.com/Real-Fruit-Snacks/Vapor) • [Wellspring](https://github.com/Real-Fruit-Snacks/Wellspring) • [Whirlpool](https://github.com/Real-Fruit-Snacks/Whirlpool)

*Remember: With great power comes great responsibility.*

</div>

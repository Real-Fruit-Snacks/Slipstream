<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/Real-Fruit-Snacks/Slipstream/main/docs/assets/logo-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/Real-Fruit-Snacks/Slipstream/main/docs/assets/logo-light.svg">
  <img alt="Slipstream" src="https://raw.githubusercontent.com/Real-Fruit-Snacks/Slipstream/main/docs/assets/logo-dark.svg" width="520">
</picture>

![Rust](https://img.shields.io/badge/language-Rust-orange.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20targets-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**SSH wrapper for red team operations — drop-in replacement with superpowers.**

Wraps the real `ssh` binary via PTY, intercepting `!` commands for tunnel management, file transfers, passive filesystem mapping, and per-command session logging. All SSH functionality passes through unchanged.

> **Authorization Required**: Designed exclusively for authorized security testing with explicit written permission.

</div>

---

## Quick Start

**Prerequisites:** Rust 1.70+, OpenSSH 6.8+ (client)

```bash
git clone https://github.com/Real-Fruit-Snacks/Slipstream.git
cd Slipstream
cargo build --release
```

**Verify:**

```bash
./target/release/slipstream ssh user@10.10.10.5
```

---

## Features

### Tunnels

iptables-style syntax for SSH tunnels. Real forwarding via `ssh -O forward` over the master socket.

```bash
!tunnel add --type socks -p 1080                          # SOCKS proxy
!tunnel add --type local -s 8080 -d 10.10.10.50 -p 80    # local forward
!tunnel add --type reverse -s 9090 -d 127.0.0.1 -p 4444  # reverse forward
!tunnel list                                               # show active
!tunnel save                                               # persist config
```

### File Transfer

Automatic fallback chain: SFTP → SCP → cat-over-SSH → base64. Windows paths handled transparently.

```bash
!upload linpeas.sh /tmp/              # upload file
!download /etc/shadow ./loot/         # download file
!upload --method scp tool.bin /opt/   # force specific method
```

### Filesystem Mapper

Passive output parsing — watches your commands and builds a searchable map without sending extra traffic.

```bash
!map                    # show mapped filesystem tree
!map find *.conf        # search by pattern
!map find suid          # find SUID binaries
!map users              # show discovered users
!map export             # export as JSON
```

### Session Logging

Every command gets its own timestamped log file. Built for OSCP exam proof and engagement reporting.

```bash
!sessions               # list active sessions
!note This is the DC    # annotate the target
!loot                   # auto-grab common recon files
!exec whoami            # run command via control socket
```

### Target Identity

Targets identified by SSH host key fingerprint, not IP. Handles DHCP changes, dual-homed hosts, and lab IP reuse. Conflict detection prompts: Archive, Keep, or Ignore.

### OS Detection

Auto-detects Linux vs Windows from SSH output. Adapts mapper parsers, CWD tracking, transfer paths, and loot targets per platform.

---

## Architecture

```
src/
├── main.rs          # Entry point, PTY spawn
├── ssh/             # Binary discovery, args, fingerprint, master socket
├── input/           # Prompt-aware interception, cooked/raw mode
├── tunnel/          # CRUD + SSH forward execution
├── transfer/        # SFTP/SCP/cat/base64 fallback chain
├── mapper/          # Output parsers, store, query, CWD tracking
├── logging/         # Per-command logs, boundary detection
├── session/         # Session lifecycle management
└── target/          # Fingerprint identity, conflict resolution
```

PTY-based interception model. In cooked mode, buffers input and checks for `!<known_command>`. Unknown `!` sequences pass through for bash history expansion. In raw mode (vim, top, tmux), all input passes through unmodified.

~6,400 lines of Rust. 38 source files. 118 tests. 2.4 MB binary.

---

## Platform Support

| | Linux Attacker | Windows Target | Linux Target |
|---|---|---|---|
| SSH Wrapping | Full | N/A | N/A |
| Tunnels | Full | Full | Full |
| File Transfer | Full | Full (path conversion) | Full |
| Mapper | ls, find, /etc/passwd | dir, net user, ipconfig | ls, find, /etc/passwd |
| Logging | Full | Full | Full |

---

## Security

Report vulnerabilities via [GitHub Security Advisories](https://github.com/Real-Fruit-Snacks/Slipstream/security/advisories). 90-day responsible disclosure.

**Slipstream does not:**
- Modify SSH traffic or inject commands into the session
- Store credentials (uses the system's ssh binary and keys)
- Bypass SSH authentication or encryption
- Operate without a real SSH connection to the target

---

## License

[MIT](LICENSE) — Copyright 2026 Real-Fruit-Snacks

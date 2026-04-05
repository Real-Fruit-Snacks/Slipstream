# Changelog

All notable changes to Slipstream will be documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-04

### Added
- Drop-in SSH replacement via PTY wrapping of real ssh binary
- Prompt-aware input interception (cooked/raw mode detection)
- Tunnel management with iptables-style syntax (SOCKS, local, reverse)
- Real SSH forwarding via `ssh -O forward` over master socket
- Tunnel save/restore per target
- File upload and download with SFTP/SCP/cat/base64 fallback chain
- Windows path handling with automatic backslash-to-forward-slash conversion
- Passive filesystem mapper from command output (ls, dir, find, net user, ipconfig)
- Filesystem map search, coverage, export, and tree display
- Per-command session logging with timestamped files
- Session index with command tracking
- Boundary detection via PROMPT_COMMAND (Linux) and PowerShell prompt (Windows)
- Target identity by SSH host key fingerprint
- Fingerprint conflict detection with Archive/Keep/Ignore prompts
- Automatic OS detection (Linux vs Windows) from SSH output
- OS-adaptive parser selection, CWD tracking, and loot targets
- Auto-loot for common recon files (passwd, shadow, SAM, systeminfo, etc.)
- Command execution via control socket (`!exec`)
- Target annotation system (`!note`)
- Session management (list, switch, connect, rename, kill, background)
- Comprehensive help system with per-command documentation
- 118 integration tests
- Data organization by fingerprint under `~/.config/slipstream/`

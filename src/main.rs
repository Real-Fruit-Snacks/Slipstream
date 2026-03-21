use clap::{Parser, Subcommand};
use crossterm::terminal;
use nix::fcntl::OFlag;
use nix::libc;
use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt, PtyMaster};
use nix::sys::termios::{self, SetArg};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{close, dup2, execvp, fork, read, setsid, write, ForkResult, Pid};
use std::ffi::CString;
use std::io;
use std::os::unix::io::{AsRawFd, BorrowedFd, RawFd};
use std::path::PathBuf;

use slipstream::config::Config;
use slipstream::input::mode::TerminalMode;
use slipstream::input::router::RouteResult;
use slipstream::logging::boundary::BoundaryDetector;
use slipstream::pty_loop::PtyLoop;
use slipstream::session::manager::{Session, SessionState};
use slipstream::signals::{check_shutdown, check_sigwinch, setup_signal_handlers};
use slipstream::ssh::args::SshArgs;
use slipstream::ssh::discovery::SshDiscovery;
use slipstream::ssh::fingerprint::FingerprintParser;
use slipstream::ssh::process::SshProcess;
use slipstream::target::storage::TargetStorage;

#[derive(Parser)]
#[command(name = "slipstream", version, about = "SSH wrapper for red team operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect via SSH (all ssh flags are passed through)
    Ssh {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Clean up session data
    Clean {
        #[arg(long)]
        target: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ssh { args } => {
            if let Err(e) = run_ssh(args) {
                eprintln!("slipstream: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Clean { target } => {
            clean(target);
        }
    }
}

fn clean(target: Option<String>) {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("slipstream"))
        .expect("Could not determine config directory");

    if let Some(fingerprint) = target {
        let target_dir = config_dir.join("targets").join(
            fingerprint.replace([':', '/', '\\'], "-")
        );
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir).ok();
            eprintln!("  Cleaned target: {}", fingerprint);
        } else {
            eprintln!("  Target not found: {}", fingerprint);
        }
    } else {
        let targets_dir = config_dir.join("targets");
        if targets_dir.exists() {
            std::fs::remove_dir_all(&targets_dir).ok();
        }
        let sessions_dir = config_dir.join("sessions");
        if sessions_dir.exists() {
            std::fs::remove_dir_all(&sessions_dir).ok();
        }
        eprintln!("  All Slipstream data cleaned.");
    }
}

fn run_ssh(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();

    let own_exe = std::env::current_exe().ok();
    let own_exe_str = own_exe.as_ref().and_then(|p| p.to_str());
    let config_ssh = if config.general.ssh_binary.is_empty() {
        None
    } else {
        Some(config.general.ssh_binary.as_str())
    };
    let ssh_binary = SshDiscovery::find_ssh(config_ssh, own_exe_str)?;

    let ssh_args = SshArgs::parse(&args);

    setup_signal_handlers()?;

    let sessions_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config")
        .join("slipstream")
        .join("sessions");
    std::fs::create_dir_all(&sessions_dir)?;

    let socket_name = format!("ssh-{}-{}.sock", ssh_args.host, std::process::id());
    let control_path = sessions_dir.join(socket_name);

    let proc = SshProcess::new(ssh_binary, args.clone(), control_path.clone());
    let cmd = proc.build_command();

    let pty_master = posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY)?;
    grantpt(&pty_master)?;
    unlockpt(&pty_master)?;
    let slave_name = unsafe { ptsname(&pty_master) }?;

    let stdin_fd = io::stdin().as_raw_fd();
    let original_termios = termios::tcgetattr(unsafe { BorrowedFd::borrow_raw(stdin_fd) }).ok();

    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    set_pty_size(pty_master.as_raw_fd(), rows, cols);

    match unsafe { fork()? } {
        ForkResult::Child => {
            child_exec(&slave_name, &cmd);
        }
        ForkResult::Parent { child } => {
            let result = parent_io_loop(
                config,
                &pty_master,
                child,
                stdin_fd,
                &ssh_args,
                Some(control_path),
            );

            if let Some(ref orig) = original_termios {
                let _ = termios::tcsetattr(
                    unsafe { BorrowedFd::borrow_raw(stdin_fd) },
                    SetArg::TCSAFLUSH,
                    orig,
                );
            }

            match result {
                Ok(exit_code) => std::process::exit(exit_code),
                Err(e) => {
                    eprintln!("slipstream: I/O loop error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    #[allow(unreachable_code)]
    Ok(())
}

fn child_exec(slave_name: &str, cmd: &[String]) -> ! {
    setsid().expect("setsid failed");

    let slave_fd = nix::fcntl::open(
        slave_name,
        OFlag::O_RDWR,
        nix::sys::stat::Mode::empty(),
    )
    .expect("open slave pty failed");

    unsafe {
        libc::ioctl(slave_fd, libc::TIOCSCTTY, 0);
    }

    dup2(slave_fd, 0).expect("dup2 stdin");
    dup2(slave_fd, 1).expect("dup2 stdout");
    dup2(slave_fd, 2).expect("dup2 stderr");

    if slave_fd > 2 {
        let _ = close(slave_fd);
    }

    if cmd.is_empty() {
        eprintln!("slipstream: empty command");
        std::process::exit(1);
    }

    let c_cmd: Vec<CString> = cmd
        .iter()
        .map(|s| CString::new(s.as_str()).expect("CString"))
        .collect();

    let prog = &c_cmd[0];
    let argv: Vec<&CString> = c_cmd.iter().collect();

    match execvp(prog, &argv) {
        Ok(_) => unreachable!(),
        Err(e) => {
            eprintln!("slipstream: execvp failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn parent_io_loop(
    config: Config,
    pty_master: &PtyMaster,
    child: Pid,
    stdin_fd: RawFd,
    ssh_args: &SshArgs,
    control_path: Option<PathBuf>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let pty_fd = pty_master.as_raw_fd();
    let stdout_fd = io::stdout().as_raw_fd();

    let mut raw_termios =
        termios::tcgetattr(unsafe { BorrowedFd::borrow_raw(stdin_fd) })?;
    termios::cfmakeraw(&mut raw_termios);
    termios::tcsetattr(
        unsafe { BorrowedFd::borrow_raw(stdin_fd) },
        SetArg::TCSANOW,
        &raw_termios,
    )?;

    // Build ssh_user_host string for control socket commands
    let ssh_user_host = match &ssh_args.user {
        Some(u) => format!("{}@{}", u, ssh_args.host),
        None => ssh_args.host.clone(),
    };

    let mut pty_loop = PtyLoop::new(config, control_path, ssh_user_host);
    pty_loop.set_child_pid(child);

    let session = Session {
        user: ssh_args.user.clone().unwrap_or_else(|| {
            std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
        }),
        host: ssh_args.host.clone(),
        hostname: None,
        port: ssh_args.port,
        state: SessionState::Connecting,
        label: None,
    };
    pty_loop.session_manager_mut().create(session);

    let mut read_buf = [0u8; 4096];
    let mut exit_code = 0i32;

    // Fix 3: Fingerprint capture state
    let mut captured_fingerprint: Option<String> = None;
    let mut fingerprint_resolved = false;

    // Fix 5: PROMPT_COMMAND injection state
    let mut prompt_injected = false;

    'main: loop {
        if check_shutdown() {
            break 'main;
        }

        if check_sigwinch() {
            if let Ok((cols, rows)) = terminal::size() {
                set_pty_size(pty_fd, rows, cols);
            }
        }

        match waitpid(child, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, code)) => {
                exit_code = code;
                break 'main;
            }
            Ok(WaitStatus::Signaled(_, _, _)) => {
                exit_code = 1;
                break 'main;
            }
            _ => {}
        }

        let mut fds = [
            PollFd::new(
                unsafe { BorrowedFd::borrow_raw(stdin_fd) },
                PollFlags::POLLIN,
            ),
            PollFd::new(
                unsafe { BorrowedFd::borrow_raw(pty_fd) },
                PollFlags::POLLIN,
            ),
        ];

        match poll(&mut fds, PollTimeout::try_from(50i32).unwrap()) {
            Err(nix::errno::Errno::EINTR) => continue,
            Err(_) => break 'main,
            Ok(0) => continue,
            Ok(_) => {}
        }

        // pty_master -> stdout
        let pty_revents = fds[1].revents().unwrap_or(PollFlags::empty());
        if pty_revents.contains(PollFlags::POLLIN) {
            match read(pty_fd, &mut read_buf) {
                Ok(0) => {
                    exit_code = 0;
                    break 'main;
                }
                Ok(n) => {
                    // Filter ssh -v debug lines from output
                    let data = &read_buf[..n];
                    let text = String::from_utf8_lossy(data);

                    // Fix 3: Capture fingerprint from debug output
                    if captured_fingerprint.is_none() {
                        for line in text.lines() {
                            if let Some(fp) = FingerprintParser::parse_line(line) {
                                captured_fingerprint = Some(fp);
                                break;
                            }
                        }
                    }

                    if text.contains("debug1: ") {
                        // Filter line by line, only pass non-debug lines
                        let filtered: String = text
                            .lines()
                            .filter(|line| !line.starts_with("debug1: "))
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !filtered.trim().is_empty() {
                            let bytes = filtered.as_bytes();
                            write_all(stdout_fd, bytes);
                            // Preserve trailing newline if original had one
                            if data.last() == Some(&b'\n') && !filtered.ends_with('\n') {
                                write_all(stdout_fd, b"\n");
                            }
                            // Fix 4: Log output
                            pty_loop.log_output(&filtered);
                        }
                    } else {
                        write_all(stdout_fd, data);
                        // Fix 4: Log output
                        pty_loop.log_output(&text);
                    }

                    // Fix 2: Terminal mode detection — detect alternate screen buffer
                    // vim/top/less/tmux use ESC[?1049h (enter) and ESC[?1049l (exit)
                    // Also detect ESC[?47h/ESC[?47l (older alternate screen)
                    if text.contains("\x1b[?1049h") || text.contains("\x1b[?47h") {
                        pty_loop.mode_tracker_mut().set(TerminalMode::Raw);
                    } else if text.contains("\x1b[?1049l") || text.contains("\x1b[?47l") {
                        pty_loop.mode_tracker_mut().set(TerminalMode::Cooked);
                    }

                    if pty_loop.target_os() == slipstream::target_os::TargetOS::Unknown {
                        if let Some(os) = slipstream::target_os::TargetOS::detect(&text) {
                            pty_loop.set_target_os(os);
                        }
                    }

                    // Fix 3: Resolve target once fingerprint is captured
                    if !fingerprint_resolved {
                        if let Some(ref fp) = captured_fingerprint {
                            fingerprint_resolved = true;
                            pty_loop.set_fingerprint(fp.clone());
                            let targets_dir = dirs::home_dir()
                                .unwrap_or_else(|| PathBuf::from("/tmp"))
                                .join(".config/slipstream/targets");
                            let storage = TargetStorage::new(targets_dir);
                            // Ensure the target directory exists
                            if let Ok(_target_dir) = storage.ensure_target_dir(fp) {
                                // Save target.toml with identity info
                                let now = chrono::Utc::now();
                                let target_info = slipstream::target::storage::TargetInfo {
                                    identity: slipstream::target::storage::Identity {
                                        fingerprint: fp.clone(),
                                        hostname: ssh_args.host.clone(),
                                    },
                                    addresses: vec![slipstream::target::storage::Address {
                                        ip: ssh_args.host.clone(),
                                        port: ssh_args.port,
                                        first_seen: now,
                                        last_seen: now,
                                    }],
                                    saved_tunnels: vec![],
                                };
                                let _ = storage.save_target(&target_info);

                                // Initialize logging with session directory
                                if let Ok(session_dir) = storage.create_session_dir(fp) {
                                    pty_loop.init_logging(session_dir);
                                    pty_loop.log_event(&format!(
                                        "session start — host={} fingerprint={}",
                                        ssh_args.host, fp
                                    ));
                                }
                            }
                        }
                    }

                    // Fix 5: PROMPT_COMMAND injection
                    // PowerShell prompt injection
                    if !prompt_injected && text.contains("PS ") && text.contains(":\\") {
                        let injection = BoundaryDetector::powershell_injection();
                        write_all(pty_fd, format!(" {}\r", injection).as_bytes());
                        prompt_injected = true;
                    }
                    // Unix shell prompt injection — skip if "Windows" or "Microsoft" appears (cmd.exe)
                    else if !prompt_injected && !text.contains("Microsoft") && !text.contains("Windows") {
                        if text.contains("$ ") || text.contains("# ") || text.contains("% ") {
                            let injection = BoundaryDetector::bash_injection();
                            // Send silently — the command itself will be echoed but that's OK
                            write_all(pty_fd, format!(" {}\r", injection).as_bytes());
                            prompt_injected = true;
                        }
                    }
                }
                Err(nix::errno::Errno::EIO) => {
                    exit_code = 0;
                    break 'main;
                }
                Err(_) => break 'main,
            }
        }

        // stdin -> pty_master
        let stdin_revents = fds[0].revents().unwrap_or(PollFlags::empty());
        if stdin_revents.contains(PollFlags::POLLIN) {
            let n = match read(stdin_fd, &mut read_buf) {
                Ok(0) => break 'main,
                Ok(n) => n,
                Err(nix::errno::Errno::EINTR) => continue,
                Err(_) => break 'main,
            };

            let data = &read_buf[..n];
            let mode = pty_loop.terminal_mode();

            match mode {
                TerminalMode::Raw => {
                    write_all(pty_fd, data);
                }
                TerminalMode::Cooked => {
                    // Cooked mode: buffer input, check for ! commands on Enter.
                    // Key insight: if the line starts with the escape prefix,
                    // we handle it entirely locally (echo + dispatch).
                    // If it doesn't, we forward chars to SSH for echo + editing.
                    let prefix = pty_loop.escape_prefix().to_string();
                    for &byte in data {
                        match byte {
                            b'\r' | b'\n' => {
                                let line = pty_loop.line_buffer_mut().take();
                                match pty_loop.router().route(&line) {
                                    RouteResult::SlipstreamCommand { command, args } => {
                                        // Handled locally — show response
                                        write_all(stdout_fd, b"\r\n");
                                        pty_loop.log_event(&format!(
                                            "command: {} {}",
                                            command, args
                                        ));
                                        let response =
                                            pty_loop.handle_command(&command, &args);
                                        write_all(stdout_fd, response.as_bytes());
                                    }
                                    RouteResult::Passthrough => {
                                        // Fix 4: Log the command being sent to SSH
                                        pty_loop.log_command(&line);
                                        // Line was forwarded char-by-char already
                                        // Just send Enter to SSH
                                        write_all(pty_fd, b"\r");
                                    }
                                }
                            }
                            0x03 => {
                                // Ctrl+C — forward to SSH, clear buffer
                                pty_loop.line_buffer_mut().clear();
                                write_all(pty_fd, &[byte]);
                            }
                            0x04 => {
                                // Ctrl+D — if line buffer is empty, forward EOF to SSH
                                if pty_loop.line_buffer_mut().content().is_empty() {
                                    write_all(pty_fd, &[byte]);
                                }
                                // If buffer has content, ignore (like bash)
                            }
                            0x08 | 0x7f => {
                                let content = pty_loop.line_buffer_mut().content();
                                let is_local = content.starts_with(&prefix);
                                pty_loop.line_buffer_mut().backspace();
                                if is_local {
                                    // Local echo: move cursor back, overwrite, move back
                                    write_all(stdout_fd, b"\x08 \x08");
                                } else {
                                    write_all(pty_fd, &[byte]);
                                }
                            }
                            _ => {
                                if let Ok(s) = std::str::from_utf8(&[byte]) {
                                    if let Some(c) = s.chars().next() {
                                        pty_loop.line_buffer_mut().push(c);
                                    }
                                }
                                let content = pty_loop.line_buffer_mut().content();
                                if content.starts_with(&prefix) {
                                    // This is a potential ! command — echo locally
                                    write_all(stdout_fd, &[byte]);
                                } else {
                                    // Normal input — forward to SSH for echo
                                    write_all(pty_fd, &[byte]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Log session end
    pty_loop.log_event("session end");

    // Drain remaining PTY output
    {
        let mut drain_fds = [PollFd::new(
            unsafe { BorrowedFd::borrow_raw(pty_fd) },
            PollFlags::POLLIN,
        )];
        loop {
            match poll(&mut drain_fds, PollTimeout::try_from(10i32).unwrap()) {
                Ok(n) if n > 0 => {}
                _ => break,
            }
            if !drain_fds[0]
                .revents()
                .unwrap_or(PollFlags::empty())
                .contains(PollFlags::POLLIN)
            {
                break;
            }
            match read(pty_fd, &mut read_buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&read_buf[..n]);
                    if text.contains("debug1: ") {
                        let filtered: String = text
                            .lines()
                            .filter(|line| !line.starts_with("debug1: "))
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !filtered.trim().is_empty() {
                            write_all(stdout_fd, filtered.as_bytes());
                            if read_buf[..n].last() == Some(&b'\n') && !filtered.ends_with('\n') {
                                write_all(stdout_fd, b"\n");
                            }
                        }
                    } else {
                        write_all(stdout_fd, &read_buf[..n]);
                    }
                }
            }
        }
    }

    Ok(exit_code)
}

fn write_all(fd: RawFd, data: &[u8]) {
    let mut offset = 0;
    while offset < data.len() {
        match write(unsafe { BorrowedFd::borrow_raw(fd) }, &data[offset..]) {
            Ok(n) => offset += n,
            Err(_) => break,
        }
    }
}

fn set_pty_size(fd: RawFd, rows: u16, cols: u16) {
    let ws = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        libc::ioctl(fd, libc::TIOCSWINSZ, &ws);
    }
}

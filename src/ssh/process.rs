use std::path::PathBuf;

/// Represents an SSH process configuration, including control socket management.
pub struct SshProcess {
    pub ssh_binary: PathBuf,
    pub args: Vec<String>,
    pub control_path: PathBuf,
    pub fingerprint: Option<String>,
}

impl SshProcess {
    /// Create a new SshProcess with the given binary, user args, and control socket path.
    pub fn new(ssh_binary: PathBuf, args: Vec<String>, control_path: PathBuf) -> Self {
        SshProcess {
            ssh_binary,
            args,
            control_path,
            fingerprint: None,
        }
    }

    /// Build the full SSH command as a Vec<String>.
    /// Injects -v, ControlMaster=auto, ControlPath, and ControlPersist=yes before user args.
    pub fn build_command(&self) -> Vec<String> {
        let mut cmd = Vec::new();
        cmd.push(self.ssh_binary.to_string_lossy().into_owned());
        cmd.push("-v".to_string());
        cmd.push("-o".to_string());
        cmd.push("ControlMaster=auto".to_string());
        cmd.push("-o".to_string());
        cmd.push(format!(
            "ControlPath={}",
            self.control_path.to_string_lossy()
        ));
        cmd.push("-o".to_string());
        cmd.push("ControlPersist=yes".to_string());
        cmd.extend(self.args.iter().cloned());
        cmd
    }
}

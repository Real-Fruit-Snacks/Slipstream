#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferMethod {
    Sftp,
    Scp,
    Cat,
    Base64,
}

fn quote_remote_path(path: &str) -> String {
    if path.contains('\\') || path.contains(' ') {
        format!("'{}'", path)
    } else {
        path.to_string()
    }
}

/// Convert Windows backslash paths to forward slashes for SCP/SFTP compatibility.
fn to_forward_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

impl TransferMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sftp" => Some(Self::Sftp),
            "scp" => Some(Self::Scp),
            "cat" => Some(Self::Cat),
            "base64" => Some(Self::Base64),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sftp => "sftp",
            Self::Scp => "scp",
            Self::Cat => "cat",
            Self::Base64 => "base64",
        }
    }

    pub fn upload_command(
        &self,
        socket_path: &str,
        user_host: &str,
        local_path: &str,
        remote_path: &str,
    ) -> String {
        let qremote = quote_remote_path(remote_path);
        match self {
            Self::Sftp => format!(
                "sftp -o ControlPath={} {} <<< 'put {} {}'",
                socket_path, user_host, local_path, qremote
            ),
            Self::Scp => format!(
                "scp -o ControlPath={} {} {}:{}",
                socket_path, local_path, user_host, qremote
            ),
            Self::Cat => format!(
                "ssh -S {} {} \"cat > {}\" < {}",
                socket_path, user_host, qremote, local_path
            ),
            Self::Base64 => format!(
                "cat {} | base64 | ssh -S {} {} \"base64 -d > {}\"",
                local_path, socket_path, user_host, qremote
            ),
        }
    }

    pub fn download_command(
        &self,
        socket_path: &str,
        user_host: &str,
        remote_path: &str,
        local_path: &str,
    ) -> String {
        let qremote = quote_remote_path(remote_path);
        match self {
            Self::Sftp => format!(
                "sftp -o ControlPath={} {} <<< 'get {} {}'",
                socket_path, user_host, qremote, local_path
            ),
            Self::Scp => format!(
                "scp -o ControlPath={} {}:{} {}",
                socket_path, user_host, qremote, local_path
            ),
            Self::Cat => format!(
                "ssh -S {} {} \"cat {}\" > {}",
                socket_path, user_host, qremote, local_path
            ),
            Self::Base64 => format!(
                "ssh -S {} {} \"base64 {}\" | base64 -d > {}",
                socket_path, user_host, qremote, local_path
            ),
        }
    }

    /// Windows-compatible upload: uses forward slashes for SCP/SFTP
    pub fn upload_command_windows(
        &self,
        socket_path: &str,
        user_host: &str,
        local_path: &str,
        remote_path: &str,
    ) -> String {
        let fwd_path = to_forward_slashes(remote_path);
        match self {
            Self::Sftp => format!(
                "sftp -o ControlPath={} {} <<< 'put {} {}'",
                socket_path, user_host, local_path, fwd_path
            ),
            Self::Scp => format!(
                "scp -o ControlPath={} {} {}:{}",
                socket_path, local_path, user_host, fwd_path
            ),
            // Windows: pipe file content via SSH channel
            Self::Cat => format!(
                "ssh -S {} {} \"powershell -c Set-Content -Path '{}' -Value (\\$input) -Encoding Byte\" < {}",
                socket_path, user_host, remote_path.replace('\'', "''"), local_path
            ),
            // Windows: base64 encode locally, decode with powershell on remote
            Self::Base64 => format!(
                "base64 {} | ssh -S {} {} \"powershell -c [IO.File]::WriteAllBytes('{}', [Convert]::FromBase64String((\\$input -join '')))\"",
                local_path, socket_path, user_host, remote_path.replace('\'', "''")
            ),
        }
    }

    /// Windows-compatible download: uses forward slashes for SCP/SFTP, type for cat
    pub fn download_command_windows(
        &self,
        socket_path: &str,
        user_host: &str,
        remote_path: &str,
        local_path: &str,
    ) -> String {
        let fwd_path = to_forward_slashes(remote_path);
        match self {
            Self::Sftp => format!(
                "sftp -o ControlPath={} {} <<< 'get {} {}'",
                socket_path, user_host, fwd_path, local_path
            ),
            Self::Scp => format!(
                "scp -o ControlPath={} {}:{} {}",
                socket_path, user_host, fwd_path, local_path
            ),
            // Windows: use 'type' via SSH channel (binary-safe through separate channel)
            Self::Cat => format!(
                "ssh -S {} {} \"type {}\" > {}",
                socket_path, user_host, quote_remote_path(remote_path), local_path
            ),
            // Windows: use powershell to base64 encode, decode locally
            Self::Base64 => format!(
                "ssh -S {} {} \"powershell -c [Convert]::ToBase64String([IO.File]::ReadAllBytes('{}'))\" | base64 -d > {}",
                socket_path, user_host, remote_path.replace('\'', "''"), local_path
            ),
        }
    }
}

pub struct FallbackChain {
    methods: Vec<TransferMethod>,
}

impl FallbackChain {
    pub fn from_strings(methods: &[&str]) -> Self {
        Self {
            methods: methods
                .iter()
                .filter_map(|s| TransferMethod::from_str(s))
                .collect(),
        }
    }

    pub fn methods(&self) -> &[TransferMethod] {
        &self.methods
    }
}

impl Default for FallbackChain {
    fn default() -> Self {
        Self {
            methods: vec![
                TransferMethod::Sftp,
                TransferMethod::Scp,
                TransferMethod::Cat,
                TransferMethod::Base64,
            ],
        }
    }
}

pub enum TransferResult {
    Success { method: TransferMethod, bytes: u64 },
    Failed { method: TransferMethod, error: String },
}

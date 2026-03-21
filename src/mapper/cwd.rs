use crate::target_os::TargetOS;

pub struct CwdTracker {
    cwd: String,
    target_os: TargetOS,
}

impl CwdTracker {
    pub fn new(target_os: TargetOS) -> Self {
        Self {
            cwd: String::new(),
            target_os,
        }
    }

    pub fn current(&self) -> &str {
        if self.cwd.is_empty() {
            match self.target_os {
                TargetOS::Windows => "",
                _ => "/",
            }
        } else {
            &self.cwd
        }
    }

    pub fn set_target_os(&mut self, os: TargetOS) {
        self.target_os = os;
    }

    pub fn update_from_pwd(&mut self, output: &str) {
        let path = output.trim();
        if path.is_empty() {
            return;
        }
        // Unix absolute path
        if path.starts_with('/') {
            self.cwd = path.to_string();
            return;
        }
        // Windows absolute path: X:\ or X:/
        if path.len() >= 3 {
            let mut chars = path.chars();
            let drive = chars.next().unwrap_or(' ');
            let colon = chars.next().unwrap_or(' ');
            let sep = chars.next().unwrap_or(' ');
            if drive.is_ascii_alphabetic() && colon == ':' && (sep == '\\' || sep == '/') {
                self.cwd = path.to_string();
            }
        }
    }

    pub fn update_from_cd(&mut self, command: &str) {
        let target = command.trim().strip_prefix("cd").unwrap_or("").trim();
        if target.is_empty() || target == "~" {
            return;
        }
        // Unix absolute path
        if target.starts_with('/') {
            self.cwd = target.to_string();
            return;
        }
        // Windows absolute path: drive letter pattern X:\ or X:/
        if target.len() >= 2 {
            let mut chars = target.chars();
            let drive = chars.next().unwrap_or(' ');
            let colon = chars.next().unwrap_or(' ');
            if drive.is_ascii_alphabetic() && colon == ':' {
                self.cwd = target.to_string();
                return;
            }
        }
        if target == ".." {
            let sep = self.target_os.path_separator();
            let cwd = self.cwd.trim_end_matches(sep).trim_end_matches('/').trim_end_matches('\\');
            if let Some(pos) = cwd.rfind(sep).or_else(|| cwd.rfind('/')) {
                self.cwd = cwd[..pos].to_string();
                if self.cwd.is_empty() {
                    // Preserve root
                    self.cwd = sep.to_string();
                }
            }
            return;
        }
        let sep = self.target_os.path_separator();
        let base = self.cwd.trim_end_matches(sep).trim_end_matches('/').trim_end_matches('\\');
        self.cwd = format!("{}{}{}", base, sep, target);
    }
}

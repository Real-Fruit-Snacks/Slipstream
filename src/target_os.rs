#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetOS {
    Unix,
    Windows,
    Unknown,
}

impl TargetOS {
    pub fn detect(output: &str) -> Option<Self> {
        if output.contains("Microsoft") || output.contains("Windows")
            || output.contains("cmd.exe") || output.contains("PS C:\\")
            || output.contains("C:\\Users") || output.contains("C:\\Windows") {
            Some(Self::Windows)
        } else if output.contains("$ ") || output.contains("# ")
            || output.contains("bash") || output.contains("/home/")
            || output.contains("Linux") {
            Some(Self::Unix)
        } else {
            None
        }
    }

    pub fn path_separator(&self) -> char {
        match self { Self::Windows => '\\', _ => '/' }
    }

    pub fn is_windows(&self) -> bool { *self == Self::Windows }
    pub fn is_unix(&self) -> bool { *self == Self::Unix }
}

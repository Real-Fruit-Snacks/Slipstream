/// Boundary events detected from PROMPT_COMMAND markers.
#[derive(Debug, PartialEq)]
pub enum BoundaryEvent {
    PromptReady { exit_code: i32 },
}

/// Detects and strips OSC 133 boundary markers injected via PROMPT_COMMAND.
pub struct BoundaryDetector;

impl BoundaryDetector {
    pub fn new() -> Self {
        BoundaryDetector
    }

    /// Parse an OSC 133 slipstream marker from `data`.
    /// Looks for `\x1b]133;A;slipstream=<N>\x07`.
    pub fn parse_marker(&self, data: &str) -> Option<BoundaryEvent> {
        let prefix = "\x1b]133;A;slipstream=";
        let start = data.find(prefix)?;
        let after = &data[start + prefix.len()..];
        let end = after.find('\x07')?;
        let code_str = &after[..end];
        let exit_code: i32 = code_str.parse().ok()?;
        Some(BoundaryEvent::PromptReady { exit_code })
    }

    /// Remove the OSC 133 slipstream marker from `data`, returning cleaned output.
    pub fn strip_marker(data: &str) -> String {
        let prefix = "\x1b]133;A;slipstream=";
        if let Some(start) = data.find(prefix) {
            let before = &data[..start];
            let after_prefix = &data[start + prefix.len()..];
            if let Some(bel_pos) = after_prefix.find('\x07') {
                let after = &after_prefix[bel_pos + 1..];
                return format!("{}{}", before, after);
            }
        }
        data.to_string()
    }

    /// Return bash PROMPT_COMMAND injection string.
    pub fn bash_injection() -> String {
        r#"PROMPT_COMMAND='printf "\033]133;A;slipstream=%s\007" "$?"; '"${PROMPT_COMMAND}"#
            .to_string()
    }

    /// Return zsh precmd injection string.
    pub fn zsh_injection() -> String {
        r#"precmd() { printf "\033]133;A;slipstream=%s\007" "$?"; }"#.to_string()
    }

    /// Return fish fish_prompt injection string.
    pub fn fish_injection() -> String {
        r#"function fish_prompt; printf "\033]133;A;slipstream=%s\007" $status; end"#.to_string()
    }

    /// Return PowerShell prompt injection string.
    pub fn powershell_injection() -> String {
        r#"function prompt { $e = $LASTEXITCODE; Write-Host -NoNewline "$([char]27)]133;A;slipstream=$e$([char]7)"; "PS $($PWD.Path)> " }"#.to_string()
    }
}

impl Default for BoundaryDetector {
    fn default() -> Self {
        Self::new()
    }
}

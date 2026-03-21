/// Terminal operating mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalMode {
    Cooked,
    Raw,
}

/// Tracks the current terminal mode.
pub struct ModeTracker {
    mode: TerminalMode,
}

impl ModeTracker {
    pub fn new() -> Self {
        ModeTracker {
            mode: TerminalMode::Cooked,
        }
    }

    pub fn current(&self) -> TerminalMode {
        self.mode
    }

    pub fn set(&mut self, mode: TerminalMode) {
        self.mode = mode;
    }

    /// Update mode based on the ICANON termios flag.
    /// `icanon = true` → Cooked, `icanon = false` → Raw.
    pub fn update_from_termios_flags(&mut self, icanon: bool) {
        self.mode = if icanon {
            TerminalMode::Cooked
        } else {
            TerminalMode::Raw
        };
    }
}

impl Default for ModeTracker {
    fn default() -> Self {
        Self::new()
    }
}

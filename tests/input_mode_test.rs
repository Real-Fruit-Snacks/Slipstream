use slipstream::input::mode::{ModeTracker, TerminalMode};

#[test]
fn test_default_mode_is_cooked() {
    let tracker = ModeTracker::new();
    assert_eq!(tracker.current(), TerminalMode::Cooked);
}

#[test]
fn test_update_to_raw() {
    let mut tracker = ModeTracker::new();
    tracker.update_from_termios_flags(false);
    assert_eq!(tracker.current(), TerminalMode::Raw);
}

#[test]
fn test_update_back_to_cooked() {
    let mut tracker = ModeTracker::new();
    tracker.update_from_termios_flags(false);
    tracker.update_from_termios_flags(true);
    assert_eq!(tracker.current(), TerminalMode::Cooked);
}

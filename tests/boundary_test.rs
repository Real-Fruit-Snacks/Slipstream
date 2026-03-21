use slipstream::logging::boundary::{BoundaryDetector, BoundaryEvent};

#[test]
fn test_detect_osc_marker() {
    let detector = BoundaryDetector::new();
    let line = "\x1b]133;A;slipstream=0\x07";
    let event = detector.parse_marker(line);
    assert!(matches!(event, Some(BoundaryEvent::PromptReady { exit_code: 0 })));
}

#[test]
fn test_detect_nonzero_exit() {
    let detector = BoundaryDetector::new();
    let line = "\x1b]133;A;slipstream=1\x07";
    let event = detector.parse_marker(line);
    assert!(matches!(event, Some(BoundaryEvent::PromptReady { exit_code: 1 })));
}

#[test]
fn test_no_marker_in_normal_output() {
    let detector = BoundaryDetector::new();
    assert!(detector.parse_marker("total 128\ndrwxr-xr-x 2 root root").is_none());
}

#[test]
fn test_bash_injection() {
    let inj = BoundaryDetector::bash_injection();
    assert!(inj.contains("PROMPT_COMMAND"));
    assert!(inj.contains("133;A;slipstream="));
}

#[test]
fn test_strip_marker() {
    let data = "output\x1b]133;A;slipstream=0\x07prompt$";
    let stripped = BoundaryDetector::strip_marker(data);
    assert!(!stripped.contains("133;A;slipstream"));
    assert!(stripped.contains("output"));
    assert!(stripped.contains("prompt$"));
}

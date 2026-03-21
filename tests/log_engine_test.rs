use slipstream::logging::engine::LogEngine;
use tempfile::TempDir;

#[test]
fn test_log_session_start() {
    let tmp = TempDir::new().unwrap();
    let mut engine = LogEngine::new(tmp.path().to_path_buf(), true);
    engine.log_session_start("user", "victim01", "10.10.10.5");
    let content = std::fs::read_to_string(engine.session_log_path()).unwrap();
    assert!(content.contains("session start"));
    assert!(content.contains("victim01"));
}

#[test]
fn test_log_command_creates_file() {
    let tmp = TempDir::new().unwrap();
    let mut engine = LogEngine::new(tmp.path().to_path_buf(), true);
    engine.start_command("whoami");
    engine.append_output("root\n");
    engine.end_command(0);
    let files: Vec<_> = std::fs::read_dir(tmp.path()).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().map(|s| s.contains("whoami")).unwrap_or(false))
        .collect();
    assert_eq!(files.len(), 1);
}

#[test]
fn test_log_event() {
    let tmp = TempDir::new().unwrap();
    let mut engine = LogEngine::new(tmp.path().to_path_buf(), true);
    engine.log_event("!upload linpeas.sh /tmp/ → success");
    let content = std::fs::read_to_string(engine.session_log_path()).unwrap();
    assert!(content.contains("!upload"));
}

#[test]
fn test_continuous_logging_fallback() {
    let tmp = TempDir::new().unwrap();
    let mut engine = LogEngine::new(tmp.path().to_path_buf(), false);
    engine.append_continuous_output("some output\n");
    let path = tmp.path().join("session_continuous.log");
    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("some output"));
}

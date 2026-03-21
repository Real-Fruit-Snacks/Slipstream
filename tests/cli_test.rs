use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_slipstream_help() {
    Command::cargo_bin("slipstream")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SSH wrapper"));
}

#[test]
fn test_slipstream_clean_no_data() {
    Command::cargo_bin("slipstream")
        .unwrap()
        .arg("clean")
        .assert()
        .success();
}

#[test]
fn test_slipstream_ssh_no_args() {
    // Should fail because no target specified
    Command::cargo_bin("slipstream")
        .unwrap()
        .arg("ssh")
        .assert()
        .failure();
}

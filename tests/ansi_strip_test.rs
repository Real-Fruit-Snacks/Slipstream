use slipstream::mapper::parser::OutputParser;

#[test]
fn test_strip_ansi_from_ls_output() {
    let colored = "\x1b[0m\x1b[01;34mdir1\x1b[0m  \x1b[01;32mscript.sh\x1b[0m  file.txt\n";
    let stripped = OutputParser::strip_ansi(colored);
    assert!(!stripped.contains("\x1b["));
    assert!(stripped.contains("dir1"));
    assert!(stripped.contains("script.sh"));
    assert!(stripped.contains("file.txt"));
}

#[test]
fn test_strip_ansi_plain_text_unchanged() {
    let plain = "hello world\n";
    let stripped = OutputParser::strip_ansi(plain);
    assert_eq!(stripped, plain);
}

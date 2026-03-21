use slipstream::input::line_buffer::LineBuffer;

#[test]
fn test_empty_buffer() {
    let buf = LineBuffer::new();
    assert!(buf.is_empty());
    assert_eq!(buf.content(), "");
}

#[test]
fn test_push_characters() {
    let mut buf = LineBuffer::new();
    buf.push('h');
    buf.push('i');
    assert!(!buf.is_empty());
    assert_eq!(buf.content(), "hi");
}

#[test]
fn test_backspace() {
    let mut buf = LineBuffer::new();
    buf.push('a');
    buf.push('b');
    buf.backspace();
    assert_eq!(buf.content(), "a");
}

#[test]
fn test_backspace_on_empty() {
    let mut buf = LineBuffer::new();
    buf.backspace(); // must not panic
    assert!(buf.is_empty());
}

#[test]
fn test_clear() {
    let mut buf = LineBuffer::new();
    buf.push('x');
    buf.clear();
    assert!(buf.is_empty());
    assert_eq!(buf.content(), "");
}

#[test]
fn test_take_resets_buffer() {
    let mut buf = LineBuffer::new();
    buf.push('h');
    buf.push('e');
    buf.push('y');
    let s = buf.take();
    assert_eq!(s, "hey");
    assert!(buf.is_empty());
}

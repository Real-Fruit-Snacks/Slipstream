/// Simple keystroke buffer for accumulating input characters.
pub struct LineBuffer {
    buf: Vec<char>,
}

impl LineBuffer {
    pub fn new() -> Self {
        LineBuffer { buf: Vec::new() }
    }

    pub fn push(&mut self, c: char) {
        self.buf.push(c);
    }

    /// Remove the last character. No-op if empty.
    pub fn backspace(&mut self) {
        self.buf.pop();
    }

    pub fn content(&self) -> String {
        self.buf.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Return the current content and reset the buffer.
    pub fn take(&mut self) -> String {
        let s = self.content();
        self.clear();
        s
    }
}

impl Default for LineBuffer {
    fn default() -> Self {
        Self::new()
    }
}

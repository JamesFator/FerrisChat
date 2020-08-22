use std::io::{Result, Write};

/// Writer implementation allowing us to write to string instead of file
pub struct StringWriter {
    buf: Vec<u8>,
}

impl StringWriter {
    pub fn new() -> StringWriter {
        StringWriter {
            buf: Vec::with_capacity(8 * 1024),
        }
    }

    pub fn to_string(self) -> String {
        if let Ok(s) = String::from_utf8(self.buf) {
            s
        } else {
            String::new()
        }
    }
}

impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for b in buf {
            self.buf.push(*b);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

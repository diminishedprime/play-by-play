//! Lets bat, which writes text, write into the byte writers notes write into.

use std::fmt;
use std::io::Write;

/// `W` taking text; a failed write is a [`fmt::Error`].
pub struct Text<W: Write>(pub W);

impl<W: Write> fmt::Write for Text<W> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.write_all(text.as_bytes()).or(Err(fmt::Error))
    }
}

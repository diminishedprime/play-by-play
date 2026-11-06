//! Draws a live program's lines into its follower's log, redrawing the block in
//! place each time.

use std::io::Write;

use crate::error::AppResult;

/// What was drawn last, so the next block can replace it.
pub struct Drawn {
    frame: Vec<u8>,
    lines: usize,
    columns: u16,
}

impl Drawn {
    /// Starts with nothing drawn, at `columns` wide.
    #[must_use]
    pub fn new(columns: u16) -> Drawn {
        Drawn {
            frame: Vec::new(),
            lines: 0,
            columns,
        }
    }

    /// Replaces the last block in `live` with `lines`, each cut to `columns`.
    /// A new width clears the whole screen first.
    ///
    /// # Errors
    ///
    /// When `live` can't be written.
    pub fn draw(&mut self, live: &mut impl Write, lines: &[String], columns: u16) -> AppResult {
        self.frame.clear();
        if self.columns != columns {
            self.frame.extend_from_slice(b"\x1b[H\x1b[2J");
        } else if self.lines > 0 {
            write!(self.frame, "\x1b[{}F\x1b[J", self.lines)?;
        }
        self.lines = lines.len();
        self.columns = columns;
        for line in lines {
            self.frame.extend_from_slice(
                console::truncate_str(line, usize::from(columns), "").as_bytes(),
            );
            self.frame.push(b'\n');
        }
        live.write_all(&self.frame)?;
        Ok(())
    }
}

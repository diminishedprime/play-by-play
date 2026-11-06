//! Where a command's output goes.

use std::cell::Cell;
use std::fs::File;
use std::io::Write;

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::AppResult;

/// Splits a command's output between Claude and the viewer.
pub struct Router<'a> {
    log: File,
    claimed: &'a Cell<bool>,
}

impl<'a> Router<'a> {
    /// A router into `log`, until the command's program sets `claimed`.
    #[must_use]
    pub fn new(log: File, claimed: &'a Cell<bool>) -> Router<'a> {
        Router { log, claimed }
    }

    /// Copies everything `output` produces to `claude` as it arrives, and to the
    /// log until the program claims the viewer, so the viewer doesn't get both its
    /// plain lines and its live block.
    ///
    /// # Errors
    ///
    /// When `output` can't be read, or `claude` or the log written.
    pub async fn copy(
        mut self,
        mut output: impl AsyncRead + Unpin,
        mut claude: impl Write,
    ) -> AppResult {
        let mut chunk = [0; 8192];
        loop {
            let read = output.read(&mut chunk).await?;
            if read == 0 {
                return Ok(());
            }
            claude.write_all(&chunk[..read])?;
            claude.flush()?;
            if !self.claimed.get() {
                self.log.write_all(&chunk[..read])?;
            }
        }
    }
}

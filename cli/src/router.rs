//! Where a command's output goes.

use std::fs::File;
use std::io::Write;

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::AppResult;

/// Copies a command's output to both Claude and the viewer.
pub struct Router {
    log: File,
}

impl Router {
    /// A router into `log`.
    #[must_use]
    pub fn new(log: File) -> Router {
        Router { log }
    }

    /// Copies everything `output` produces to `claude` and to the log as it
    /// arrives, whether or not the program claimed the viewer, since its live
    /// block goes to its follower instead.
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
            self.log.write_all(&chunk[..read])?;
        }
    }
}

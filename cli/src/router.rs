//! Where a command's output goes.

use std::fs::File;
use std::io::{Read, Write};

use crate::channel::Channel;
use crate::error::AppResult;

/// Splits a command's output between Claude and the viewer.
pub struct Router {
    log: File,
    channel: Option<Channel>,
}

impl Router {
    /// A router into `log`, which the command's program can claim through
    /// `channel`.
    #[must_use]
    pub fn new(_log: File, _channel: Option<Channel>) -> Router {
        todo!()
    }

    /// Copies everything `output` produces to `claude` as it arrives, and to the
    /// log until the program claims the viewer, so the viewer doesn't get both its
    /// plain lines and its live block.
    ///
    /// # Errors
    ///
    /// When `output` can't be read, or `claude` or the log written.
    pub fn copy(self, _output: impl Read, _claude: impl Write) -> AppResult {
        todo!()
    }
}

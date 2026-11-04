//! Whether someone is watching: a recorded pid that's still running, not just a
//! leftover file.

use crate::error::AppResult;
use crate::log::Log;

/// A `play-by-play watch` following a workspace's log right now.
pub struct Viewer {
    columns: u16,
}

impl Viewer {
    /// The viewer following `log`, or `None` when no pid is recorded or it isn't
    /// running.
    ///
    /// # Errors
    ///
    /// When the pid or width file exists but can't be read.
    pub fn watching(_log: &Log) -> AppResult<Option<Viewer>> {
        todo!("read the viewer's pid and check it's alive, then read its width")
    }

    /// The width as of [`Viewer::watching`]; later changes reach the program over
    /// its socket.
    #[must_use]
    pub fn columns(&self) -> u16 {
        todo!()
    }
}

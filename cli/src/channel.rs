//! The socket a command's program connects to, to claim the viewer's terminal and
//! hear its width.

use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use crate::error::AppResult;

/// The socket `play-by-play run` offers one command, only while a viewer is
/// watching; a command that never connects is left a plain command.
pub struct Channel {
    path: PathBuf,
    listener: UnixListener,
    program: Option<UnixStream>,
}

impl Channel {
    /// Offers a socket in `dir` for one command.
    ///
    /// # Errors
    ///
    /// When the socket can't be bound.
    pub fn open(_dir: &Path) -> AppResult<Channel> {
        todo!("bind a uniquely named socket, nonblocking")
    }

    /// Where the program connects, for [`play_by_play::env::SOCKET`].
    #[must_use]
    pub fn path(&self) -> &Path {
        todo!()
    }

    /// Whether the program has connected, which claims the viewer's terminal.
    ///
    /// # Errors
    ///
    /// When the socket can't be checked.
    pub fn claimed(&mut self) -> AppResult<bool> {
        todo!("accept a waiting connection, and send it the current width")
    }

    /// Tells the program the viewer's width changed.
    ///
    /// # Errors
    ///
    /// When the program can't be written to.
    pub fn send_columns(&mut self, _columns: u16) -> AppResult {
        todo!()
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        todo!("remove the socket file")
    }
}

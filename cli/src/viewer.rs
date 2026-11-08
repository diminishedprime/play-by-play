//! Whether someone is watching: a recorded pid that's still running, not just a
//! leftover file.

use std::fs;

use play_by_play::log::Log;
use rustix::process::{self, Pid};

use crate::error::AppResult;
use crate::log::Screen;

/// A `play-by-play watch` following a workspace's log right now.
pub struct Viewer {
    columns: u16,
    rows: u16,
}

impl Viewer {
    /// The viewer following `log`, or `None` when no pid is recorded or it isn't
    /// running.
    ///
    /// # Errors
    ///
    /// When the pid or width file exists but can't be read.
    pub fn watching(log: &Log) -> AppResult<Option<Viewer>> {
        Ok(fs::read_to_string(log.viewer_path())
            .ok()
            .and_then(|pid| pid.parse().ok())
            .and_then(Pid::from_raw)
            .filter(|pid| process::test_kill_process(*pid).is_ok())
            .map(|_| Viewer {
                columns: Screen::of(log).columns,
                rows: fs::read_to_string(log.rows_path())
                    .ok()
                    .and_then(|rows| rows.parse().ok())
                    .unwrap_or(u16::MAX),
            }))
    }

    /// The width as of [`Viewer::watching`]; later changes reach the program over
    /// its socket.
    #[must_use]
    pub fn columns(&self) -> u16 {
        self.columns
    }

    /// The height as of [`Viewer::watching`], or [`u16::MAX`] when no viewer has
    /// said; later changes reach the program over its socket.
    #[must_use]
    pub fn rows(&self) -> u16 {
        self.rows
    }
}

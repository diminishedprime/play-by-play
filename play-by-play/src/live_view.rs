use std::hash::Hash;

use indexmap::IndexMap;

use crate::error::AppResult;
use crate::play_by_play::PlayByPlay;
use crate::terminal::Terminal;

/// Shows a program's [`PlayByPlay`]: a plain line on stdout for each row that
/// changes, and, when someone is watching, a live block in the viewer's terminal.
pub struct LiveView<K> {
    terminal: Option<Terminal>,
    rows: IndexMap<K, Row>,
    scratch: String,
}

struct Row {
    line: String,
    good: usize,
}

impl<K: Eq + Hash + Clone> LiveView<K> {
    /// Connects to the viewer's terminal, when `play-by-play run` offered one.
    ///
    /// # Errors
    ///
    /// When `play-by-play run` offered a terminal that can't be reached.
    pub fn open() -> AppResult<LiveView<K>> {
        todo!("Terminal::connect, no rows yet")
    }

    /// Shows `play` as it is now: a plain line for each new or changed row, and
    /// the live block redrawn. A row that fails to render prints its error to
    /// stderr and keeps its last good text in the block, with the error after it.
    ///
    /// # Errors
    ///
    /// When stdout or the viewer's log can't be written; never for a row's own
    /// error.
    pub fn update<P: PlayByPlay<Key = K>>(&mut self, _play: &P) -> AppResult {
        todo!(
            "write each row into scratch, swap it into the row's line when it differs from line[..good], move_index it to where it came, truncate the rows not seen, print what changed, draw the block"
        )
    }
}

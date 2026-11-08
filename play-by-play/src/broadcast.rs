use std::env as process_env;
use std::future;
use std::hash::Hash;
use std::io::ErrorKind;
use std::path::Path;

use crate::board::Board;
use crate::env;
use crate::error::{AppError, AppResult};
use crate::screen::Screen;
use crate::terminal::Terminal;

/// Shows a program's [`Board`] live in the viewer's terminal, when someone is
/// watching.
pub struct Broadcast<K> {
    terminal: Option<Terminal>,
    screen: Screen<K>,
}

impl<K: Eq + Hash + Clone> Broadcast<K> {
    /// Connects to the viewer's terminal, when play-by-play offered one.
    ///
    /// # Errors
    ///
    /// When play-by-play offered a terminal that can't be reached.
    pub async fn open() -> AppResult<Broadcast<K>> {
        match process_env::var_os(env::SOCKET) {
            Some(socket) => Broadcast::connect(socket).await,
            None => Ok(Broadcast::offline()),
        }
    }

    /// Connects to the viewer's terminal listening at `socket`.
    ///
    /// # Errors
    ///
    /// When `socket` can't be reached, or closes before sending the viewer's
    /// width.
    pub async fn connect(socket: impl AsRef<Path>) -> AppResult<Broadcast<K>> {
        Ok(Broadcast {
            terminal: Some(Terminal::connect(socket).await?),
            screen: Screen::new(),
        })
    }

    /// A broadcast with nobody watching, so rendering does nothing.
    #[must_use]
    pub fn offline() -> Broadcast<K> {
        Broadcast {
            terminal: None,
            screen: Screen::new(),
        }
    }

    /// Waits until the viewer's terminal changes width, so the program can
    /// [`render`](Broadcast::render) at the new width. With nobody
    /// watching, it never finishes. Safe to cancel, as in a `select!` beside the
    /// program's own work.
    ///
    /// # Errors
    ///
    /// When the viewer's terminal can't be read.
    pub async fn resized(&mut self) -> AppResult {
        match &mut self.terminal {
            Some(terminal) => terminal.resized().await,
            None => future::pending().await,
        }
    }

    /// Redraws `board` in the viewer's terminal as it is now. With nobody
    /// watching, it does nothing.
    ///
    /// # Errors
    ///
    /// When the viewer's terminal can't be written.
    ///
    /// # Panics
    ///
    /// When a line's [`show`](crate::Live::show) fails, which writing into a `String`
    /// never causes on its own.
    pub async fn render<B: Board<Key = K>>(&mut self, board: &B) -> AppResult {
        let Some(terminal) = &mut self.terminal else {
            return Ok(());
        };
        terminal.catch_up().await?;
        self.screen.refresh(
            board.lines(terminal.rows().saturating_sub(1)),
            terminal.columns(),
            terminal.spinner(),
        );
        match terminal.send(self.screen.block()).await {
            Err(AppError::Io(error))
                if matches!(
                    error.kind(),
                    ErrorKind::BrokenPipe | ErrorKind::ConnectionReset
                ) =>
            {
                self.terminal = None;
                Ok(())
            }
            drawn => drawn,
        }
    }
}

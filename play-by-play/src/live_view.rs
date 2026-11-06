use std::future;
use std::hash::Hash;
use std::io::{self, ErrorKind, Write as _};

use crate::error::{AppError, AppResult};
use crate::redraw::{Line, Redraw, Width};
use crate::screen::Screen;
use crate::terminal::Terminal;

/// Shows a program's [`Redraw`] content: each line's narration on stdout when
/// it changes, and, when someone is watching, a live block in the viewer's
/// terminal.
pub struct LiveView<K> {
    terminal: Option<Terminal>,
    screen: Screen<K>,
}

impl<K: Eq + Hash + Clone> LiveView<K> {
    /// Connects to the viewer's terminal, when `play-by-play run` offered one.
    ///
    /// # Errors
    ///
    /// When `play-by-play run` offered a terminal that can't be reached.
    pub async fn open() -> AppResult<LiveView<K>> {
        Ok(LiveView {
            terminal: Terminal::connect().await?,
            screen: Screen::new(),
        })
    }

    /// Waits until the viewer's terminal changes width, so the program can
    /// [`render`](LiveView::render) at the new width. With nobody
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

    /// Shows `lines` as they are now: each new narration on stdout, and the live
    /// block redrawn.
    ///
    /// # Errors
    ///
    /// When stdout or the viewer's log can't be written.
    ///
    /// # Panics
    ///
    /// When a line's [`draw`](Line::draw) or [`narrate`](Line::narrate) fails,
    /// which writing into a `String` never causes on its own.
    pub async fn render<R: Redraw<Key = K>>(&mut self, lines: &R) -> AppResult {
        self.refresh_screen(lines);
        self.print_narrations()?;
        let Some(terminal) = &mut self.terminal else {
            return Ok(());
        };
        match terminal.draw(self.screen.block()).await {
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

    fn refresh_screen<R: Redraw<Key = K>>(&mut self, lines: &R) {
        let width = self.terminal.as_ref().map_or(Width::Unbounded, |terminal| {
            Width::Columns(terminal.columns())
        });
        let mut frame = self.screen.frame();
        for line in lines.draw(width) {
            let (drawn, narration) = frame.line(line.key());
            line.draw(width, drawn).expect("a line draws into a String");
            line.narrate(narration)
                .expect("a line narrates into a String");
        }
    }

    fn print_narrations(&self) -> AppResult {
        let mut out = io::stdout().lock();
        self.screen
            .narrations()
            .try_for_each(|narration| writeln!(out, "{narration}"))?;
        Ok(())
    }
}

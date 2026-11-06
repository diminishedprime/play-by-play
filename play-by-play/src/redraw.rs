use std::fmt;
use std::hash::Hash;

/// Implement this for whatever your program wants to show live, then hand it to
/// [`LiveView::render`](crate::LiveView::render) whenever it changes. Each of
/// its lines is one line in the viewer's terminal.
pub trait Redraw {
    /// A unique identifier for a line. Subsequent lines with same Key will
    /// overwrite the previous.
    type Key: Eq + Hash + Clone;

    /// One of its lines.
    type Line<'a>: Line<Key = Self::Key>
    where
        Self: 'a;

    /// Yield every line, in the order they should be drawn. If a key you yielded
    /// last render isn't yielded this time, its line is cleared from the live
    /// view.
    fn draw(&self, width: Width) -> impl Iterator<Item = Self::Line<'_>>;
}

/// One line of a [`Redraw`]: the key it's known by from one render to the next,
/// and how it looks at a width.
pub trait Line {
    /// What the line is known by from one render to the next.
    type Key: Eq + Hash + Clone;

    /// The line's key.
    fn key(&self) -> &Self::Key;

    /// Writes the line's text into `out`. Keep it under [`Width`] `columns`.
    /// Everything above that will be truncated (unless it's
    /// [`Width::Unbounded`])
    ///
    /// # Errors
    ///
    /// When `out` can't be written.
    fn draw(&self, width: Width, out: &mut impl fmt::Write) -> fmt::Result;

    /// Writes what Claude should hear about the line into `out`, or nothing to
    /// stay quiet. It's printed to stdout whenever it differs from what this
    /// line last narrated.
    ///
    /// # Errors
    ///
    /// When `out` can't be written.
    fn narrate(&self, out: &mut impl fmt::Write) -> fmt::Result;
}

impl<T: Line> Line for &T {
    type Key = T::Key;

    fn key(&self) -> &T::Key {
        T::key(self)
    }

    fn draw(&self, width: Width, out: &mut impl fmt::Write) -> fmt::Result {
        T::draw(self, width, out)
    }

    fn narrate(&self, out: &mut impl fmt::Write) -> fmt::Result {
        T::narrate(self, out)
    }
}

/// How wide a line can be, so it can be formatted to fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    /// Someone is watching in a terminal this many columns wide.
    Columns(u16),
    /// No watchers, print output fon Claude.
    Unbounded,
}

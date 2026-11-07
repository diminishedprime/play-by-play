use std::fmt;
use std::hash::Hash;

use crate::painter::Painter;

/// The lines shown live in the viewer's terminal.
/// Pass it to [`Broadcast::render`](crate::Broadcast::render) whenever it
/// changes.
pub trait Board {
    /// A unique identifier for a line. Subsequent lines with same Key will
    /// overwrite the previous.
    type Key: Eq + Hash + Clone;

    /// One of its lines.
    type Line<'a>: Live<Key = Self::Key>
    where
        Self: 'a;

    /// Yield every line, in the order they should be drawn. If a key you yielded
    /// last render isn't yielded this time, its line is cleared from the live
    /// view.
    fn lines(&self) -> impl Iterator<Item = Self::Line<'_>>;
}

/// A [`Board`] line's key and its text in the viewer's terminal.
pub trait Live {
    /// What the line is known by from one render to the next.
    type Key: Eq + Hash + Clone;

    /// The line's key.
    fn key(&self) -> &Self::Key;

    /// Writes the line's text into `out`. Keep it within `columns`, the
    /// viewer's width; anything past that is cut off.
    ///
    /// # Errors
    ///
    /// When `out` can't be written.
    fn show(&self, columns: u16, out: &mut Painter<'_, impl fmt::Write>) -> fmt::Result;
}

impl<T: Live> Live for &T {
    type Key = T::Key;

    fn key(&self) -> &T::Key {
        T::key(self)
    }

    fn show(&self, columns: u16, out: &mut Painter<'_, impl fmt::Write>) -> fmt::Result {
        T::show(self, columns, out)
    }
}

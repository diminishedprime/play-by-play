use std::fmt::Display;
use std::hash::Hash;

/// Implement this for whatever your program wants to show live, then hand it to
/// [`LiveView::update`](crate::LiveView::update) whenever it changes. Each row is
/// one line in the viewer's terminal.
pub trait PlayByPlay {
    /// A unique identifier for a row. Subsequent rows with same Key will
    /// overwrite the previous.
    type Key: Eq + Hash + Clone;

    /// A row's text. Keep it under [`Width`] `rows`. Everything above that will
    /// be truncated (unless it's [`Width::Unbounded`])
    type Row<'a>: Display
    where
        Self: 'a;

    /// Why a row can't be shown right now. This will be printed to stderr.
    type Error: Display;

    /// Yield every row, in the order they should be drawn. If a key you yielded
    /// last update isn't yielded this time, its row is cleared from the live view.
    fn rows(
        &self,
        width: Width,
    ) -> impl Iterator<Item = (&Self::Key, Result<Self::Row<'_>, Self::Error>)>;

    /// What's printed to stdout when a row changes, or `None` to print nothing.
    fn plain<'a>(&'a self, _key: &'a Self::Key, row: &'a str) -> Option<impl Display + 'a> {
        Some(row)
    }
}

/// How wide a row can be, so it can be formatted to fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    /// Someone is watching in a terminal this many columns wide.
    Columns(u16),
    /// No watchers, print output fon Claude.
    Unbounded,
}

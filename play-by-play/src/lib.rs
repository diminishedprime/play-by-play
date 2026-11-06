//! Lets a program show a live view of its progress to someone following Claude's
//! commands with `play-by-play watch`, while Claude, CI and tests still get plain
//! lines.
//!
//! Implement [`Redraw`] for what the program shows, open a [`LiveView`], and
//! render it whenever that changes. With nobody watching, only the plain lines
//! print.

pub mod env;
mod error;
mod live_view;
mod redraw;
mod screen;
mod terminal;

pub use error::{AppError, AppResult};
pub use live_view::LiveView;
pub use redraw::{Line, Redraw, Width};

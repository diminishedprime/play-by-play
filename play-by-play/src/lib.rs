//! Lets a program show a live view of its progress to someone following Claude's
//! commands with `play-by-play watch`. Its stdout still goes to Claude, CI and
//! tests as usual.
//!
//! Implement [`Board`] for what the program shows, open a [`Broadcast`], and
//! render it whenever that changes.

mod board;
mod broadcast;
pub mod env;
mod error;
/// The messages a program and play-by-play send each other over the
/// socket.
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/play_by_play.v1.rs"));
}
mod screen;
mod terminal;

pub use board::{Board, Live};
pub use broadcast::Broadcast;
pub use error::{AppError, AppResult};

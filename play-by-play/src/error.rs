use std::io;
use std::num::TryFromIntError;

/// Why a [`Broadcast`](crate::Broadcast) couldn't show something.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// stdout, the viewer's log, or the socket failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// play-by-play sent a message that isn't one of
    /// [`proto`](crate::proto)'s.
    #[error("play-by-play sent a message that can't be read: {0}")]
    Decode(#[from] prost::DecodeError),
    /// A message couldn't be written.
    #[error("play-by-play couldn't write a message: {0}")]
    Encode(#[from] prost::EncodeError),
    /// A width or a message is bigger than the socket's messages can hold.
    #[error("a width or a message is too big for play-by-play's socket: {0}")]
    Overflow(#[from] TryFromIntError),
    /// A play-by-play variable is set to something that invalid.
    #[error("{0} is set to {1:?}, which play-by-play can't use")]
    Variable(&'static str, String),
    /// A name that isn't one of [`Color`](crate::Color)'s.
    #[error("{0:?} isn't a color play-by-play paints in")]
    Color(String),
}

/// The result of this crate's calls, failing with [`AppError`].
pub type AppResult<T = ()> = Result<T, AppError>;

use std::io;

/// Why a [`LiveView`](crate::LiveView) couldn't show something. A row failing to
/// render isn't one of these: that's the program's own error, shown in the block.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// stdout, the viewer's log, or the socket failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A play-by-play variable is set to something that invalid.
    #[error("{0} is set to {1:?}, which play-by-play can't use")]
    Variable(&'static str, String),
}

/// The result of this crate's calls, failing with [`AppError`].
pub type AppResult<T = ()> = Result<T, AppError>;

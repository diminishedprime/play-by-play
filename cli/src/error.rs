//! Failures of `play-by-play`, and a wrapped command's failure, which exits with
//! the command's own code.

use std::io;
use std::process::{ExitCode, Termination};

/// Why `play-by-play` couldn't run or watch.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The log, the viewer's files, the socket, or the shell failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A tool's input or result on stdin isn't the JSON its kind expects.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// The log's folder can't be watched.
    #[error(transparent)]
    Notify(#[from] notify::Error),
    /// A code block can't be highlighted.
    #[error(transparent)]
    Highlight(#[from] bat::error::Error),
    /// The viewer's terminal can't be asked its size.
    #[error(transparent)]
    Terminal(#[from] rustix::io::Errno),
    /// The wrapped command exited with this code, so play-by-play does too.
    #[error("the command exited with {0}")]
    Exited(u8),
}

/// The result of this crate's calls, failing with [`AppError`].
pub type AppResult<T = ()> = Result<T, AppError>;

/// How `play-by-play` exits: as its success says, or failing with its error
/// printed.
pub struct AppExit<T = ()>(AppResult<T>);

impl<T> From<AppResult<T>> for AppExit<T> {
    fn from(result: AppResult<T>) -> AppExit<T> {
        AppExit(result)
    }
}

impl<T: Termination> Termination for AppExit<T> {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(done) => done.report(),
            Err(AppError::Exited(code)) => ExitCode::from(code),
            Err(error) => {
                eprintln!("play-by-play: {error}");
                ExitCode::FAILURE
            }
        }
    }
}

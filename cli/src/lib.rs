//! The `play-by-play` command: `run` wraps each of Claude's commands, `watch`
//! follows them.

pub mod channel;
pub mod cmd;
pub mod doctor;
pub mod error;
pub mod log;
pub mod markdown;
pub mod note;
pub mod router;
pub mod run;
pub mod text;
pub mod viewer;
pub mod watch;

use std::env;
use std::path::PathBuf;

use crate::cmd::Action::{Doctor, NoteHeader, NoteResult, Run, Unwrap, Watch, WatchInTmux};
use crate::cmd::Cli;
use crate::error::AppResult;
use crate::log::Log;

/// For `run`, the exit code is the wrapped command's own, so Claude sees whether
/// its command failed rather than whether play-by-play did.
///
/// # Errors
///
/// When play-by-play itself fails.
pub fn main(cli: Cli) -> AppResult {
    let dir = env::temp_dir().join("play-by-play");
    let root = env::var_os("CLAUDE_PROJECT_DIR")
        .map_or_else(env::current_dir, |root| Ok(PathBuf::from(root)))?;
    let log = Log::for_workspace(&dir, &root);
    match cli.action() {
        NoteHeader(kind) => note::header(&log, &kind),
        NoteResult(kind, color) => note::result(&log, &kind, color),
        Watch => watch::watch(&log),
        WatchInTmux => watch::in_tmux(&log),
        Run { command } => run::run(&log, &command),
        Unwrap => run::unwrap(),
        Doctor => doctor::doctor(&dir),
    }
}

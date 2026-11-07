//! The `play-by-play` command: `-c` wraps each of Claude's commands, `watch`
//! follows them.

pub mod channel;
pub mod cmd;
pub mod doctor;
pub mod drawn;
pub mod error;
pub mod extension;
pub mod hook;
pub mod log;
pub mod markdown;
pub mod note;
pub mod router;
pub mod run;
pub mod text;
pub mod viewer;
pub mod watch;

use std::env;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use play_by_play::log::Log;

use crate::cmd::Action::{
    Doctor, Follow, HookSessionStart, NoteHeader, NoteResult, Run, Unwrap, Watch, WatchInTmux,
};
use crate::cmd::Cli;
use crate::error::AppResult;

/// For `-c`, the exit code is the wrapped command's own, so Claude sees whether
/// its command failed rather than whether play-by-play did.
///
/// # Errors
///
/// When play-by-play itself fails.
pub fn main(cli: Cli) -> AppResult<ExitCode> {
    let dir = Log::folder();
    let root = env::var_os("CLAUDE_PROJECT_DIR")
        .map_or_else(env::current_dir, |root| Ok(PathBuf::from(root)))?;
    let log = Log::for_workspace(&dir, &root);
    let done = match cli.action() {
        Run { command } => return run::run(&log, &dir, &command),
        NoteHeader(kind) => note::header(&log, &kind),
        NoteResult(kind, color) => note::result(&log, &kind, color),
        Watch => watch::watch(&log),
        WatchInTmux => watch::in_tmux(&log),
        Follow(pid) => watch::follow(&log, pid),
        Unwrap => run::unwrap(),
        Doctor => doctor::doctor(&dir),
        HookSessionStart => hook::session_start(&mut io::stdout()),
    };
    done.map(|()| ExitCode::SUCCESS)
}

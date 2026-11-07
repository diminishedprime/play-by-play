//! The Claude Code hooks `hook` answers.

use clap::Subcommand;

/// The hook event `hook` prints the output for.
#[derive(Subcommand)]
pub enum HookEvent {
    /// Tells Claude the person is watching through play-by-play.
    SessionStart,
}

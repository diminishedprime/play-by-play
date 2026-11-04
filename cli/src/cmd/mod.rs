//! `play-by-play`'s arguments; the doc comments here are its `--help` text.

pub mod note;

use clap::{Parser, Subcommand};

use crate::cmd::note::{NoteKind, NotePhase};
use crate::log::Color;

/// Runs Claude's commands so a viewer can follow them live
#[derive(Parser)]
#[command(
    name = "play-by-play",
    version,
    args_conflicts_with_subcommands = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Runs this command, as `run --command` does.
    #[arg(short, long)]
    pub command: Option<String>,
    /// What to do.
    #[command(subcommand)]
    pub subcommand: Option<Command>,
}

/// `play-by-play`'s subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Runs a command in zsh for Claude, showing it to the viewer.
    Run {
        /// The command to run.
        #[arg(long)]
        command: String,
    },
    /// Prints the command a `run` wrapper on stdin hold.
    ///
    /// This is useful with a `PreToolUse` hook on Bash that handles custom
    /// permissions:
    ///
    /// ```json
    /// "PreToolUse": [
    ///   {
    ///     "matcher": "Bash",
    ///     "hooks": [
    ///       {
    ///         "type": "command",
    ///         "command": "jq -j .tool_input.command | play-by-play unwrap | command_approver"
    ///       }
    ///     ]
    ///   }
    /// ]
    /// ```
    #[command(verbatim_doc_comment)]
    Unwrap,
    /// Follows the commands Claude runs in this workspace.
    Watch {
        /// Follows them inside the workspace's tmux session, starting it if
        /// needed.
        #[arg(long)]
        tmux: bool,
    },
    /// Checks that play-by-play is set up to work.
    Doctor,
    /// Shows the viewer a tool call that isn't a command.
    Note {
        /// Before or after the tool runs.
        #[command(subcommand)]
        phase: NotePhase,
    },
}

/// What `play-by-play` was asked to do, with `note`'s phase folded in.
pub enum Action {
    /// Run a command for Claude.
    Run {
        /// The command to run.
        command: String,
    },
    /// Print the command a `run` wrapper holds.
    Unwrap,
    /// Follow the workspace's log.
    Watch,
    /// Follow the workspace's log inside its tmux session.
    WatchInTmux,
    /// Check play-by-play's setup.
    Doctor,
    /// Show a tool call's header.
    NoteHeader(NoteKind),
    /// Show a tool call's result, closing in its header's color.
    NoteResult(NoteKind, Color),
}

impl From<Command> for Action {
    fn from(command: Command) -> Action {
        match command {
            Command::Run { command } => Action::Run { command },
            Command::Unwrap => Action::Unwrap,
            Command::Watch { tmux: false } => Action::Watch,
            Command::Watch { tmux: true } => Action::WatchInTmux,
            Command::Doctor => Action::Doctor,
            Command::Note {
                phase: NotePhase::Header { kind },
            } => Action::NoteHeader(kind),
            Command::Note {
                phase: NotePhase::Result { kind, color },
            } => Action::NoteResult(kind, color),
        }
    }
}

impl Cli {
    /// Parses the process's arguments.
    #[must_use]
    pub fn parse_args() -> Cli {
        Cli::parse()
    }

    /// What was asked for.
    ///
    /// # Panics
    ///
    /// Never: clap asks for `-c` or a subcommand before parsing succeeds.
    #[must_use]
    pub fn action(self) -> Action {
        match self.command {
            Some(command) => Action::Run { command },
            None => self
                .subcommand
                .expect("clap asks for -c or a subcommand")
                .into(),
        }
    }
}

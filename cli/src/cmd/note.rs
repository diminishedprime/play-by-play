//! The tool calls `note` shows.

use clap::Subcommand;

use crate::log::Color;

/// When in a tool call `note` runs.
#[derive(Subcommand)]
pub enum NotePhase {
    /// Before the tool runs: shows its header, from the tool's input on stdin.
    Header {
        /// The tool that was called.
        #[command(subcommand)]
        kind: NoteKind,
    },
    /// After the tool runs: shows its result, from the tool's result on stdin.
    Result {
        /// The color its header printed, which its closing line repeats.
        #[arg(long, value_enum)]
        color: Color,
        /// The tool that was called.
        #[command(subcommand)]
        kind: NoteKind,
    },
}

/// The tool calls `note` shows, each as JSON on stdin.
#[derive(Subcommand)]
pub enum NoteKind {
    /// A file Claude read.
    Read,
    /// A file Claude wrote.
    Write,
    /// An edit Claude made.
    Edit,
    /// A page Claude fetched.
    Fetch,
    /// A web search Claude ran.
    Search,
    /// A subagent Claude started.
    Agent,
}

impl NoteKind {
    /// How the viewer names this tool, as its header and closing line do.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            NoteKind::Read => "read",
            NoteKind::Write => "write",
            NoteKind::Edit => "edit",
            NoteKind::Fetch => "fetch",
            NoteKind::Search => "search",
            NoteKind::Agent => "agent",
        }
    }
}

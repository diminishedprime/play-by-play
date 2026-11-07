//! Tool calls that aren't commands, shown in the log. Each tool's input, and then
//! its result, arrive as the tool's own JSON on stdin.
//!
//! The types come from Claude Code's tool types, in
//! `plugin/.claude-plugin/types/claude-code-tools/index.d.ts`. Refresh it with:
//!
//! ```sh
//! just plugin-types
//! ```

pub mod agent;
pub mod edit;
pub mod fetch;
pub mod patch;
pub mod read;
pub mod search;
pub mod source_excerpt;
pub mod write;

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read, Write};

use play_by_play::Color;
use play_by_play::log::Log;
use serde::Deserialize;
use serde_json::Value;

use crate::cmd::note::NoteKind;
use crate::error::AppResult;
use crate::log::{CLEAR, ColorWheel, Painted, Screen};
use crate::note::agent::{AgentInput, AgentResult};
use crate::note::edit::{EditInput, EditResult};
use crate::note::fetch::{FetchInput, FetchResult};
use crate::note::read::{ReadInput, ReadResult};
use crate::note::search::{SearchInput, SearchResult};
use crate::note::write::{WriteInput, WriteResult};

/// Writes `kind`'s header into `log`, from the tool's input on stdin, and
/// prints the color it used for the result's closing line.
///
/// # Errors
///
/// When stdin can't be read or isn't the kind's input, or the log or stdout
/// can't be written.
pub fn header(log: &Log, kind: &NoteKind) -> AppResult {
    let mut json = String::new();
    io::stdin().read_to_string(&mut json)?;
    let color = ColorWheel::turn(log)?;
    match kind {
        NoteKind::Read => serde_json::from_str::<ReadInput>(&json)?.headed(log, color),
        NoteKind::Write => serde_json::from_str::<WriteInput>(&json)?.headed(log, color),
        NoteKind::Edit => serde_json::from_str::<EditInput>(&json)?.headed(log, color),
        NoteKind::Fetch => serde_json::from_str::<FetchInput>(&json)?.headed(log, color),
        NoteKind::Search => serde_json::from_str::<SearchInput>(&json)?.headed(log, color),
        NoteKind::Agent => serde_json::from_str::<AgentInput>(&json)?.headed(log, color),
    }?;
    writeln!(io::stdout(), "{color}")?;
    Ok(())
}

/// Writes `kind`'s result into `log`, from the tool's result on stdin, then
/// its closing line in `color`.
///
/// # Errors
///
/// When stdin can't be read or isn't the kind's result, or the log can't be
/// written.
pub fn result(log: &Log, kind: &NoteKind, color: Color) -> AppResult {
    let mut json = String::new();
    io::stdin().read_to_string(&mut json)?;
    match kind {
        NoteKind::Read => serde_json::from_str::<OpenResult<ReadResult>>(&json)?.appended(log),
        NoteKind::Write => serde_json::from_str::<WriteResult>(&json)?.appended(log),
        NoteKind::Edit => serde_json::from_str::<EditResult>(&json)?.appended(log),
        NoteKind::Fetch => serde_json::from_str::<FetchResult>(&json)?.appended(log),
        NoteKind::Search => serde_json::from_str::<SearchResult>(&json)?.appended(log),
        NoteKind::Agent => serde_json::from_str::<OpenResult<AgentResult>>(&json)?.appended(log),
    }?;
    writeln!(
        log.append()?,
        "{}",
        Painted {
            color,
            text: format_args!("{}: end", kind.name())
        }
    )?;
    Ok(())
}

/// A value Claude Code can add to: one known when the types were written, or
/// whatever came instead.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Open<'a, T> {
    /// A value the types know.
    Known(T),
    /// A value added since.
    Other(#[serde(borrow)] Cow<'a, str>),
}

/// A result Claude Code can add kinds to: one known when the types were written,
/// or the whole of whatever came instead.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum OpenResult<T> {
    /// A kind the types know.
    Known(T),
    /// A kind added since, as it came.
    Other(Value),
}

impl<T: Body> Body for OpenResult<T> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        match self {
            OpenResult::Known(known) => known.body(out, screen),
            OpenResult::Other(value) => {
                writeln!(out, "{value}")?;
                Ok(())
            }
        }
    }
}

/// What a tool call shows when it starts: a cleared screen and its header.
pub trait Header {
    /// Writes this header into `out` for `screen`, starting in `color`.
    ///
    /// # Errors
    ///
    /// When `out` can't be written, or this can't be shown.
    fn header(&self, out: &mut impl Write, color: Color, screen: &Screen) -> AppResult;

    /// Writes this header into `log`, in `color`.
    ///
    /// # Errors
    ///
    /// When the log can't be written, or this can't be shown.
    fn headed(&self, log: &Log, color: Color) -> AppResult {
        self.header(&mut log.append()?, color, &Screen::of(log))
    }
}

/// A header that's only a title, written whole in its color.
pub trait Title {
    /// Writes this title into `f`.
    ///
    /// # Errors
    ///
    /// When `f` can't be written.
    fn title(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

struct Titled<'a, T>(&'a T);

impl<T: Title> Display for Titled<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.title(f)
    }
}

impl<T: Title> Header for T {
    fn header(&self, out: &mut impl Write, color: Color, _screen: &Screen) -> AppResult {
        writeln!(
            out,
            "{CLEAR}{}",
            Painted {
                color,
                text: Titled(self)
            }
        )?;
        Ok(())
    }
}

/// What a tool call shows when it finishes, under its header.
pub trait Body {
    /// Writes this into `out`, fit to the viewer's `screen`.
    ///
    /// # Errors
    ///
    /// When `out` can't be written, or this can't be shown.
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult;

    /// Writes this into `log`, under what's already there.
    ///
    /// # Errors
    ///
    /// When the log can't be written, or this can't be shown.
    fn appended(&self, log: &Log) -> AppResult {
        self.body(&mut log.append()?, &Screen::of(log))
    }
}

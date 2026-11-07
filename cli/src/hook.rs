//! The output of the Claude Code hooks play-by-play's plugin runs it for.

use std::io::Write;

use serde_json::json;

use crate::error::AppResult;

/// What Claude is told about play-by-play when a session starts.
const CONTEXT: &str = include_str!("play-by-play.md");

/// Writes a `SessionStart` hook's output, giving Claude play-by-play's context.
///
/// # Errors
///
/// When the output can't be written.
pub fn session_start(out: &mut impl Write) -> AppResult {
    serde_json::to_writer(
        out,
        &json!({
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": CONTEXT,
            },
        }),
    )?;
    Ok(())
}

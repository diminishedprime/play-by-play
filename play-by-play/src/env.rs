//! The variables `play-by-play run` sets for a command, only while a viewer is
//! watching; with neither set, nobody is watching.

/// The descriptor that appends to the log the viewer follows, so what's written
/// to it appears in the viewer's terminal and never reaches Claude.
pub const FD: &str = "PLAY_BY_PLAY_FD";

/// The socket a program connects to: connecting claims the viewer's terminal,
/// and the viewer's width arrives on it whenever it changes.
pub const SOCKET: &str = "PLAY_BY_PLAY_SOCKET";

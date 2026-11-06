//! The variables play-by-play sets for a command, only while a viewer is
//! watching; with none set, nobody is watching.

/// The socket a program connects to: connecting claims the viewer's terminal,
/// and the viewer's width arrives on it whenever it changes.
pub const SOCKET: &str = "PLAY_BY_PLAY_SOCKET";

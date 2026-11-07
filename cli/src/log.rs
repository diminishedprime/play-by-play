//! How the viewer's log is written: its terminal, colors, and escapes.

use std::fmt::{self, Display, Formatter, Write as _};
use std::fs;

use play_by_play::log::Log;
use play_by_play::{Color, Painter};

use crate::error::AppResult;

/// The viewer's terminal, as notes write for it.
pub struct Screen {
    /// How wide it is, or [`u16::MAX`] when no viewer has said.
    pub columns: u16,
    /// The code theme closest to its background, or bat's own default when no
    /// viewer has said.
    pub theme: Option<String>,
}

impl Screen {
    /// A screen with no width to wrap to and bat's default theme.
    pub const UNBOUNDED: Screen = Screen {
        columns: u16::MAX,
        theme: None,
    };

    /// What the viewer of `log` has said about its terminal so far.
    #[must_use]
    pub fn of(log: &Log) -> Screen {
        Screen {
            columns: fs::read_to_string(log.columns_path())
                .ok()
                .and_then(|columns| columns.parse().ok())
                .unwrap_or(u16::MAX),
            theme: fs::read_to_string(log.theme_path()).ok(),
        }
    }
}

/// Clears the viewer's screen and moves to its top, so each header starts a
/// fresh screen.
pub const CLEAR: &str = "\x1b[H\x1b[2J";

/// The colors the viewer's headers take turns in, from where the workspace
/// last left off.
pub struct ColorWheel(usize);

impl ColorWheel {
    const COLORS: [Color; 5] = [
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];

    /// The color the next header in `log` uses, turning the workspace's wheel
    /// on to the one after.
    ///
    /// # Errors
    ///
    /// When the folder can't be made or the wheel can't be saved.
    ///
    /// # Panics
    ///
    /// Never: the wheel turns forever.
    pub fn turn(log: &Log) -> AppResult<Color> {
        let path = log.color_path();
        let mut wheel = ColorWheel(
            fs::read_to_string(&path)
                .ok()
                .and_then(|at| at.parse().ok())
                .unwrap_or(0),
        );
        let color = wheel.next().expect("the wheel turns forever");
        fs::create_dir_all(log.dir())?;
        fs::write(&path, wheel.0.to_string())?;
        Ok(color)
    }
}

impl Iterator for ColorWheel {
    type Item = Color;

    fn next(&mut self) -> Option<Color> {
        let color = ColorWheel::COLORS[self.0 % ColorWheel::COLORS.len()];
        self.0 = (self.0 + 1) % ColorWheel::COLORS.len();
        Some(color)
    }
}

/// Text the viewer's terminal opens `url` from when it's clicked.
pub struct Link<'a> {
    /// Where the link goes.
    pub url: &'a str,
    /// What the link shows.
    pub text: &'a str,
}

impl Display for Link<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Painter::from(f).link(self.url, |out| out.blue(|out| out.write_str(self.text)))
    }
}

/// `text` painted whole in `color`.
pub struct Painted<T> {
    /// The color it's painted in.
    pub color: Color,
    /// What's painted.
    pub text: T,
}

impl<T: Display> Display for Painted<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Painter::from(f).paint(self.color, |out| write!(out, "{}", self.text))
    }
}

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::error::AppError;

/// Writes a line's text, and the escapes that style it, into `W`.
pub struct Painter<'a, W> {
    out: &'a mut W,
    marker: Marker,
    written: Marker,
    plain: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Marker {
    color: Option<Color>,
    bold: Flag<1, 22>,
    italic: Flag<3, 23>,
    underline: Flag<4, 24>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Flag<const ON: u8, const OFF: u8>(bool);

impl<const ON: u8, const OFF: u8> Flag<ON, OFF> {
    fn switch(self, to: Self, out: &mut impl fmt::Write) -> fmt::Result {
        match (self.0, to.0) {
            (false, true) => write!(out, "\x1b[{ON}m"),
            (true, false) => write!(out, "\x1b[{OFF}m"),
            _ => Ok(()),
        }
    }
}

/// A color text can be painted in, drawn as the viewer's theme shows it. It
/// displays as, and parses from, its lowercase name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// The theme's red.
    Red,
    /// The theme's green.
    Green,
    /// The theme's yellow.
    Yellow,
    /// The theme's blue.
    Blue,
    /// The theme's magenta.
    Magenta,
    /// The theme's cyan.
    Cyan,
    /// The theme's bright black.
    Gray,
}

impl Color {
    fn name(self) -> &'static str {
        match self {
            Color::Red => "red",
            Color::Green => "green",
            Color::Yellow => "yellow",
            Color::Blue => "blue",
            Color::Magenta => "magenta",
            Color::Cyan => "cyan",
            Color::Gray => "gray",
        }
    }

    fn escape(self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::Gray => "\x1b[90m",
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Color {
    type Err = AppError;

    fn from_str(name: &str) -> Result<Color, AppError> {
        match name {
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "gray" => Ok(Color::Gray),
            other => Err(AppError::Color(other.to_owned())),
        }
    }
}

impl<'a, W: fmt::Write> From<&'a mut W> for Painter<'a, W> {
    fn from(out: &'a mut W) -> Painter<'a, W> {
        Painter {
            out,
            marker: Marker::default(),
            written: Marker::default(),
            plain: false,
        }
    }
}

impl<'a, W: fmt::Write> Painter<'a, W> {
    /// Paints into `out` without any escapes, writing only the text, for
    /// output that isn't going to a terminal.
    pub fn plain(out: &'a mut W) -> Painter<'a, W> {
        Painter {
            plain: true,
            ..Painter::from(out)
        }
    }
}

impl<W: fmt::Write> Painter<'_, W> {
    /// Makes what `paint` writes an underlined link the viewer's terminal
    /// opens `url` from when it's clicked.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn link(&mut self, url: &str, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        if self.plain {
            return paint(self);
        }
        write!(self.out, "\x1b]8;;{url}\x1b\\")?;
        self.underline(paint)?;
        self.switch()?;
        self.out.write_str("\x1b]8;;\x1b\\")
    }

    /// Paints what `paint` writes in italics.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn italic(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.with(
            Marker {
                italic: Flag(true),
                ..self.marker
            },
            paint,
        )
    }

    /// Paints what `paint` writes underlined.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn underline(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.with(
            Marker {
                underline: Flag(true),
                ..self.marker
            },
            paint,
        )
    }

    /// Paints what `paint` writes in `color`.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn paint(
        &mut self,
        color: Color,
        paint: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        self.with(
            Marker {
                color: Some(color),
                ..self.marker
            },
            paint,
        )
    }

    /// Paints what `paint` writes in green.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn green(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.paint(Color::Green, paint)
    }

    /// Paints what `paint` writes in blue.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn blue(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.paint(Color::Blue, paint)
    }

    /// Paints what `paint` writes in cyan.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn cyan(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.paint(Color::Cyan, paint)
    }

    /// Paints what `paint` writes in bold.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn bold(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.with(
            Marker {
                bold: Flag(true),
                ..self.marker
            },
            paint,
        )
    }

    fn with(
        &mut self,
        marker: Marker,
        paint: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        let outer = self.marker;
        self.marker = marker;
        paint(self)?;
        self.marker = outer;
        if outer == Marker::default() {
            self.switch()?;
        }
        Ok(())
    }

    fn switch(&mut self) -> fmt::Result {
        let (from, to) = (self.written, self.marker);
        if self.plain || from == to {
            return Ok(());
        }
        from.bold.switch(to.bold, self.out)?;
        from.italic.switch(to.italic, self.out)?;
        from.underline.switch(to.underline, self.out)?;
        if from.color != to.color {
            self.out
                .write_str(to.color.map_or("\x1b[39m", Color::escape))?;
        }
        self.written = to;
        Ok(())
    }
}

impl<W: fmt::Write> fmt::Write for Painter<'_, W> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.switch()?;
        self.out.write_str(text)
    }
}

use std::fmt::{self, Display, Formatter, Write as _};
use std::str::FromStr;

use similar::{ChangeTag, TextDiff};
use unicode_width::UnicodeWidthStr;

use crate::error::AppError;
use crate::spinner::Spinner;

/// Writes a line's text, and the escapes that style it, into `W`.
pub struct Painter<'a, W> {
    out: Counted<'a, W>,
    spinners: Option<&'a mut Vec<usize>>,
    spinner: &'static str,
    marker: Marker,
    written: Marker,
    plain: bool,
    indent: usize,
    fresh: bool,
    columns: usize,
    column: usize,
    linking: Option<String>,
    reflow: Option<Vec<Run>>,
}

struct Run {
    text: String,
    marker: Marker,
    link: Option<String>,
}

struct Counted<'a, W> {
    out: &'a mut W,
    len: usize,
}

impl<W: fmt::Write> fmt::Write for Counted<'_, W> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.len += text.len();
        self.out.write_str(text)
    }
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
            out: Counted { out, len: 0 },
            spinners: None,
            spinner: Spinner::FRAMES[0],
            marker: Marker::default(),
            written: Marker::default(),
            plain: false,
            indent: 0,
            fresh: true,
            columns: usize::MAX,
            column: 0,
            linking: None,
            reflow: None,
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

    pub(crate) fn recording(
        out: &'a mut W,
        spinners: &'a mut Vec<usize>,
        spinner: &'static str,
        columns: u16,
    ) -> Painter<'a, W> {
        Painter {
            spinners: Some(spinners),
            spinner,
            columns: usize::from(columns),
            ..Painter::from(out)
        }
    }
}

impl<W: fmt::Write> Painter<'_, W> {
    /// Writes `text` in the current style, starting each of its lines that
    /// isn't blank at the current indent, which is never styled.
    ///
    /// # Errors
    ///
    /// When the underlying writer fails.
    pub fn write_str(&mut self, text: &str) -> fmt::Result {
        let Some(runs) = &mut self.reflow else {
            return self.emit(text);
        };
        match runs.last_mut() {
            Some(run) if run.marker == self.marker && run.link == self.linking => {
                run.text.push_str(text);
            }
            _ => runs.push(Run {
                text: text.to_owned(),
                marker: self.marker,
                link: self.linking.clone(),
            }),
        }
        Ok(())
    }

    fn emit(&mut self, text: &str) -> fmt::Result {
        text.split_inclusive('\n').try_for_each(|line| {
            if line != "\n" {
                self.write_indent()?;
            }
            self.switch()?;
            self.fresh = line.ends_with('\n');
            self.column = match line.strip_suffix('\n') {
                Some(_) => 0,
                None => self.column + line.width(),
            };
            self.out.write_str(line)
        })
    }

    /// Reflows what `paint` writes to the viewer's width: its text breaks
    /// between words onto new lines at the current indent, keeping each
    /// word's style and link.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn reflowed(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        if self.reflow.is_some() || self.plain {
            return paint(self);
        }
        self.reflow = Some(Vec::new());
        let painted = paint(self);
        let runs = self.reflow.take().unwrap_or_default();
        painted?;
        let outer = self.marker;
        let mut word = Vec::new();
        let mut spaces = 0;
        for run in &runs {
            let mut rest = run.text.as_str();
            while !rest.is_empty() {
                let end = rest.find([' ', '\n']).unwrap_or(rest.len());
                if end > 0 {
                    word.push((run, &rest[..end]));
                }
                match rest[end..].chars().next() {
                    Some(' ') => {
                        self.place(&mut word, &mut spaces)?;
                        spaces += 1;
                    }
                    Some(_) => {
                        self.place(&mut word, &mut spaces)?;
                        spaces = 0;
                        self.marker = Marker::default();
                        self.emit("\n")?;
                    }
                    None => {}
                }
                rest = &rest[(end + 1).min(rest.len())..];
            }
        }
        self.place(&mut word, &mut spaces)?;
        self.marker = outer;
        Ok(())
    }

    fn place(&mut self, word: &mut Vec<(&Run, &str)>, spaces: &mut usize) -> fmt::Result {
        if word.is_empty() {
            return Ok(());
        }
        let width: usize = word.iter().map(|(_, piece)| piece.width()).sum();
        let start = 2 * self.indent;
        let column = if self.fresh { start } else { self.column };
        self.marker = Marker::default();
        if *spaces > 0 && column > start && column + *spaces + width > self.columns {
            self.emit("\n")?;
        } else if *spaces > 0 {
            self.emit(&" ".repeat(*spaces))?;
        }
        *spaces = 0;
        for (run, piece) in word.drain(..) {
            self.marker = run.marker;
            match &run.link {
                Some(url) => {
                    self.write_indent()?;
                    write!(self.out, "\x1b]8;;{url}\x1b\\")?;
                    self.emit(piece)?;
                    self.marker = Marker::default();
                    self.switch()?;
                    self.out.write_str("\x1b]8;;\x1b\\")?;
                }
                None => self.emit(piece)?,
            }
        }
        Ok(())
    }

    /// Writes formatted `args` as [`Painter::write_str`] does, so `write!`
    /// works without [`fmt::Write`] in scope.
    ///
    /// # Errors
    ///
    /// When the underlying writer fails.
    pub fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::Write::write_fmt(self, args)
    }

    /// Indents each line `paint` starts by two more columns.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn indented(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.indent += 1;
        let painted = paint(self);
        self.indent -= 1;
        painted
    }

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
        if self.reflow.is_some() {
            let outer = self.linking.replace(url.to_owned());
            let painted = self.underline(paint);
            self.linking = outer;
            return painted;
        }
        self.write_indent()?;
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

    /// Paints what `paint` writes in red.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn red(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.paint(Color::Red, paint)
    }

    /// Paints what `paint` writes in gray.
    ///
    /// # Errors
    ///
    /// When `paint` or the underlying writer fails.
    pub fn gray(&mut self, paint: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.paint(Color::Gray, paint)
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

    /// Paints the unified diff from `before` to `after`, with three lines of
    /// context around each change: hunk headers and unchanged lines in gray,
    /// removed lines in red, and added lines in green.
    ///
    /// # Errors
    ///
    /// When the underlying writer fails.
    pub fn diff(&mut self, before: &str, after: &str) -> fmt::Result {
        let diff = TextDiff::from_lines(before, after);
        let mut first = true;
        for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
            if !first {
                self.write_str("\n")?;
            }
            first = false;
            self.gray(|out| write!(out, "{}", hunk.header()))?;
            for change in hunk.iter_changes() {
                self.write_str("\n")?;
                let line = change.value().trim_end_matches('\n');
                match change.tag() {
                    ChangeTag::Delete => self.red(|out| write!(out, "-{line}"))?,
                    ChangeTag::Insert => self.green(|out| write!(out, "+{line}"))?,
                    ChangeTag::Equal => self.gray(|out| write!(out, " {line}"))?,
                }
            }
        }
        Ok(())
    }

    /// Writes a spinner that turns while it's shown.
    ///
    /// # Errors
    ///
    /// When the underlying writer fails.
    pub fn spinner(&mut self) -> fmt::Result {
        if self.reflow.is_some() {
            return self.write_str(self.spinner);
        }
        self.write_str(self.spinner)?;
        if let Some(spinners) = &mut self.spinners {
            spinners.push(self.out.len - self.spinner.len());
        }
        Ok(())
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

    fn write_indent(&mut self) -> fmt::Result {
        if self.fresh && self.indent > 0 {
            self.switch_to(Marker::default())?;
            write!(self.out, "{:width$}", "", width = 2 * self.indent)?;
            self.fresh = false;
            self.column = 2 * self.indent;
        }
        Ok(())
    }

    fn switch(&mut self) -> fmt::Result {
        self.switch_to(self.marker)
    }

    fn switch_to(&mut self, to: Marker) -> fmt::Result {
        let from = self.written;
        if self.plain || self.reflow.is_some() || from == to {
            return Ok(());
        }
        from.bold.switch(to.bold, &mut self.out)?;
        from.italic.switch(to.italic, &mut self.out)?;
        from.underline.switch(to.underline, &mut self.out)?;
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
        Painter::write_str(self, text)
    }
}

//! A file's lines as bat shows them: numbered from where they start, in a grid,
//! highlighted for the file's language.

use std::io::{self, Read, Write};

use bat::line_range::{LineRange, LineRanges};
use bat::{Input, PrettyPrinter, WrappingMode};

use crate::error::AppResult;
use crate::log::Screen;
use crate::note::Body;
use crate::text::Text;

/// `content` from the file at `path`, its first line being line `start`.
pub struct SourceExcerpt<'a> {
    /// Where the file is, which picks its language.
    pub path: &'a str,
    /// The lines shown.
    pub content: &'a str,
    /// The line number of `content`'s first line.
    pub start: u64,
}

impl Body for SourceExcerpt<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        let first = usize::try_from(self.start).unwrap_or(usize::MAX);
        let lines = io::repeat(b'\n')
            .take(self.start.saturating_sub(1))
            .chain(self.content.as_bytes());
        let mut printer = PrettyPrinter::new();
        printer
            .input(Input::from_reader(lines).name(self.path))
            .line_ranges(LineRanges::from(vec![LineRange::new(first, usize::MAX)]))
            .line_numbers(true)
            .grid(true)
            .term_width(usize::from(screen.columns))
            .wrapping_mode(WrappingMode::Character)
            .colored_output(true);
        if let Some(theme) = &screen.theme {
            printer.theme(theme);
        }
        printer.print_with_writer(Some(Text(&mut *out)))?;
        Ok(())
    }
}

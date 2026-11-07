use std::fmt;

use ::play_by_play::{Board, Live, Painter};

use crate::downloads::{Download, Downloads};

impl Live for Download {
    type Key = String;

    fn key(&self) -> &String {
        &self.file
    }

    fn show(&self, columns: u16, out: &mut Painter<'_, impl fmt::Write>) -> fmt::Result {
        let room = usize::from(columns).saturating_sub(self.progress.len() + 1);
        let file = self
            .file
            .char_indices()
            .nth(room)
            .map_or(self.file.as_str(), |(at, _)| &self.file[..at]);
        write!(out, "{file} ")?;
        if self.progress == "?" {
            return out.spinner();
        }
        out.write_str(&self.progress)
    }
}

impl Board for Downloads {
    type Key = String;
    type Line<'a> = &'a Download;

    fn lines(&self) -> impl Iterator<Item = &Download> {
        self.0.iter()
    }
}

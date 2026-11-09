use std::fmt;

use ::play_by_play::{Board, Live, Painter};

use crate::downloads::{Download, Downloads};

impl Live for Download {
    type Key = String;

    fn key(&self) -> &String {
        &self.file
    }

    fn show(&self, columns: u16, out: &mut Painter<'_, impl fmt::Write>) -> fmt::Result {
        if let Some((before, after)) = self.progress.split_once("->") {
            out.write_str(&self.file)?;
            return out.indented(|out| {
                writeln!(out)?;
                out.diff(&format!("{before}\n"), &format!("{after}\n"))
            });
        }
        if self.progress.contains(' ') {
            return out.reflowed(|out| write!(out, "{} {}", self.file, self.progress));
        }
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
        out.write_str(&self.progress)?;
        if self.progress != "100%" {
            return Ok(());
        }
        out.indented(|out| {
            writeln!(out)?;
            out.link(&format!("file:///downloads/{}", self.file), |out| {
                out.write_str("open")
            })
        })
    }
}

impl Board for Downloads {
    type Key = String;
    type Line<'a> = &'a Download;

    fn lines(&self, rows: u16) -> impl Iterator<Item = &Download> {
        self.0.iter().take(usize::from(rows))
    }
}

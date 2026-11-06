use std::fmt;

use ::play_by_play::{Line, Redraw, Width};

use crate::downloads::{Download, Downloads};

impl Line for Download {
    type Key = String;

    fn key(&self) -> &String {
        &self.file
    }

    fn draw(&self, width: Width, out: &mut impl fmt::Write) -> fmt::Result {
        let file = match width {
            Width::Columns(columns) => {
                let room = usize::from(columns).saturating_sub(self.progress.len() + 1);
                self.file
                    .char_indices()
                    .nth(room)
                    .map_or(self.file.as_str(), |(at, _)| &self.file[..at])
            }
            Width::Unbounded => &self.file,
        };
        write!(out, "{file} {}", self.progress)
    }

    fn narrate(&self, out: &mut impl fmt::Write) -> fmt::Result {
        if self.progress == "100%" {
            write!(out, "downloaded {}", self.file)
        } else {
            write!(out, "downloading {}", self.file)
        }
    }
}

impl Redraw for Downloads {
    type Key = String;
    type Line<'a> = &'a Download;

    fn draw(&self, _width: Width) -> impl Iterator<Item = &Download> {
        self.0.iter()
    }
}

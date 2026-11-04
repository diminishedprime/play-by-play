#![allow(missing_docs)]

use std::borrow::Cow;
use std::io::{self, Read, Write};
use std::slice;

use serde::Deserialize;

use crate::note::Open;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk<'a> {
    pub old_start: u64,
    pub old_lines: u64,
    pub new_start: u64,
    pub new_lines: u64,
    #[serde(borrow)]
    pub lines: Vec<Cow<'a, str>>,
}

/// `hunks` as a unified diff, read a piece at a time without building it.
pub struct Hunks<'h, 'a> {
    hunks: slice::Iter<'h, Hunk<'a>>,
    lines: slice::Iter<'h, Cow<'a, str>>,
    header: [u8; 96],
    piece: Piece<'h>,
    newline: bool,
    at: usize,
}

enum Piece<'h> {
    Header(usize),
    Text(&'h [u8]),
}

impl<'h, 'a> Hunks<'h, 'a> {
    #[must_use]
    pub fn new(hunks: &'h [Hunk<'a>]) -> Hunks<'h, 'a> {
        Hunks {
            hunks: hunks.iter(),
            lines: [].iter(),
            header: [0; 96],
            piece: Piece::Text(b""),
            newline: false,
            at: 0,
        }
    }

    fn current(&self) -> &[u8] {
        match self.piece {
            Piece::Header(length) => &self.header[..length],
            Piece::Text(text) => text,
        }
    }

    fn advance(&mut self) -> io::Result<bool> {
        self.at = 0;
        if self.newline {
            self.newline = false;
            self.piece = Piece::Text(b"\n");
            return Ok(true);
        }
        if let Some(line) = self.lines.next() {
            self.piece = Piece::Text(line.as_bytes());
            self.newline = true;
            return Ok(true);
        }
        let Some(hunk) = self.hunks.next() else {
            return Ok(false);
        };
        let unused = {
            let mut rest = &mut self.header[..];
            writeln!(
                rest,
                "@@ -{},{} +{},{} @@",
                hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
            )?;
            rest.len()
        };
        self.piece = Piece::Header(self.header.len() - unused);
        self.lines = hunk.lines.iter();
        Ok(true)
    }
}

impl Read for Hunks<'_, '_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        while self.at == self.current().len() {
            if !self.advance()? {
                return Ok(0);
            }
        }
        let bytes = &self.current()[self.at..];
        let read = bytes.len().min(buffer.len());
        buffer[..read].copy_from_slice(&bytes[..read]);
        self.at += read;
        Ok(read)
    }
}

#[derive(Deserialize)]
pub struct GitDiff<'a> {
    #[serde(borrow)]
    pub filename: Cow<'a, str>,
    #[serde(borrow)]
    pub status: Open<'a, DiffStatus>,
    pub additions: u64,
    pub deletions: u64,
    pub changes: u64,
    #[serde(borrow)]
    pub patch: Cow<'a, str>,
    #[serde(borrow)]
    pub repository: Option<Cow<'a, str>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStatus {
    Modified,
    Added,
}

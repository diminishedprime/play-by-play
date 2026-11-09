use std::fmt;

use ::play_by_play::{Board, Broadcast, Live, Painter};
use tokio::io::{self, AsyncBufReadExt, BufReader};

pub struct Debug;

pub enum Shown {
    Columns,
    Rows(u16),
}

impl Debug {
    pub async fn run(mut broadcast: Broadcast<&'static str>) {
        let mut stdin = BufReader::new(io::stdin()).lines();
        loop {
            broadcast
                .render(&Debug)
                .await
                .expect("the broadcast renders");
            tokio::select! {
                line = stdin.next_line() => {
                    if line.expect("stdin can be read").is_none() {
                        return;
                    }
                }
                resized = broadcast.resized() => resized.expect("the viewer's size arrives"),
            }
        }
    }
}

impl Board for Debug {
    type Key = &'static str;
    type Line<'a> = Shown;

    fn lines(&self, rows: u16) -> impl Iterator<Item = Shown> {
        [Shown::Columns, Shown::Rows(rows)].into_iter()
    }
}

impl Live for Shown {
    type Key = &'static str;

    fn key(&self) -> &&'static str {
        match self {
            Shown::Columns => &"columns",
            Shown::Rows(_) => &"rows",
        }
    }

    fn show(&self, columns: u16, out: &mut Painter<'_, impl fmt::Write>) -> fmt::Result {
        match self {
            Shown::Columns => write!(out, "columns {columns}"),
            Shown::Rows(rows) => write!(out, "rows {rows}"),
        }
    }
}

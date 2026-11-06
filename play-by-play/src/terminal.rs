use std::env as process_env;
use std::future;
use std::io::Write as _;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::env;
use crate::error::AppResult;

pub(crate) struct Terminal {
    socket: UnixStream,
    columns: u16,
    width: [u8; 2],
    filled: usize,
    frame: Vec<u8>,
    drawn: usize,
    drawn_columns: u16,
}

impl Terminal {
    pub(crate) async fn connect() -> AppResult<Option<Terminal>> {
        let Some(path) = process_env::var_os(env::SOCKET) else {
            return Ok(None);
        };
        let mut socket = UnixStream::connect(path).await?;
        let mut width = [0; 2];
        socket.read_exact(&mut width).await?;
        let columns = u16::from_ne_bytes(width);
        Ok(Some(Terminal {
            socket,
            columns,
            width,
            filled: 0,
            frame: Vec::new(),
            drawn: 0,
            drawn_columns: columns,
        }))
    }

    pub(crate) fn columns(&self) -> u16 {
        self.columns
    }

    pub(crate) async fn resized(&mut self) -> AppResult {
        loop {
            let read = self.socket.read(&mut self.width[self.filled..]).await?;
            if read == 0 {
                return future::pending().await;
            }
            self.filled += read;
            if self.filled == self.width.len() {
                self.filled = 0;
                self.columns = u16::from_ne_bytes(self.width);
                return Ok(());
            }
        }
    }

    pub(crate) async fn draw<'a>(&mut self, lines: impl IntoIterator<Item = &'a str>) -> AppResult {
        self.frame.clear();
        if self.drawn_columns != self.columns {
            self.frame.extend_from_slice(b"\x1b[H\x1b[2J");
        } else if self.drawn > 0 {
            write!(self.frame, "\x1b[{}F\x1b[J", self.drawn)?;
        }
        self.drawn = 0;
        self.drawn_columns = self.columns;
        for line in lines {
            let cut = line
                .char_indices()
                .nth(usize::from(self.columns))
                .map_or(line, |(at, _)| &line[..at]);
            self.frame.extend_from_slice(cut.as_bytes());
            self.frame.push(b'\n');
            self.drawn += 1;
        }
        self.socket.write_all(&self.frame).await?;
        Ok(())
    }
}

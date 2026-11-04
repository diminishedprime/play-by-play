use std::fs::File;
use std::os::unix::net::UnixStream;

use crate::error::AppResult;

pub(crate) struct Terminal {
    log: File,
    socket: UnixStream,
    columns: u16,
    drawn: Option<Drawn>,
}

struct Drawn {
    ends_at: u64,
    lines: u16,
}

impl Terminal {
    pub(crate) fn connect() -> AppResult<Option<Terminal>> {
        todo!("None without env::SOCKET; else connect, which claims, read the width, open env::FD")
    }

    pub(crate) fn columns(&mut self) -> u16 {
        todo!("take any width the socket sent since, else the last one")
    }

    pub(crate) fn draw<'a>(&mut self, _lines: impl IntoIterator<Item = &'a str>) -> AppResult {
        todo!(
            "compare the log's length to the last draw's end, then rewrite or reprint, cut to the columns"
        )
    }
}

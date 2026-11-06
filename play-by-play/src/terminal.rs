use std::future;
use std::io::{self, ErrorKind};
use std::path::Path;

use bytes::{BufMut as _, BytesMut};
use prost::Message as _;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt as _;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use crate::error::AppResult;
use crate::proto;

const FRAME_LENGTH: usize = 4;
const LINES_FIELD: u8 = 1;
const LENGTH_DELIMITED: u8 = 2;
const LINES_KEY: u8 = (LINES_FIELD << 3) | LENGTH_DELIMITED;

pub(crate) struct Terminal {
    widths: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    socket: OwnedWriteHalf,
    columns: u16,
    message: BytesMut,
}

impl Terminal {
    pub(crate) async fn connect(path: impl AsRef<Path>) -> AppResult<Terminal> {
        let (read, socket) = UnixStream::connect(path).await?.into_split();
        let mut widths = FramedRead::new(read, LengthDelimitedCodec::new());
        let Some(columns) = Terminal::next_width(&mut widths).await? else {
            return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
        };
        Ok(Terminal {
            widths,
            socket,
            columns,
            message: BytesMut::new(),
        })
    }

    pub(crate) fn columns(&self) -> u16 {
        self.columns
    }

    pub(crate) async fn resized(&mut self) -> AppResult {
        match Terminal::next_width(&mut self.widths).await? {
            Some(columns) => {
                self.columns = columns;
                Ok(())
            }
            None => future::pending().await,
        }
    }

    async fn next_width(
        widths: &mut FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    ) -> AppResult<Option<u16>> {
        let Some(message) = widths.next().await else {
            return Ok(None);
        };
        let width = proto::Width::decode(message?)?;
        Ok(Some(u16::try_from(width.columns)?))
    }

    pub(crate) async fn send<'a>(&mut self, lines: impl Iterator<Item = &'a str>) -> AppResult {
        self.message.clear();
        self.message.put_bytes(0, FRAME_LENGTH);
        for line in lines {
            self.message.put_u8(LINES_KEY);
            prost::encode_length_delimiter(line.len(), &mut self.message)?;
            self.message.put_slice(line.as_bytes());
        }
        let length = u32::try_from(self.message.len() - FRAME_LENGTH)?;
        self.message[..FRAME_LENGTH].copy_from_slice(&length.to_be_bytes());
        self.socket.write_all(&self.message).await?;
        Ok(())
    }
}

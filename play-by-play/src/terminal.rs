use std::future;
use std::io::{self, ErrorKind};
use std::mem;
use std::path::Path;

use bytes::{BufMut as _, BytesMut};
use prost::Message as _;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::StreamExt as _;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

use crate::error::AppResult;
use crate::proto;
use crate::spinner::Spinner;

const FRAME_LENGTH: usize = 4;
const LINES_FIELD: u8 = 1;
const LENGTH_DELIMITED: u8 = 2;
const LINES_KEY: u8 = (LINES_FIELD << 3) | LENGTH_DELIMITED;

pub(crate) struct Terminal {
    widths: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    columns: u16,
    spinner: Spinner,
    spare: Sent,
    sent: Sender<Sent>,
    written: Receiver<io::Result<Sent>>,
}

#[derive(Default)]
struct Sent {
    message: BytesMut,
    spinners: Vec<usize>,
}

struct Writer {
    socket: OwnedWriteHalf,
    latest: Sent,
    sent: Receiver<Sent>,
    written: Sender<io::Result<Sent>>,
    spinner: Spinner,
}

impl Terminal {
    pub(crate) async fn connect(path: impl AsRef<Path>) -> AppResult<Terminal> {
        let (read, socket) = UnixStream::connect(path).await?.into_split();
        let mut widths = FramedRead::new(read, LengthDelimitedCodec::new());
        let Some(columns) = Terminal::next_width(&mut widths).await? else {
            return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
        };
        let (sent, to_write) = mpsc::channel(1);
        let (was_written, written) = mpsc::channel(1);
        let spinner = Spinner::new();
        tokio::spawn(
            Writer {
                socket,
                latest: Sent::default(),
                sent: to_write,
                written: was_written,
                spinner,
            }
            .run(),
        );
        Ok(Terminal {
            widths,
            columns,
            spinner,
            spare: Sent::default(),
            sent,
            written,
        })
    }

    pub(crate) fn columns(&self) -> u16 {
        self.columns
    }

    pub(crate) fn spinner(&self) -> &'static str {
        self.spinner.frame()
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

    pub(crate) async fn send<'a>(
        &mut self,
        lines: impl Iterator<Item = (&'a str, &'a [usize])>,
    ) -> AppResult {
        let mut sent = mem::take(&mut self.spare);
        sent.message.clear();
        sent.spinners.clear();
        sent.message.put_bytes(0, FRAME_LENGTH);
        for (line, spinners) in lines {
            sent.message.put_u8(LINES_KEY);
            prost::encode_length_delimiter(line.len(), &mut sent.message)?;
            let start = sent.message.len();
            sent.message.put_slice(line.as_bytes());
            sent.spinners.extend(spinners.iter().map(|at| start + at));
        }
        let length = u32::try_from(sent.message.len() - FRAME_LENGTH)?;
        sent.message[..FRAME_LENGTH].copy_from_slice(&length.to_be_bytes());
        if self.sent.send(sent).await.is_err() {
            return Err(io::Error::from(ErrorKind::BrokenPipe).into());
        }
        match self.written.recv().await {
            Some(written) => {
                self.spare = written?;
                Ok(())
            }
            None => Err(io::Error::from(ErrorKind::BrokenPipe).into()),
        }
    }
}

impl Writer {
    async fn run(mut self) {
        let mut turns = self.spinner.turns();
        loop {
            tokio::select! {
                sent = self.sent.recv() => {
                    let Some(sent) = sent else {
                        return;
                    };
                    let previous = mem::replace(&mut self.latest, sent);
                    let wrote = self.socket.write_all(&self.latest.message).await;
                    let failed = wrote.is_err();
                    if self.written.send(wrote.map(|()| previous)).await.is_err() || failed {
                        return;
                    }
                }
                _ = turns.tick(), if !self.latest.spinners.is_empty() => {
                    self.latest.turn(self.spinner.frame());
                    if self.socket.write_all(&self.latest.message).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}

impl Sent {
    fn turn(&mut self, frame: &str) {
        for &at in &self.spinners {
            self.message[at..at + frame.len()].copy_from_slice(frame.as_bytes());
        }
    }
}

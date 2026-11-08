//! The socket a command's program connects to, to claim the viewer's terminal and
//! hear its width.

use std::cell::Cell;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::process;

use bytes::{Bytes, BytesMut};
use notify::{Config, KqueueWatcher, RecursiveMode, Watcher};
use play_by_play::proto;
use prost::Message as _;
use tokio::io::AsyncWriteExt;
use tokio::net::unix::WriteHalf;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tokio_stream::StreamExt as _;
use tokio_util::codec::{Encoder as _, FramedRead, LengthDelimitedCodec};

use crate::drawn::Drawn;
use crate::error::{AppError, AppResult};

/// The socket play-by-play offers one command, only while a viewer is
/// watching; a command that never connects is left a plain command.
pub struct Channel {
    path: PathBuf,
    listener: UnixListener,
    program: Option<UnixStream>,
    columns: u16,
    sent: Option<Size>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Size {
    columns: u16,
    rows: u16,
}

impl Channel {
    /// Offers a socket in `dir` for one command.
    ///
    /// # Errors
    ///
    /// When the socket can't be bound.
    pub fn open(dir: &Path) -> AppResult<Channel> {
        let path = dir.join(format!("{}.socket", process::id()));
        let listener = UnixListener::bind(&path)?;
        Ok(Channel {
            path,
            listener,
            program: None,
            columns: 0,
            sent: None,
        })
    }

    /// Where the program connects, for [`play_by_play::env::SOCKET`].
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Waits for the program to connect, which claims the viewer's terminal.
    ///
    /// # Errors
    ///
    /// When the connection can't be accepted.
    pub async fn accept(&mut self) -> AppResult {
        let (program, _) = self.listener.accept().await?;
        self.program = Some(program);
        Ok(())
    }

    /// Tells the program the size its follower records in `follower_columns`
    /// and `follower_rows`, or the viewer's `columns` and `rows` while no
    /// follower has, unless it's already gone.
    ///
    /// # Errors
    ///
    /// When the program can't be written to for any other reason.
    ///
    /// # Panics
    ///
    /// When no program has connected through [`Channel::accept`].
    pub async fn send_size(
        &mut self,
        columns: u16,
        rows: u16,
        follower_columns: &Path,
        follower_rows: &Path,
    ) -> AppResult {
        let program = self
            .program
            .as_mut()
            .expect("a program connected before hearing the width");
        let size = Channel::recorded_size(follower_columns, follower_rows)
            .unwrap_or(Size { columns, rows });
        self.columns = size.columns;
        self.sent = Some(size);
        let sent = async {
            program.write_all(&Channel::width(size)?).await?;
            Ok(())
        };
        Channel::unless_gone(sent.await)
    }

    /// Draws each of the program's [`proto::Lines`] into `live`, cut to the
    /// width the program was last told, until it disconnects. Meanwhile sends
    /// it each size its follower records in `columns` and `rows`, rereading
    /// them on every change in the follower's `dir`.
    ///
    /// # Errors
    ///
    /// When the program can't be read or written, `live` written, or `dir`
    /// watched.
    ///
    /// # Panics
    ///
    /// When no program has connected through [`Channel::accept`].
    pub async fn serve(
        &mut self,
        mut live: impl Write,
        dir: &Path,
        columns: &Path,
        rows: &Path,
    ) -> AppResult {
        let width = Cell::new(self.columns);
        let first = self.sent;
        let (reader, mut writer) = self
            .program
            .as_mut()
            .expect("a program connected before sending its view")
            .split();
        let mut relay = pin!(async {
            let mut messages = FramedRead::new(reader, LengthDelimitedCodec::new());
            let mut drawn = Drawn::new(width.get());
            while let Some(message) = messages.next().await {
                let lines = proto::Lines::decode(message?)?;
                drawn.draw(&mut live, &lines.lines, width.get())?;
            }
            Ok(())
        });
        let sending = async {
            let (sender, mut changes) = mpsc::unbounded_channel();
            let mut watcher = KqueueWatcher::new(
                move |change| {
                    let _ = sender.send(change);
                },
                Config::default(),
            )?;
            watcher.watch(dir, RecursiveMode::NonRecursive)?;
            let mut sent =
                Channel::send_recorded(&mut writer, columns, rows, first, &width).await?;
            loop {
                let change = changes
                    .recv()
                    .await
                    .expect("the watcher keeps its sender while serve runs");
                change?;
                sent = Channel::send_recorded(&mut writer, columns, rows, sent, &width).await?;
            }
        };
        let mut widths = pin!(async { Channel::unless_gone(sending.await) });
        tokio::select! {
            biased;
            Err(error) = &mut widths => Err(error),
            relayed = &mut relay => relayed,
        }
    }

    fn unless_gone(sent: AppResult) -> AppResult {
        match sent {
            Err(AppError::Io(error))
                if matches!(
                    error.kind(),
                    ErrorKind::BrokenPipe | ErrorKind::ConnectionReset
                ) =>
            {
                Ok(())
            }
            other => other,
        }
    }

    async fn send_recorded(
        writer: &mut WriteHalf<'_>,
        columns: &Path,
        rows: &Path,
        sent: Option<Size>,
        width: &Cell<u16>,
    ) -> AppResult<Option<Size>> {
        let recorded = Channel::recorded_size(columns, rows);
        match recorded {
            Some(size) if recorded != sent => {
                writer.write_all(&Channel::width(size)?).await?;
                width.set(size.columns);
                Ok(recorded)
            }
            _ => Ok(sent),
        }
    }

    fn recorded_size(columns: &Path, rows: &Path) -> Option<Size> {
        Channel::recorded(columns).map(|columns| Size {
            columns,
            rows: Channel::recorded(rows).unwrap_or(u16::MAX),
        })
    }

    fn recorded(path: &Path) -> Option<u16> {
        fs::read_to_string(path)
            .ok()
            .and_then(|recorded| recorded.parse().ok())
    }

    fn width(size: Size) -> AppResult<BytesMut> {
        let width = proto::Width {
            columns: size.columns.into(),
            rows: size.rows.into(),
        };
        let mut message = BytesMut::new();
        LengthDelimitedCodec::new().encode(Bytes::from(width.encode_to_vec()), &mut message)?;
        Ok(message)
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

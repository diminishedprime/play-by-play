//! The socket a command's program connects to, to claim the viewer's terminal and
//! hear its width.

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::process;

use notify::{Config, KqueueWatcher, RecursiveMode, Watcher};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::WriteHalf;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

use crate::error::{AppError, AppResult};

/// The socket `play-by-play run` offers one command, only while a viewer is
/// watching; a command that never connects is left a plain command.
pub struct Channel {
    path: PathBuf,
    listener: UnixListener,
    program: Option<UnixStream>,
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

    /// Tells the program the viewer's width, unless it's already gone.
    ///
    /// # Errors
    ///
    /// When the program can't be written to for any other reason.
    ///
    /// # Panics
    ///
    /// When no program has connected through [`Channel::accept`].
    pub async fn send_columns(&mut self, columns: u16) -> AppResult {
        let program = self
            .program
            .as_mut()
            .expect("a program connected before hearing the width");
        let sent = async {
            program.write_all(&columns.to_ne_bytes()).await?;
            Ok(())
        };
        Channel::unless_gone(sent.await)
    }

    /// Copies everything the program sends into `live` until it disconnects,
    /// meanwhile sending it each width its follower records in `columns`,
    /// rereading it on every change in the follower's `dir`.
    ///
    /// # Errors
    ///
    /// When the program can't be read or written, `live` written, or `dir`
    /// watched.
    ///
    /// # Panics
    ///
    /// When no program has connected through [`Channel::accept`].
    pub async fn serve(&mut self, mut live: impl Write, dir: &Path, columns: &Path) -> AppResult {
        let (mut reader, mut writer) = self
            .program
            .as_mut()
            .expect("a program connected before sending its view")
            .split();
        let mut relay = pin!(async {
            let mut chunk = [0; 8192];
            loop {
                let read = reader.read(&mut chunk).await?;
                if read == 0 {
                    return Ok(());
                }
                live.write_all(&chunk[..read])?;
            }
        });
        let widths = async {
            let (sender, mut changes) = mpsc::unbounded_channel();
            let mut watcher = KqueueWatcher::new(
                move |change| {
                    let _ = sender.send(change);
                },
                Config::default(),
            )?;
            watcher.watch(dir, RecursiveMode::NonRecursive)?;
            let mut sent = Channel::send_recorded(&mut writer, columns, None).await?;
            while let Some(change) = changes.recv().await {
                change?;
                sent = Channel::send_recorded(&mut writer, columns, sent).await?;
            }
            Ok(())
        };
        tokio::select! {
            relayed = &mut relay => relayed,
            watched = widths => {
                Channel::unless_gone(watched)?;
                relay.await
            }
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
        sent: Option<u16>,
    ) -> AppResult<Option<u16>> {
        let recorded = fs::read_to_string(columns)
            .ok()
            .and_then(|recorded| recorded.parse::<u16>().ok());
        match recorded {
            Some(width) if recorded != sent => {
                writer.write_all(&width.to_ne_bytes()).await?;
                Ok(recorded)
            }
            _ => Ok(sent),
        }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

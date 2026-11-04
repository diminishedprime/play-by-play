//! The viewer: left running in a terminal (the VS Code task opens one), it shows
//! every command Claude runs in the workspace as it runs.

use std::fs::File;
use std::io::{self, ErrorKind, Seek, SeekFrom, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::task::{Context, Poll};

use bat::assets::HighlightingAssets;
use memchr::memmem;
use memmap2::Mmap;
use notify::{Config, Event, KqueueWatcher, RecursiveMode, Watcher};
use rustix::termios;
use terminal_colorsaurus::QueryOptions;
use tokio::runtime;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::wrappers::SignalStream;
use tokio_stream::{Stream, StreamExt};

use crate::error::AppResult;
use crate::log::{CLEAR, Log};

/// Follows `log` until interrupted, showing what's added to it from now on.
///
/// # Errors
///
/// When the log's folder can't be made or watched, or the log can't be read.
pub fn watch(log: &Log) -> AppResult {
    log.append()?;
    runtime::Builder::new_current_thread()
        .enable_io()
        .build()?
        .block_on(Follower::from(log).follow())
}

/// Becomes tmux, attached to the workspace's session, which is started running
/// `play-by-play watch` if it isn't already.
///
/// # Errors
///
/// When tmux can't be started.
pub fn in_tmux(log: &Log) -> AppResult {
    let error = Command::new("tmux")
        .args(["new-session", "-A", "-s"])
        .arg(log.path().file_stem().unwrap_or_default())
        .arg("tmux set-option destroy-unattached off && play-by-play watch")
        .exec();
    Err(error.into())
}

struct Follower {
    path: PathBuf,
    columns: PathBuf,
    theme: PathBuf,
    from: u64,
}

impl From<&Log<'_>> for Follower {
    fn from(log: &Log<'_>) -> Follower {
        Follower {
            path: log.path(),
            columns: log.columns_path(),
            theme: log.theme_path(),
            from: 0,
        }
    }
}

impl Follower {
    async fn follow(mut self) -> AppResult {
        let resizes = Wake::resizes()?;
        self.record_width()?;
        self.record_theme()?;
        let changes = Wake::changes(&self.path)?;
        self.from = self.last_screen()?;
        let mut stdout = io::stdout();
        self.print_tail(&mut stdout)?;
        let mut wakes = changes.merge(resizes);
        while let Some(wake) = wakes.next().await {
            match wake {
                Wake::Change => self.print_tail(&mut stdout)?,
                Wake::Resize => self.record_width()?,
            }
        }
        Ok(())
    }

    fn record_theme(&self) -> AppResult {
        let Ok(background) = terminal_colorsaurus::background_color(QueryOptions::default()) else {
            return Ok(());
        };
        let assets = HighlightingAssets::from_binary();
        let closest = assets.themes().min_by_key(|name| {
            assets
                .get_theme(name)
                .settings
                .background
                .map_or(u32::MAX, |theme| {
                    [
                        (theme.r, background.r),
                        (theme.g, background.g),
                        (theme.b, background.b),
                    ]
                    .into_iter()
                    .map(|(theme, terminal)| {
                        u32::from(theme).abs_diff(u32::from(terminal >> 8)).pow(2)
                    })
                    .sum()
                })
        });
        if let Some(name) = closest {
            write!(File::create(&self.theme)?, "{name}")?;
        }
        Ok(())
    }

    fn record_width(&self) -> AppResult {
        let columns = termios::tcgetwinsize(io::stdout())?.ws_col;
        write!(File::create(&self.columns)?, "{columns}")?;
        Ok(())
    }

    fn last_screen(&self) -> AppResult<u64> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(error.into()),
        };
        // SAFETY: play-by-play only ever appends to the log; a truncation while it's mapped could still crash this read.
        let log = unsafe { Mmap::map(&file)? };
        let start = memmem::rfind(&log, CLEAR.as_bytes()).unwrap_or(0);
        Ok(u64::try_from(start).expect("a log's length fits in a u64"))
    }

    fn print_tail(&mut self, out: &mut impl Write) -> AppResult {
        let mut file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        file.seek(SeekFrom::Start(self.from))?;
        self.from += io::copy(&mut file, out)?;
        out.flush()?;
        Ok(())
    }
}

enum Wake {
    Change,
    Resize,
}

impl Wake {
    fn changes(path: &Path) -> AppResult<Changes> {
        let (sender, events) = mpsc::unbounded_channel();
        let mut watcher = KqueueWatcher::new(
            move |change| {
                let _ = sender.send(change);
            },
            Config::default(),
        )?;
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        Ok(Changes {
            _watcher: watcher,
            events,
        })
    }

    fn resizes() -> AppResult<impl Stream<Item = Wake>> {
        Ok(SignalStream::new(signal(SignalKind::window_change())?).map(|()| Wake::Resize))
    }
}

struct Changes {
    _watcher: KqueueWatcher,
    events: UnboundedReceiver<notify::Result<Event>>,
}

impl Stream for Changes {
    type Item = Wake;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Wake>> {
        self.events
            .poll_recv(context)
            .map(|event| event.map(|_| Wake::Change))
    }
}

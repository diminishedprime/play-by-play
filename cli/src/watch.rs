//! The viewer: left running in a terminal (the VS Code task opens one), it shows
//! every command Claude runs in the workspace as it runs.

use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{self, Command};
use std::task::{Context, Poll};

use bat::assets::HighlightingAssets;
use memchr::memmem;
use memmap2::Mmap;
use notify::{Config, Event, KqueueWatcher, RecursiveMode, Watcher};
use play_by_play::log::Log;
use rustix::termios;
use terminal_colorsaurus::QueryOptions;
use tokio::runtime;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::wrappers::SignalStream;
use tokio_stream::{Stream, StreamExt};

use crate::error::AppResult;
use crate::log::CLEAR;

/// Follows `log` until interrupted, showing what's added to it from now on.
///
/// # Errors
///
/// When the log's folder can't be made or watched, or the log can't be read.
pub fn watch(log: &Log) -> AppResult {
    let file = log.append()?;
    fs::write(log.viewer_path(), process::id().to_string())?;
    runtime::Builder::new_current_thread()
        .enable_io()
        .build()?
        .block_on(Follower::from(log).follow(&file))
}

/// Follows the live view of the program under the `run` with `pid`, waiting for
/// it when the program hasn't claimed the viewer yet.
///
/// # Errors
///
/// When the program's view can't be read.
pub fn follow(log: &Log, pid: u32) -> AppResult {
    File::options()
        .create(true)
        .append(true)
        .open(log.live_path(pid))?;
    runtime::Builder::new_current_thread()
        .enable_io()
        .build()?
        .block_on(Tail::from(log.live_path(pid)).follow(Size {
            columns: Recorded {
                path: log.follower_columns_path(pid),
                temporary: log.follower_columns_temporary_path(pid),
            },
            rows: Recorded {
                path: log.follower_rows_path(pid),
                temporary: log.follower_rows_temporary_path(pid),
            },
        }))
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

struct Tail {
    path: PathBuf,
    from: u64,
}

impl From<PathBuf> for Tail {
    fn from(path: PathBuf) -> Tail {
        Tail { path, from: 0 }
    }
}

impl Tail {
    async fn follow(mut self, size: Size) -> AppResult {
        let resizes = Wake::resizes()?;
        let hangups = Wake::hangups()?;
        size.record()?;
        let changes = Wake::changes(&self.path);
        let mut stdout = io::stdout();
        self.print(&mut stdout)?;
        let mut wakes = changes.merge(resizes).merge(hangups);
        while let Some(wake) = wakes.next().await {
            match wake {
                Wake::Change => self.print(&mut stdout)?,
                Wake::Resize => size.record()?,
                Wake::HangUp => break,
            }
        }
        Ok(())
    }

    fn print(&mut self, out: &mut impl Write) -> AppResult {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.from))?;
        self.from += io::copy(&mut file, out)?;
        out.flush()?;
        Ok(())
    }
}

struct Recorded {
    path: PathBuf,
    temporary: PathBuf,
}

impl Recorded {
    fn write(&self, value: impl Display) -> AppResult {
        [&self.temporary, &self.path]
            .into_iter()
            .filter_map(|file| file.parent())
            .try_for_each(fs::create_dir_all)?;
        write!(File::create(&self.temporary)?, "{value}")?;
        fs::rename(&self.temporary, &self.path)?;
        Ok(())
    }
}

struct Size {
    columns: Recorded,
    rows: Recorded,
}

impl Size {
    fn record(&self) -> AppResult {
        let size = termios::tcgetwinsize(io::stdout())?;
        self.rows.write(size.ws_row)?;
        self.columns.write(size.ws_col)
    }
}

struct Follower {
    tail: Tail,
    size: Size,
    theme: Recorded,
}

impl From<&Log<'_>> for Follower {
    fn from(log: &Log<'_>) -> Follower {
        Follower {
            tail: Tail::from(log.path()),
            size: Size {
                columns: Recorded {
                    path: log.columns_path(),
                    temporary: log.columns_temporary_path(),
                },
                rows: Recorded {
                    path: log.rows_path(),
                    temporary: log.rows_temporary_path(),
                },
            },
            theme: Recorded {
                path: log.theme_path(),
                temporary: log.theme_temporary_path(),
            },
        }
    }
}

impl Follower {
    async fn follow(mut self, file: &File) -> AppResult {
        let resizes = Wake::resizes()?;
        let hangups = Wake::hangups()?;
        self.size.record()?;
        self.record_theme();
        let changes = Wake::changes(&self.tail.path);
        self.tail.from = Follower::last_screen(file)?;
        let mut stdout = io::stdout();
        if file.metadata()?.len() == 0 {
            writeln!(stdout, "play-by-play is watching this workspace.")?;
            writeln!(stdout, "Claude's commands show up here as they run.")?;
        }
        self.tail.print(&mut stdout)?;
        let mut wakes = changes.merge(resizes).merge(hangups);
        while let Some(wake) = wakes.next().await {
            match wake {
                Wake::Change => self.tail.print(&mut stdout)?,
                Wake::Resize => self.size.record()?,
                Wake::HangUp => break,
            }
        }
        Ok(())
    }

    fn record_theme(&self) {
        let Ok(background) = terminal_colorsaurus::background_color(QueryOptions::default()) else {
            return;
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
            let _ = self.theme.write(name);
        }
    }

    fn last_screen(file: &File) -> AppResult<u64> {
        // SAFETY: play-by-play only ever appends to the log; a truncation while it's mapped could still crash this read.
        let log = unsafe { Mmap::map(file)? };
        let start = memmem::rfind(&log, CLEAR.as_bytes()).unwrap_or(0);
        Ok(u64::try_from(start).expect("a log's length fits in a u64"))
    }
}

enum Wake {
    Change,
    Resize,
    HangUp,
}

impl Wake {
    fn changes(path: &Path) -> Changes {
        let (sender, events) = mpsc::unbounded_channel();
        let mut watcher = KqueueWatcher::new(
            move |change| {
                let _ = sender.send(change);
            },
            Config::default(),
        )
        .expect("kqueue starts, or there's no log to follow");
        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .expect("kqueue watches the log, or there's no log to follow");
        Changes {
            _watcher: watcher,
            events,
        }
    }

    fn resizes() -> AppResult<impl Stream<Item = Wake>> {
        Ok(SignalStream::new(signal(SignalKind::window_change())?).map(|()| Wake::Resize))
    }

    fn hangups() -> AppResult<impl Stream<Item = Wake>> {
        Ok(SignalStream::new(signal(SignalKind::hangup())?).map(|()| Wake::HangUp))
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

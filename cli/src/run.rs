//! What every Bash command Claude runs goes through: the plugin rewrites each one
//! to `pbp -c '<command>'`.

use std::cell::Cell;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{self, Stdio};

use play_by_play::env;
use tokio::process::Command;
use tokio::runtime;
use tokio::sync::Notify;

use crate::channel::Channel;
use crate::error::{AppError, AppResult};
use crate::log::{Color, Log, RESET, Screen};
use crate::note::{Header, Title};
use crate::router::Router;
use crate::viewer::Viewer;

/// Runs `command` in zsh, writing to `log`, and routes its output to Claude and
/// the viewer.
///
/// # Errors
///
/// When the log can't be written or zsh can't be started; the command exiting
/// non-zero is [`AppError::Exited`] with its code.
///
/// # Panics
///
/// Never: both streams are piped before they're taken.
pub fn run(log: &Log, dir: &Path, command: &str) -> AppResult {
    let color = log.next_color()?;
    let viewer = Viewer::watching(log)?;
    let live_log = log.live_path(process::id());
    let follower = log.follower_dir(process::id());
    let columns = log.follower_columns_path(process::id());
    let mut log = log.append()?;
    Prompt(command).header(&mut log, color, &Screen::UNBOUNDED)?;
    let mut shell = Command::new("zsh");
    shell
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let code = runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(Tee::run(
            shell, &log, dir, viewer, &live_log, &follower, &columns,
        ))?;
    // The longest exit line is 19 bytes.
    // color 5 + "exit: " 6 + code 3 + reset 4 + newline 1
    let mut line = [0; 32];
    let unused = {
        let mut rest = &mut line[..];
        let color = if code == 0 { color } else { Color::Red };
        writeln!(rest, "{color}exit: {code}{RESET}")?;
        rest.len()
    };
    log.write_all(&line[..line.len() - unused])?;
    match code {
        0 => Ok(()),
        code => Err(AppError::Exited(code)),
    }
}

/// Prints the command the `pbp -c '<command>'` on stdin holds.
///
/// # Errors
///
/// When stdin can't be read or stdout can't be written.
pub fn unwrap() -> AppResult {
    let mut wrapped = String::new();
    io::stdin().read_to_string(&mut wrapped)?;
    let command = wrapped
        .strip_prefix("pbp -c '")
        .and_then(|quoted| quoted.strip_suffix('\''))
        .map_or_else(|| wrapped.clone(), |quoted| quoted.replace("'\\''", "'"));
    io::stdout().write_all(command.as_bytes())?;
    Ok(())
}

struct Prompt<'a>(&'a str);

impl Title for Prompt<'_> {
    fn title(&self, out: &mut impl Write) -> AppResult {
        write!(out, "$ {}", self.0)?;
        Ok(())
    }
}

struct Tee;

impl Tee {
    async fn run(
        mut shell: Command,
        log: &File,
        dir: &Path,
        viewer: Option<Viewer>,
        live_log: &Path,
        follower: &Path,
        columns: &Path,
    ) -> AppResult<u8> {
        let mut watched = viewer
            .map(|viewer| Channel::open(dir).map(|channel| (channel, viewer)))
            .transpose()?;
        if let Some((channel, _)) = &watched {
            shell.env(env::SOCKET, channel.path());
        }
        let mut child = match shell.spawn() {
            Err(error) if error.kind() == ErrorKind::NotFound => return Err(AppError::NoZsh),
            spawned => spawned?,
        };
        let stdout = child.stdout.take().expect("the command's stdout is piped");
        let stderr = child.stderr.take().expect("the command's stderr is piped");
        let claimed = Cell::new(false);
        let copied = Notify::new();
        let copying = async {
            tokio::try_join!(
                Router::new(log.try_clone()?, &claimed).copy(stdout, io::stdout()),
                Router::new(log.try_clone()?, &claimed).copy(stderr, io::stderr()),
            )?;
            copied.notify_one();
            AppResult::Ok(())
        };
        let claim = async {
            let Some((channel, viewer)) = &mut watched else {
                return Ok(());
            };
            tokio::select! {
                biased;
                accepted = channel.accept() => accepted?,
                () = copied.notified() => return Ok(()),
            }
            claimed.set(true);
            channel.send_columns(viewer.columns()).await?;
            fs::create_dir_all(follower)?;
            channel
                .serve(File::create(live_log)?, follower, columns)
                .await
        };
        tokio::try_join!(copying, claim)?;
        let status = child.wait().await?;
        Ok(status
            .code()
            .map_or(u8::MAX, |code| u8::try_from(code).unwrap_or(u8::MAX)))
    }
}

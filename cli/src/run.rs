//! What every Bash command Claude runs goes through: the plugin rewrites each one
//! to `pbp -c '<command>'`.

use std::cell::Cell;
use std::fmt::{self, Formatter};
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{self, ExitCode, Stdio};

use play_by_play::log::Log;
use play_by_play::{Color, env};
use tokio::process::Command;
use tokio::runtime;
use tokio::sync::Notify;

use crate::channel::Channel;
use crate::error::{AppError, AppResult};
use crate::extension::Extension;
use crate::log::{ColorWheel, Painted, Screen};
use crate::note::{Header, Title};
use crate::router::Router;
use crate::viewer::Viewer;

/// Runs `command` in zsh, writing to `log`, and routes its output to Claude and
/// the viewer. Exits with the command's own code.
///
/// # Errors
///
/// When the log can't be written or zsh can't be started.
///
/// # Panics
///
/// Never: both streams are piped before they're taken.
pub fn run(workspace: &Log, dir: &Path, command: &str) -> AppResult<ExitCode> {
    Ok(ExitCode::from(Prompt::new(workspace, dir, command)?.run()?))
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

struct Prompt<'a> {
    command: &'a str,
    workspace: &'a Log<'a>,
    dir: &'a Path,
    color: Color,
    viewer: Option<Viewer>,
    log: File,
}

impl<'a> Prompt<'a> {
    fn new(workspace: &'a Log<'a>, dir: &'a Path, command: &'a str) -> AppResult<Prompt<'a>> {
        Ok(Prompt {
            command,
            workspace,
            dir,
            color: ColorWheel::turn(workspace)?,
            viewer: Viewer::watching(workspace)?,
            log: workspace.append()?,
        })
    }

    fn run(mut self) -> AppResult<u8> {
        self.header(&mut &self.log, self.color, &Screen::UNBOUNDED)?;
        let mut shell = Command::new("zsh");
        shell
            .arg("-c")
            .arg(self.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let code = runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(Tee::run(shell, &mut self))?;
        self.footer(code)?;
        Ok(code)
    }

    fn footer(&self, code: u8) -> AppResult {
        // The longest exit line is 20 bytes.
        // color 5 + "exit: " 6 + code 3 + color end 5 + newline 1
        let mut line = [0; 32];
        let unused = {
            let mut rest = &mut line[..];
            let color = if code == 0 { self.color } else { Color::Red };
            writeln!(
                rest,
                "{}",
                Painted {
                    color,
                    text: format_args!("exit: {code}")
                }
            )?;
            rest.len()
        };
        (&self.log).write_all(&line[..line.len() - unused])?;
        Ok(())
    }
}

impl Title for Prompt<'_> {
    fn title(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "$ {}", self.command)
    }
}

struct Tee;

impl Tee {
    async fn run(mut shell: Command, prompt: &mut Prompt<'_>) -> AppResult<u8> {
        let Prompt {
            command,
            workspace,
            dir,
            ref log,
            ref mut viewer,
            ..
        } = *prompt;
        let mut watched = viewer
            .take()
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
            Extension::tell_claimed(&workspace.extension_socket_path(), command).await?;
            let follower = workspace.follower_dir(process::id());
            fs::create_dir_all(&follower)?;
            channel
                .serve(
                    File::create(workspace.live_path(process::id()))?,
                    &follower,
                    &workspace.follower_columns_path(process::id()),
                )
                .await
        };
        tokio::try_join!(copying, claim)?;
        let status = child.wait().await?;
        Ok(status
            .code()
            .map_or(u8::MAX, |code| u8::try_from(code).unwrap_or(u8::MAX)))
    }
}

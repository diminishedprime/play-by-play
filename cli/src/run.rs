//! What every Bash command Claude runs goes through: the plugin rewrites each one
//! to `pbp -c '<command>'`.

use std::fmt::{self, Formatter};
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{self, ExitCode, Stdio};

use play_by_play::log::Log;
use play_by_play::{Color, env};
use tokio::process::{Child, Command};
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
        let claim = prompt
            .viewer
            .take()
            .map(|viewer| Claim::open(prompt, viewer))
            .transpose()?;
        if let Some(claim) = &claim {
            shell.env(env::SOCKET, claim.channel.path());
        }
        let mut child = Tee::spawn(shell)?;
        let copied = Notify::new();
        let serving = async {
            match claim {
                Some(claim) => claim.serve(&copied).await,
                None => Ok(()),
            }
        };
        tokio::try_join!(Tee::copy(&mut child, &prompt.log, &copied), serving)?;
        let status = child.wait().await?;
        Ok(status
            .code()
            .map_or(u8::MAX, |code| u8::try_from(code).unwrap_or(u8::MAX)))
    }

    fn spawn(mut shell: Command) -> AppResult<Child> {
        match shell.spawn() {
            Err(error) if error.kind() == ErrorKind::NotFound => Err(AppError::NoZsh),
            spawned => Ok(spawned?),
        }
    }

    async fn copy(child: &mut Child, log: &File, copied: &Notify) -> AppResult {
        let stdout = child.stdout.take().expect("the command's stdout is piped");
        let stderr = child.stderr.take().expect("the command's stderr is piped");
        tokio::try_join!(
            Router::new(log.try_clone()?).copy(stdout, io::stdout()),
            Router::new(log.try_clone()?).copy(stderr, io::stderr()),
        )?;
        copied.notify_one();
        Ok(())
    }
}

struct Claim<'a> {
    channel: Channel,
    viewer: Viewer,
    workspace: &'a Log<'a>,
    command: &'a str,
    live: Option<File>,
}

impl<'a> Claim<'a> {
    fn open(prompt: &Prompt<'a>, viewer: Viewer) -> AppResult<Claim<'a>> {
        Ok(Claim {
            channel: Channel::open(prompt.dir)?,
            viewer,
            workspace: prompt.workspace,
            command: prompt.command,
            live: None,
        })
    }

    async fn serve(mut self, copied: &Notify) -> AppResult {
        loop {
            tokio::select! {
                biased;
                accepted = self.channel.accept() => accepted?,
                () = copied.notified() => return Ok(()),
            }
            self.serve_program().await?;
        }
    }

    async fn serve_program(&mut self) -> AppResult {
        let pid = process::id();
        let columns = self.workspace.follower_columns_path(pid);
        let rows = self.workspace.follower_rows_path(pid);
        self.channel
            .send_size(self.viewer.columns(), self.viewer.rows(), &columns, &rows)
            .await?;
        if self.live.is_none() {
            self.live = Some(self.claim_viewer().await?);
        }
        let Claim {
            channel,
            workspace,
            live: Some(live),
            ..
        } = self
        else {
            return Ok(());
        };
        channel
            .serve(live, &workspace.follower_dir(pid), &columns, &rows)
            .await
    }

    async fn claim_viewer(&self) -> AppResult<File> {
        let pid = process::id();
        Extension::tell_claimed(&self.workspace.extension_socket_path(), self.command).await?;
        fs::create_dir_all(self.workspace.follower_dir(pid))?;
        Ok(File::create(self.workspace.live_path(pid))?)
    }
}

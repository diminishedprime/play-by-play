//! What every Bash command Claude runs goes through: the plugin rewrites each one
//! to `play-by-play run --command '<command>'`.

use std::fs::File;
use std::io::{self, Read, Write};
use std::process::Stdio;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::runtime;

use crate::error::{AppError, AppResult};
use crate::log::{Color, Log, RESET, Screen};
use crate::note::{Header, Title};

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
pub fn run(log: &Log, command: &str) -> AppResult {
    let color = log.next_color()?;
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
        .block_on(Tee::run(shell, &log))?;
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

/// Prints the command the `play-by-play run --command '<command>'` on stdin
/// holds.
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
    async fn run(mut shell: Command, log: &File) -> AppResult<u8> {
        let mut child = shell.spawn()?;
        let stdout = child.stdout.take().expect("the command's stdout is piped");
        let stderr = child.stderr.take().expect("the command's stderr is piped");
        tokio::try_join!(
            Tee::copy(stdout, io::stdout(), log.try_clone()?),
            Tee::copy(stderr, io::stderr(), log.try_clone()?),
        )?;
        let status = child.wait().await?;
        Ok(status
            .code()
            .map_or(u8::MAX, |code| u8::try_from(code).unwrap_or(u8::MAX)))
    }

    async fn copy(
        mut from: impl AsyncRead + Unpin,
        mut claude: impl Write,
        mut log: impl Write,
    ) -> AppResult {
        let mut chunk = [0; 8192];
        loop {
            let read = from.read(&mut chunk).await?;
            if read == 0 {
                return Ok(());
            }
            claude.write_all(&chunk[..read])?;
            claude.flush()?;
            log.write_all(&chunk[..read])?;
        }
    }
}

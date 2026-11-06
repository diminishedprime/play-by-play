use std::env;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io;
use std::iter;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::process::Command;

use rustix::pty::{self, OpenptFlags};
use tokio::io::Interest;
use tokio::io::unix::AsyncFd;

use super::ran::Ran;
use super::spawn::{Spawned, Stdio};

pub struct Sut {
    command: Command,
    stdin: String,
    fakes: PathBuf,
}

impl Sut {
    pub fn new(dir: &Path, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Sut {
        let mut command = Command::new(Sut::binary());
        command.args(args);
        Sut::within_world(dir, command)
    }

    pub fn in_terminal(dir: &Path, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Sut {
        let mut command = Command::new(env!("CARGO_BIN_EXE_in-terminal"));
        command.arg(Sut::binary()).args(args);
        Sut::within_world(dir, command)
    }

    fn binary() -> String {
        env::var("PLAY_BY_PLAY_BIN").expect("PLAY_BY_PLAY_BIN names the play-by-play binary")
    }

    fn within_world(dir: &Path, mut command: Command) -> Sut {
        let path = env::join_paths(
            iter::once(dir.join("bin"))
                .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
        )
        .expect("the fake tools' folder joins PATH");
        command
            .env("TMPDIR", dir)
            .env("PATH", path)
            .env_remove("CLAUDE_PROJECT_DIR")
            .env_remove("TMUX")
            .env("CARGO_HOME", dir.join("cargo"));
        Sut {
            command,
            stdin: String::new(),
            fakes: dir.join("bin"),
        }
    }

    pub fn inside_tmux(mut self) -> Self {
        self.command.env("TMUX", "/tmp/tmux-501/default,1,0");
        self
    }

    pub fn only_fakes_on_path(mut self) -> Self {
        self.command.env("PATH", &self.fakes);
        self
    }

    pub fn project(mut self, root: &Path) -> Self {
        self.command.env("CLAUDE_PROJECT_DIR", root);
        self
    }

    pub fn within(mut self, folder: &Path) -> Self {
        self.command.current_dir(folder);
        self
    }

    pub fn stdin(mut self, input: &str) -> Self {
        input.clone_into(&mut self.stdin);
        self
    }

    pub async fn run(self) -> Ran {
        Ran::new(
            Spawned::new(&self.command, [Stdio::Pipe, Stdio::Pipe, Stdio::Pipe])
                .output(self.stdin.into_bytes())
                .await,
        )
    }

    pub async fn run_in_terminal(self) -> Ran {
        let terminal =
            pty::openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("a pseudo-terminal opens");
        pty::grantpt(&terminal).expect("the pseudo-terminal is granted");
        pty::unlockpt(&terminal).expect("the pseudo-terminal unlocks");
        let name = pty::ptsname(&terminal, Vec::new()).expect("the pseudo-terminal has a name");
        let screen = OpenOptions::new()
            .write(true)
            .open(name.to_str().expect("the pseudo-terminal's name is UTF-8"))
            .expect("the pseudo-terminal's screen opens");
        rustix::io::ioctl_fionbio(&terminal, true).expect("the pseudo-terminal stops blocking");
        let terminal =
            unsafe { AsyncFd::register(terminal) }.expect("the pseudo-terminal can be awaited");
        let spawned = Spawned::new(
            &self.command,
            [Stdio::Null, Stdio::Fd(screen.into()), Stdio::Pipe],
        );
        let (seen, mut output) =
            tokio::join!(Sut::read_terminal(&terminal), spawned.output(Vec::new()));
        output.stdout = String::from_utf8(seen)
            .expect("play-by-play prints UTF-8")
            .replace("\r\n", "\n")
            .into_bytes();
        Ran::new(output)
    }

    async fn read_terminal(terminal: &AsyncFd<OwnedFd>) -> Vec<u8> {
        let mut seen = Vec::new();
        let mut chunk = [0; 4096];
        while let Ok(read @ 1..) = terminal
            .async_io(Interest::READABLE, |fd| {
                rustix::io::read(fd, &mut chunk).map_err(io::Error::from)
            })
            .await
        {
            seen.extend_from_slice(&chunk[..read]);
        }
        seen
    }

    pub fn start(self) -> Running {
        Running(Spawned::new(
            &self.command,
            [Stdio::Pipe, Stdio::Pipe, Stdio::Pipe],
        ))
    }

    pub fn into_command(self) -> Command {
        self.command
    }
}

pub struct Running(Spawned);

impl Running {
    pub async fn finishes(self) -> Ran {
        Ran::new(self.0.output(Vec::new()).await)
    }
}

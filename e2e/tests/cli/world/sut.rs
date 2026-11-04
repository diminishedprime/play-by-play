use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::iter;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;

use rustix::pty::{self, OpenptFlags};

use super::ran::Ran;

pub struct Sut {
    command: Command,
    stdin: String,
    fakes: PathBuf,
}

impl Sut {
    pub fn new(dir: &Path, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Sut {
        let binary =
            env::var("PLAY_BY_PLAY_BIN").expect("PLAY_BY_PLAY_BIN names the play-by-play binary");
        let path = env::join_paths(
            iter::once(dir.join("bin"))
                .chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())),
        )
        .expect("the fake tools' folder joins PATH");
        let mut command = Command::new(binary);
        command
            .args(args)
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

    pub fn run(mut self) -> Ran {
        let mut child = self
            .command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("play-by-play starts");
        child
            .stdin
            .take()
            .expect("play-by-play's stdin is piped")
            .write_all(self.stdin.as_bytes())
            .expect("play-by-play takes its stdin");
        Ran::new(child.wait_with_output().expect("play-by-play finishes"))
    }

    pub fn run_in_terminal(self) -> Ran {
        let terminal =
            pty::openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("a pseudo-terminal opens");
        pty::grantpt(&terminal).expect("the pseudo-terminal is granted");
        pty::unlockpt(&terminal).expect("the pseudo-terminal unlocks");
        let name = pty::ptsname(&terminal, Vec::new()).expect("the pseudo-terminal has a name");
        let screen = OpenOptions::new()
            .write(true)
            .open(name.to_str().expect("the pseudo-terminal's name is UTF-8"))
            .expect("the pseudo-terminal's screen opens");
        let mut terminal = File::from(terminal);
        let reading = thread::spawn(move || {
            let mut seen = Vec::new();
            let mut chunk = [0; 4096];
            while let Ok(read @ 1..) = terminal.read(&mut chunk) {
                seen.extend_from_slice(&chunk[..read]);
            }
            seen
        });
        let mut command = self.command;
        let child = command
            .stdin(Stdio::null())
            .stdout(screen)
            .stderr(Stdio::piped())
            .spawn()
            .expect("play-by-play starts");
        drop(command);
        let mut output = child.wait_with_output().expect("play-by-play finishes");
        let seen = reading.join().expect("the terminal is read");
        output.stdout = String::from_utf8(seen)
            .expect("play-by-play prints UTF-8")
            .replace("\r\n", "\n")
            .into_bytes();
        Ran::new(output)
    }

    pub fn start(mut self) -> Running {
        Running(
            self.command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("play-by-play starts"),
        )
    }

    pub fn into_command(self) -> Command {
        self.command
    }
}

pub struct Running(Child);

impl Running {
    pub fn finishes(self) -> Ran {
        Ran::new(self.0.wait_with_output().expect("play-by-play finishes"))
    }
}

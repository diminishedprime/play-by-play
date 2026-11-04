use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::iter;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use super::ran::Ran;

pub struct Sut {
    command: Command,
    stdin: String,
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
            .env_remove("CLAUDE_PROJECT_DIR");
        Sut {
            command,
            stdin: String::new(),
        }
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

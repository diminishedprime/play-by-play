use std::fs::{File as StdFile, OpenOptions};
use std::iter;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::time::Duration;

use rustix::process;
use rustix::pty::{self, OpenptFlags};
use rustix::termios::{self, Winsize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time;

use super::style::Style;
use super::sut::Sut;

pub struct Viewer {
    _watch: Child,
    output: UnboundedReceiver<Vec<u8>>,
    screen: StdFile,
    seen: Vec<u8>,
}

impl Viewer {
    pub const COLUMNS: u16 = 77;
    pub const BACKGROUND: &[u8] = b"\x1b]11;rgb:0000/2b2b/3636\x1b\\";

    pub fn start(dir: &Path, workspace: &Path) -> Viewer {
        let terminal =
            pty::openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("a pseudo-terminal opens");
        pty::grantpt(&terminal).expect("the pseudo-terminal is granted");
        pty::unlockpt(&terminal).expect("the pseudo-terminal unlocks");
        let name = pty::ptsname(&terminal, Vec::new()).expect("the pseudo-terminal has a name");
        let screen = OpenOptions::new()
            .read(true)
            .write(true)
            .open(name.to_str().expect("the pseudo-terminal's name is UTF-8"))
            .expect("the pseudo-terminal's screen opens");
        Viewer::size(&screen, Viewer::COLUMNS);
        let mut command = Sut::new(dir, ["watch"]).within(workspace).into_command();
        command.env("TERM", "xterm-256color");
        let controlling = screen.try_clone().expect("the screen is shared");
        unsafe {
            command.pre_exec(move || {
                process::setsid()?;
                process::ioctl_tiocsctty(&controlling)?;
                Ok(())
            });
        }
        let watch = Command::from(command)
            .stdin(screen.try_clone().expect("the screen is shared"))
            .stdout(screen.try_clone().expect("the screen is shared"))
            .stderr(screen.try_clone().expect("the screen is shared"))
            .kill_on_drop(true)
            .spawn()
            .expect("watch starts");
        let answers = File::from_std(
            terminal
                .try_clone()
                .expect("the pseudo-terminal is shared")
                .into(),
        );
        let (sender, output) = mpsc::unbounded_channel();
        tokio::spawn(Viewer::answer(
            File::from_std(terminal.into()),
            answers,
            sender,
        ));
        Viewer {
            _watch: watch,
            output,
            screen,
            seen: Vec::new(),
        }
    }

    async fn answer(mut terminal: File, mut answers: File, output: UnboundedSender<Vec<u8>>) {
        let mut chunk = [0; 4096];
        while let Ok(read @ 1..) = terminal.read(&mut chunk).await {
            let bytes = &chunk[..read];
            if Viewer::asks(bytes, b"\x1b]11;?") {
                let _ = answers.write_all(Viewer::BACKGROUND).await;
            }
            if Viewer::asks(bytes, b"\x1b[c") {
                let _ = answers.write_all(b"\x1b[?62;22c").await;
            }
            if output.send(bytes.to_vec()).is_err() {
                return;
            }
        }
    }

    fn asks(bytes: &[u8], query: &[u8]) -> bool {
        bytes.windows(query.len()).any(|window| window == query)
    }

    pub fn resize(&self, columns: u16) {
        Viewer::size(&self.screen, columns);
    }

    fn size(screen: &StdFile, columns: u16) {
        termios::tcsetwinsize(
            screen,
            Winsize {
                ws_row: 24,
                ws_col: columns,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )
        .expect("the pseudo-terminal takes a size");
    }

    pub async fn sees(&mut self, expected: &[&str]) {
        let _ = time::timeout(Duration::from_secs(1), self.watch_for(expected)).await;
        assert_eq!(
            self.screen_lines(),
            expected,
            "the screen: {:?}",
            String::from_utf8_lossy(&self.seen)
        );
    }

    async fn watch_for(&mut self, expected: &[&str]) {
        while self.screen_lines() != expected {
            match self.output.recv().await {
                Some(bytes) => self.seen.extend_from_slice(&bytes),
                None => return,
            }
        }
    }

    pub fn colors(&self) -> Vec<Style> {
        String::from_utf8_lossy(&self.seen)
            .split("\x1b[H\x1b[2J")
            .skip(1)
            .map(|screen| {
                let (code, _) = screen
                    .strip_prefix("\x1b[")
                    .and_then(|header| header.split_once('m'))
                    .expect("each screen starts with its header's color");
                Style::of(code)
            })
            .collect()
    }

    pub fn style_of(&self, text: &str) -> Option<Style> {
        let seen = String::from_utf8_lossy(&self.seen);
        let at = seen.find(text)?;
        let (_, code) = seen[..at].strip_suffix('m')?.rsplit_once("\x1b[")?;
        Some(Style::of(code))
    }

    pub fn exit_colors(&self) -> Vec<Style> {
        let seen = String::from_utf8_lossy(&self.seen);
        seen.match_indices("mexit: ")
            .map(|(at, _)| {
                let (_, code) = seen[..at]
                    .rsplit_once("\x1b[")
                    .expect("each exit line starts with its color");
                Style::of(code)
            })
            .collect()
    }

    fn screen_lines(&self) -> Vec<String> {
        let seen = String::from_utf8_lossy(&self.seen);
        let screen = seen
            .rsplit_once("\x1b[H\x1b[2J")
            .map_or(seen.as_ref(), |(_, screen)| screen);
        Viewer::without_escapes(screen)
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn without_escapes(screen: &str) -> String {
        let mut chunks = screen.split('\x1b');
        let first = chunks.next().unwrap_or_default();
        iter::once(first)
            .chain(chunks.map(|chunk| {
                if let Some(style) = chunk.strip_prefix('[') {
                    style
                        .split_once(|c: char| c.is_ascii_alphabetic())
                        .map_or("", |(_, text)| text)
                } else {
                    chunk.strip_prefix('\\').unwrap_or_default()
                }
            }))
            .collect()
    }
}

use std::fs::{File as StdFile, OpenOptions};
use std::io;
use std::num::NonZeroU16;
use std::os::fd::OwnedFd;
use std::path::Path;
use std::process::ExitStatus;
use std::time::Duration;

use rustix::pty::{self, OpenptFlags};
use rustix::termios::{self, Winsize};
use tokio::io::Interest;
use tokio::io::unix::AsyncFd;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time;

use super::spawn::{Spawned, Stdio};
use super::style::Style;
use super::sut::Sut;

pub struct Viewer {
    watch: Spawned,
    reader: JoinHandle<()>,
    output: UnboundedReceiver<Vec<u8>>,
    screen: StdFile,
    seen: Vec<u8>,
    shown: vt100::Parser,
    screens: Vec<Vec<String>>,
}

impl Drop for Viewer {
    fn drop(&mut self) {
        self.reader.abort();
    }
}

impl Viewer {
    pub const ROWS: u16 = 24;
    pub const COLUMNS: u16 = 77;
    pub const FOLLOWER_COLUMNS: u16 = 38;
    pub const BACKGROUND: &[u8] = b"\x1b]11;rgb:0000/2b2b/3636\x1b\\";
    const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

    pub fn start(dir: &Path, workspace: &Path) -> Viewer {
        let watch = Sut::in_terminal(dir, ["watch"]).within(workspace);
        Viewer::running(watch, Viewer::COLUMNS)
    }

    pub fn start_for(dir: &Path, workspace: &Path, project: &Path) -> Viewer {
        let watch = Sut::in_terminal(dir, ["watch"])
            .within(workspace)
            .project(project);
        Viewer::running(watch, Viewer::COLUMNS)
    }

    pub fn follow(dir: &Path, workspace: &Path, pid: &str) -> Viewer {
        let follow = Sut::in_terminal(dir, ["follow", pid]).within(workspace);
        Viewer::running(follow, Viewer::FOLLOWER_COLUMNS)
    }

    fn running(sut: Sut, columns: u16) -> Viewer {
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
        Viewer::size(&screen, columns);
        let mut command = sut.into_command();
        command.env("TERM", "xterm-256color");
        let watch = Spawned::new(
            &command,
            [
                Stdio::Fd(screen.try_clone().expect("the screen is shared").into()),
                Stdio::Fd(screen.try_clone().expect("the screen is shared").into()),
                Stdio::Fd(screen.try_clone().expect("the screen is shared").into()),
            ],
        );
        rustix::io::ioctl_fionbio(&terminal, true).expect("the pseudo-terminal stops blocking");
        let terminal =
            unsafe { AsyncFd::register(terminal) }.expect("the pseudo-terminal can be awaited");
        let (sender, output) = mpsc::unbounded_channel();
        Viewer {
            watch,
            reader: tokio::spawn(Viewer::answer(terminal, sender)),
            output,
            screen,
            seen: Vec::new(),
            shown: vt100::Parser::new(Viewer::rows(), Viewer::columns(columns), 0),
            screens: Vec::new(),
        }
    }

    async fn answer(terminal: AsyncFd<OwnedFd>, output: UnboundedSender<Vec<u8>>) {
        let mut chunk = [0; 4096];
        while let Ok(read @ 1..) = terminal
            .async_io(Interest::READABLE, |fd| {
                rustix::io::read(fd, &mut chunk).map_err(io::Error::from)
            })
            .await
        {
            let bytes = &chunk[..read];
            if Viewer::asks(bytes, b"\x1b]11;?") {
                let _ = rustix::io::write(terminal.get_ref(), Viewer::BACKGROUND);
            }
            if Viewer::asks(bytes, b"\x1b[c") {
                let _ = rustix::io::write(terminal.get_ref(), b"\x1b[?62;22c");
            }
            if output.send(bytes.to_vec()).is_err() {
                return;
            }
        }
    }

    fn asks(bytes: &[u8], query: &[u8]) -> bool {
        bytes.windows(query.len()).any(|window| window == query)
    }

    pub async fn close(mut self) {
        self.reader.abort();
        let _ = (&mut self.reader).await;
        let hung_up = time::timeout(Duration::from_secs(1), self.watch.exit()).await;
        if hung_up.is_err() {
            self.watch.kill().await;
        }
    }

    pub async fn hang_up(mut self) -> ExitStatus {
        self.reader.abort();
        let _ = (&mut self.reader).await;
        self.watch.exit().await
    }

    pub fn resize(&mut self, columns: u16) {
        Viewer::size(&self.screen, columns);
        self.shown
            .set_size(Viewer::rows(), Viewer::columns(columns));
    }

    fn rows() -> NonZeroU16 {
        NonZeroU16::new(Viewer::ROWS).expect("the screen has rows")
    }

    fn columns(columns: u16) -> NonZeroU16 {
        NonZeroU16::new(columns).expect("the screen has columns")
    }

    fn size(screen: &StdFile, columns: u16) {
        termios::tcsetwinsize(
            screen,
            Winsize {
                ws_row: Viewer::ROWS,
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
                Some(bytes) => {
                    self.shown.process(&bytes);
                    self.seen.extend_from_slice(&bytes);
                    let screen = self.screen_lines();
                    if self.screens.last() != Some(&screen) {
                        self.screens.push(screen);
                    }
                }
                None => return,
            }
        }
    }

    pub fn turned_spinners_in_order(&self, rows: &[&str]) {
        for row in rows {
            let frames: Vec<char> = self
                .screens
                .iter()
                .filter_map(|screen| Viewer::spinner_on(screen, row))
                .collect();
            let jumps: Vec<(char, char)> = frames
                .windows(2)
                .map(|pair| (pair[0], pair[1]))
                .filter(|&(from, to)| from != to && Viewer::next_frame(from) != to)
                .collect();
            assert!(
                jumps.is_empty(),
                "row {row}'s spinner jumped {jumps:?}, its frames were {frames:?}"
            );
        }
    }

    fn spinner_on(screen: &[String], row: &str) -> Option<char> {
        screen
            .iter()
            .find_map(|line| line.strip_prefix(row)?.strip_prefix(' ')?.chars().next())
    }

    fn next_frame(frame: char) -> char {
        let at = Viewer::SPINNER
            .iter()
            .position(|&shown| shown == frame)
            .expect("the row shows a spinner frame");
        Viewer::SPINNER[(at + 1) % Viewer::SPINNER.len()]
    }

    pub fn printed(&self, text: &str) -> bool {
        String::from_utf8_lossy(&self.seen).contains(text)
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
        self.shown
            .screen()
            .contents()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

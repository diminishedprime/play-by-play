//! Each workspace's files share one name under the log directory, told apart by
//! their extension.

use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::iter;
use std::os::unix::ffi::OsStrExt;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

use clap::ValueEnum;
use sha2::{Digest, Sha256};

use crate::error::AppResult;

/// Where the commands Claude runs in one workspace are copied.
pub struct Log<'a> {
    dir: &'a Path,
    name: &'a OsStr,
    hex: [u8; 12],
}

impl<'a> Log<'a> {
    const COLORS: [Color; 5] = [
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];

    /// The log for the workspace at `root`: one per root, so every command in a
    /// window shares it, even inside a polyrepo's subrepos.
    #[must_use]
    pub fn for_workspace(dir: &'a Path, root: &'a Path) -> Log<'a> {
        Log {
            dir,
            name: root.file_name().unwrap_or_default(),
            hex: Log::hex(root),
        }
    }

    /// What `play-by-play watch` follows and every command is copied into.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.file(&["log"])
    }

    /// The log file, opened to read and append.
    ///
    /// # Errors
    ///
    /// When the folder can't be made or the log can't be opened.
    pub fn append(&self) -> AppResult<File> {
        fs::create_dir_all(self.dir)?;
        Ok(OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(self.path())?)
    }

    /// Holds the viewer's pid, so `run` can tell whether anyone is still watching.
    #[must_use]
    pub fn viewer_path(&self) -> PathBuf {
        self.file(&["viewer"])
    }

    /// Holds the viewer's width, rewritten whenever its terminal resizes.
    #[must_use]
    pub fn columns_path(&self) -> PathBuf {
        self.file(&["columns"])
    }

    /// Holds the name of the code theme closest to the viewer's background.
    #[must_use]
    pub fn theme_path(&self) -> PathBuf {
        self.file(&["theme"])
    }

    /// What the viewer has said about its terminal so far.
    #[must_use]
    pub fn screen(&self) -> Screen {
        Screen {
            columns: fs::read_to_string(self.columns_path())
                .ok()
                .and_then(|columns| columns.parse().ok())
                .unwrap_or(u16::MAX),
            theme: fs::read_to_string(self.theme_path()).ok(),
        }
    }

    /// Holds the color the next header uses, so consecutive commands alternate.
    #[must_use]
    pub fn color_path(&self) -> PathBuf {
        self.file(&["color"])
    }

    /// The color the next header uses, moving the workspace on to the one after.
    ///
    /// # Errors
    ///
    /// When the folder can't be made or the color can't be saved.
    pub fn next_color(&self) -> AppResult<Color> {
        let path = self.color_path();
        let at = fs::read_to_string(&path)
            .ok()
            .and_then(|at| at.parse::<usize>().ok())
            .unwrap_or(0);
        fs::create_dir_all(self.dir)?;
        fs::write(&path, ((at + 1) % Log::COLORS.len()).to_string())?;
        Ok(Log::COLORS[at % Log::COLORS.len()])
    }

    /// Where a live program under the `run` with `pid` writes its view, which
    /// `play-by-play follow` shows.
    #[must_use]
    pub fn live_path(&self, pid: u32) -> PathBuf {
        self.file(&[Digits::from(pid).as_str(), "live"])
    }

    /// Holds only what the follower of the live program under the `run` with
    /// `pid` records, so every change in it is the follower's.
    #[must_use]
    pub fn follower_dir(&self, pid: u32) -> PathBuf {
        self.file(&[Digits::from(pid).as_str()])
    }

    /// Holds the follower's width, rewritten whenever its terminal resizes.
    #[must_use]
    pub fn follower_columns_path(&self, pid: u32) -> PathBuf {
        self.follower_dir(pid).join("columns")
    }

    /// Where the viewer's width is written before it's renamed into
    /// [`Log::columns_path`], so nobody reads it half written.
    #[must_use]
    pub fn columns_temporary_path(&self) -> PathBuf {
        self.file_in(&[Log::TEMPORARY], &["columns"])
    }

    /// Where the viewer's code theme is written before it's renamed into
    /// [`Log::theme_path`], so nobody reads it half written.
    #[must_use]
    pub fn theme_temporary_path(&self) -> PathBuf {
        self.file_in(&[Log::TEMPORARY], &["theme"])
    }

    /// Where the follower's width is written before it's renamed into
    /// [`Log::follower_columns_path`], outside the follower's watched folder.
    #[must_use]
    pub fn follower_columns_temporary_path(&self, pid: u32) -> PathBuf {
        self.file_in(&[Log::TEMPORARY], &[Digits::from(pid).as_str(), "columns"])
    }

    const TEMPORARY: &str = "tmp";

    fn file(&self, extensions: &[&str]) -> PathBuf {
        self.file_in(&[], extensions)
    }

    fn file_in(&self, folders: &[&str], extensions: &[&str]) -> PathBuf {
        let name = [self.name, OsStr::new("_"), OsStr::from_bytes(&self.hex)];
        let length = self.dir.as_os_str().len()
            + folders.iter().map(|folder| folder.len() + 1).sum::<usize>()
            + 1
            + name.iter().map(|part| part.len()).sum::<usize>()
            + extensions
                .iter()
                .map(|extension| extension.len() + 1)
                .sum::<usize>();
        let path = iter::once(self.dir.as_os_str())
            .chain(
                folders
                    .iter()
                    .flat_map(|folder| [OsStr::new(MAIN_SEPARATOR_STR), OsStr::new(folder)]),
            )
            .chain(iter::once(OsStr::new(MAIN_SEPARATOR_STR)))
            .chain(name)
            .chain(
                extensions
                    .iter()
                    .flat_map(|extension| [OsStr::new("."), OsStr::new(extension)]),
            )
            .fold(OsString::with_capacity(length), |mut path, part| {
                path.push(part);
                path
            });
        PathBuf::from(path)
    }

    fn hex(root: &Path) -> [u8; 12] {
        let digest = Sha256::digest(root.as_os_str().as_encoded_bytes());
        let mut hex = [0; 12];
        hex::encode_to_slice(&digest[..6], &mut hex).expect("6 bytes are 12 hex digits");
        hex
    }
}

struct Digits {
    bytes: [u8; 10],
    length: usize,
}

impl From<u32> for Digits {
    fn from(number: u32) -> Digits {
        let mut bytes = [0; 10];
        let unused = {
            let mut rest = &mut bytes[..];
            write!(rest, "{number}").expect("a u32 fits in 10 digits");
            rest.len()
        };
        Digits {
            bytes,
            length: bytes.len() - unused,
        }
    }
}

impl Digits {
    fn as_str(&self) -> &str {
        str::from_utf8(&self.bytes[..self.length]).expect("digits are UTF-8")
    }
}

/// The viewer's terminal, as notes write for it.
pub struct Screen {
    /// How wide it is, or [`u16::MAX`] when no viewer has said.
    pub columns: u16,
    /// The code theme closest to its background, or bat's own default when no
    /// viewer has said.
    pub theme: Option<String>,
}

impl Screen {
    /// A screen with no width to wrap to and bat's default theme.
    pub const UNBOUNDED: Screen = Screen {
        columns: u16::MAX,
        theme: None,
    };
}

/// Clears the viewer's screen and moves to its top, so each header starts a
/// fresh screen.
pub const CLEAR: &str = "\x1b[H\x1b[2J";

/// Ends a [`Color`], [`BOLD`], or [`GRAY`].
pub const RESET: &str = "\x1b[0m";

/// Starts bold text, ended by [`RESET`].
pub const BOLD: &str = "\x1b[1m";

/// Ends [`BOLD`] while keeping the current color.
pub const NORMAL: &str = "\x1b[22m";

/// Starts gray text, ended by [`RESET`].
pub const GRAY: &str = "\x1b[90m";
/// A color the viewer's headers and exit lines are written in; it displays as
/// the escape that switches to it.
#[derive(Clone, Copy, ValueEnum)]
pub enum Color {
    /// Kept for a failed command's exit line.
    Red,
    /// The first header color.
    Green,
    /// A header color.
    Yellow,
    /// A header color.
    Blue,
    /// A header color.
    Magenta,
    /// The last header color, before the cycle starts over.
    Cyan,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code = match self {
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
        };
        write!(f, "\x1b[{code}m")
    }
}

/// Text the viewer's terminal opens `url` from when it's clicked.
pub struct Link<'a> {
    /// Where the link goes.
    pub url: &'a str,
    /// What the link shows.
    pub text: &'a str,
}

impl Display for Link<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\x1b]8;;{}\x1b\\{}{}{RESET}\x1b]8;;\x1b\\",
            self.url,
            Color::Blue,
            self.text
        )
    }
}

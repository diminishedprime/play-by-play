//! Each workspace's files share one name under the log directory, told apart by
//! their extension.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::iter;
use std::os::unix::ffi::OsStrExt;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

use sha2::{Digest, Sha256};

/// Where the commands Claude runs in one workspace are copied.
pub struct Log<'a> {
    dir: &'a Path,
    name: &'a OsStr,
    hex: [u8; 12],
}

impl<'a> Log<'a> {
    /// The folder every workspace's files are in, under the system's temporary
    /// folder.
    #[must_use]
    pub fn folder() -> PathBuf {
        env::temp_dir().join("pbp")
    }

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

    /// The folder every workspace's files are in.
    #[must_use]
    pub fn dir(&self) -> &Path {
        self.dir
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
    pub fn append(&self) -> io::Result<File> {
        fs::create_dir_all(self.dir)?;
        OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(self.path())
    }

    /// Holds the viewer's pid, so `run` can tell whether anyone is still watching.
    #[must_use]
    pub fn viewer_path(&self) -> PathBuf {
        self.file(&["viewer"])
    }

    /// Where the VS Code extension listens to hear which commands claim the
    /// viewer.
    #[must_use]
    pub fn extension_socket_path(&self) -> PathBuf {
        let mut name = OsString::from(OsStr::from_bytes(&self.hex));
        name.push(".e.sock");
        self.dir.join(name)
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

    /// Holds the color the next header uses, so consecutive commands alternate.
    #[must_use]
    pub fn color_path(&self) -> PathBuf {
        self.file(&["color"])
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

mod claude;
mod ran;
mod style;
mod sut;
mod viewer;

use std::ffi::OsStr;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{Config, KqueueWatcher, RecursiveMode, Watcher};
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio::time;

pub use claude::Claude;
pub use style::Style;
pub use sut::Sut;
pub use viewer::Viewer;

pub struct World {
    viewer: Viewer,
    dir: TempDir,
    workspace: PathBuf,
}

impl World {
    pub fn new() -> World {
        let dir =
            TempDir::with_prefix("play-by-play-e2e-").expect("the world's folder can be made");
        let workspace = dir.path().join("workspace");
        fs::create_dir(&workspace).expect("the workspace can be made");
        let workspace = fs::canonicalize(workspace).expect("the workspace resolves");
        let bin = dir.path().join("bin");
        fs::create_dir(&bin).expect("the fake tools' folder can be made");
        let tmux = bin.join("tmux");
        fs::write(
            &tmux,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
                dir.path().join("tmux.args").display()
            ),
        )
        .expect("the fake tmux can be written");
        fs::set_permissions(&tmux, Permissions::from_mode(0o755))
            .expect("the fake tmux can be made runnable");
        fs::create_dir(dir.path().join("play-by-play")).expect("the log folder can be made");
        World {
            viewer: Viewer::start(dir.path(), &workspace),
            dir,
            workspace,
        }
    }

    pub fn sut(&self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Sut {
        Sut::new(self.dir(), args)
    }

    pub fn claude(&self) -> Claude<'_> {
        self.into()
    }

    pub fn viewer(&mut self) -> &mut Viewer {
        &mut self.viewer
    }

    pub fn viewer_width(&self) -> String {
        self.viewer_recorded("columns")
            .expect("the viewer recorded its width")
    }

    pub fn viewer_theme(&self) -> String {
        self.viewer_recorded("theme")
            .expect("the viewer recorded its theme")
    }

    pub async fn viewer_width_becomes(&self, width: u16) {
        self.viewer_records("columns", &width.to_string()).await;
    }

    pub async fn viewer_theme_becomes(&self, theme: &str) {
        self.viewer_records("theme", theme).await;
    }

    async fn viewer_records(&self, extension: &str, value: &str) {
        let (sender, mut changes) = mpsc::unbounded_channel();
        let mut watcher = KqueueWatcher::new(
            move |change| {
                let _ = sender.send(change);
            },
            Config::default(),
        )
        .expect("the log folder can be watched");
        watcher
            .watch(&self.dir().join("play-by-play"), RecursiveMode::Recursive)
            .expect("the log folder is watched");
        let _ = time::timeout(Duration::from_secs(1), async {
            while self.viewer_recorded(extension).as_deref() != Some(value)
                && changes.recv().await.is_some()
            {}
        })
        .await;
        assert_eq!(self.viewer_recorded(extension).as_deref(), Some(value));
    }

    fn viewer_recorded(&self, extension: &str) -> Option<String> {
        let file = fs::read_dir(self.dir().join("play-by-play"))
            .expect("the log folder exists")
            .map(|entry| entry.expect("the log folder lists").path())
            .find(|path| path.extension() == Some(OsStr::new(extension)))?;
        Some(fs::read_to_string(file).expect("the viewer's file reads"))
    }

    pub fn tmux_ran(&self, args: &[&str]) {
        let ran = fs::read_to_string(self.dir().join("tmux.args")).expect("tmux was run");
        assert_eq!(ran.lines().collect::<Vec<_>>(), args);
    }

    fn dir(&self) -> &Path {
        self.dir.path()
    }

    fn workspace(&self) -> &Path {
        &self.workspace
    }
}

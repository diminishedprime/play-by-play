use std::fmt::{self, Display, Formatter, Write as _};

use super::{Assessment, Check, Facts, Fix};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MISMATCH: &str = "mismatch:";
const LINE_START: &str = "\n    ";
const SEPARATOR: &str = " - ";

#[derive(Clone, Copy)]
struct Installed<'a> {
    name: &'static str,
    version: &'a str,
}

impl Installed<'_> {
    fn columns(&self) -> usize {
        console::measure_text_width(self.version)
    }

    fn padding(&self, width: usize) -> usize {
        width.saturating_sub(self.columns())
    }
}

impl Display for Installed<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let padding = self.padding(f.width().unwrap_or_default());
        write!(f, "{}{:padding$}{SEPARATOR}{}", self.version, "", self.name)
    }
}

#[derive(Default)]
struct Layout {
    width: usize,
    lines: usize,
    text: usize,
    columns: usize,
}

impl Layout {
    fn of<'a>(installed: impl Iterator<Item = Installed<'a>>) -> Layout {
        installed.fold(Layout::default(), |layout, piece| {
            let columns = piece.columns();
            Layout {
                width: layout.width.max(columns),
                lines: layout.lines + 1,
                text: layout.text + piece.version.len() + piece.name.len(),
                columns: layout.columns + columns,
            }
        })
    }

    fn capacity(&self) -> usize {
        MISMATCH.len() + self.lines * (LINE_START.len() + SEPARATOR.len() + self.width) + self.text
            - self.columns
    }
}

pub(super) struct Versions;

impl Versions {
    fn installed(facts: &Facts) -> impl Iterator<Item = Installed<'_>> + Clone {
        let play_by_play = Installed {
            name: "play-by-play",
            version: VERSION,
        };
        let plugin = facts.plugin_version().map(|version| Installed {
            name: "Claude plugin",
            version,
        });
        let extension = facts.extension_version().map(|version| Installed {
            name: "VS Code extension",
            version,
        });
        [Some(play_by_play), plugin, extension]
            .into_iter()
            .flatten()
    }

    fn compare<'a>(installed: impl Iterator<Item = Installed<'a>> + Clone) -> Assessment<'a> {
        if installed.clone().all(|piece| piece.version == VERSION) {
            return Self::passed(format!("all versions match at {VERSION}"));
        }
        let layout = Layout::of(installed.clone());
        let width = layout.width;
        let mut mismatch = String::with_capacity(layout.capacity());
        mismatch.push_str(MISMATCH);
        let mismatch = installed.fold(mismatch, |mut mismatch, piece| {
            write!(mismatch, "{LINE_START}{piece:width$}")
                .expect("a version line writes into a String");
            mismatch
        });
        Self::failed(mismatch, ())
    }
}

impl Check for Versions {
    const LABEL: &str = "Version:";
    type Problem<'a> = ();

    fn suggested_fix<'a>((): ()) -> Fix<'a> {
        Fix::Text("run `just install` from the play-by-play repo")
    }

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>> {
        let comparable = facts.plugin_version().is_some() || facts.extension_version().is_some();
        comparable
            .then(|| Versions::compare(Versions::installed(facts)))
            .into_iter()
    }
}

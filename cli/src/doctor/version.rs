use std::fmt::{self, Display, Formatter};

use super::{Assessment, Check, Facts, Fix};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
struct Installed<'a> {
    name: &'static str,
    version: &'a str,
}

impl Display for Installed<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let width = f.width().unwrap_or_default();
        write!(f, "{:<width$} - {}", self.version, self.name)
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
        let width = installed
            .clone()
            .map(|piece| piece.version.len())
            .max()
            .unwrap_or_default();
        let listed = installed
            .map(|piece| format!("\n    {piece:width$}"))
            .collect::<String>();
        Self::failed(format!("mismatch:{listed}"), ())
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

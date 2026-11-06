use std::process::Command;

use crate::error::AppResult;

use super::{Assessment, Check, Facts, Fix, OnPath};

const ID: &str = "diminishedprime.play-by-play-vscode@";

pub(super) struct Extension;

impl Extension {
    pub(super) fn listed_version() -> AppResult<Option<String>> {
        let listed = Command::new("code")
            .args(["--list-extensions", "--show-versions"])
            .output()?;
        Ok(String::from_utf8_lossy(&listed.stdout)
            .lines()
            .find_map(|line| line.strip_prefix(ID))
            .map(str::to_owned))
    }
}

impl Check for Extension {
    const LABEL: &str = "Extension:";
    type Problem<'a> = ();

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>> {
        match &facts.extension {
            OnPath::Unchecked => None,
            OnPath::Missing => Some(Self::failed("not installed", ())),
            OnPath::Found(_) => Some(Self::passed("installed")),
        }
        .into_iter()
    }

    fn suggested_fix<'a>((): ()) -> Fix<'a> {
        Fix::Text("run `just install` from the play-by-play repo")
    }
}

use std::process::Command;

use serde::Deserialize;

use crate::error::AppResult;

use super::{Assessment, Check, Facts, Fix, OnPath};

const ID: &str = "play-by-play@play-by-play";

#[derive(Deserialize)]
pub(super) struct Listed {
    id: String,
    pub(super) version: String,
    enabled: bool,
}

pub(super) struct Plugin;

impl Plugin {
    pub(super) fn listed() -> AppResult<Option<Listed>> {
        let listed = Command::new("claude")
            .args(["plugin", "list", "--json"])
            .output()?;
        Ok(serde_json::from_slice::<Vec<Listed>>(&listed.stdout)?
            .into_iter()
            .find(|plugin| plugin.id == ID))
    }
}

pub(super) enum Problem {
    NotInstalled,
    Disabled,
}

impl Check for Plugin {
    const LABEL: &str = "Plugin:";
    type Problem<'a> = Problem;

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>> {
        match &facts.plugin {
            OnPath::Unchecked => None,
            OnPath::Missing => Some(Self::failed("not installed", Problem::NotInstalled)),
            OnPath::Found(plugin) if plugin.enabled => Some(Self::passed("installed and enabled")),
            OnPath::Found(_) => Some(Self::failed("disabled", Problem::Disabled)),
        }
        .into_iter()
    }

    fn suggested_fix<'a>(problem: Problem) -> Fix<'a> {
        match problem {
            Problem::NotInstalled => {
                Fix::Text("run `just install` from the play-by-play repo, then /reload-plugins")
            }
            Problem::Disabled => Fix::Text(
                "run `claude plugin enable play-by-play@play-by-play`, then /reload-plugins",
            ),
        }
    }
}

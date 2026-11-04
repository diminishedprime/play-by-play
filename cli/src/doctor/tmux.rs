use std::process::Command;

use crate::error::AppResult;

use super::{Assessment, Check, Facts, Fix};

pub(super) struct Tmux;

impl Tmux {
    pub(super) fn has_hyperlinks() -> AppResult<bool> {
        let shown = Command::new("tmux")
            .args(["show-options", "-s", "terminal-features"])
            .output()?;
        Ok(String::from_utf8_lossy(&shown.stdout).contains("hyperlinks"))
    }
}

impl Check for Tmux {
    const LABEL: &str = "tmux:";
    type Problem<'a> = ();

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>> {
        facts
            .hyperlinks
            .map(|hyperlinks| {
                if hyperlinks {
                    Self::passed("hyperlinks turned on")
                } else {
                    Self::warned("hyperlinks turned off. Viewer's links won't open", ())
                }
            })
            .into_iter()
    }

    fn suggested_fix<'a>((): ()) -> Fix<'a> {
        Fix::Text("add `set -as terminal-features ',*:hyperlinks'` to your tmux config")
    }
}

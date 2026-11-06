use std::env;
use std::path::Path;

use crate::error::AppResult;

use super::{Assessment, Check, Facts, Fix};

pub(super) enum Missing<'a> {
    NotInstalled,
    OutsidePath(&'a Path),
    Zsh,
    Tmux,
}

#[derive(Clone, Copy, PartialEq)]
enum Need {
    Installed,
    Shell,
    InsideTmux,
    Optional(&'static str),
}

const PROGRAMS: [(&str, Need); 6] = [
    ("play-by-play", Need::Installed),
    ("pbp", Need::Installed),
    ("zsh", Need::Shell),
    ("tmux", Need::InsideTmux),
    ("claude", Need::Optional("the Claude Code plugin")),
    ("code", Need::Optional("the VS Code extension")),
];

pub(super) enum OnPath<T> {
    Unchecked,
    Missing,
    Found(T),
}

impl<T> OnPath<T> {
    pub(super) fn look(
        name: &str,
        listed: impl FnOnce() -> AppResult<Option<T>>,
    ) -> AppResult<OnPath<T>> {
        let looked = Self::present(name).then(listed).transpose()?;
        Ok(looked.map_or(OnPath::Unchecked, |listed| {
            listed.map_or(OnPath::Missing, OnPath::Found)
        }))
    }

    pub(super) fn found(&self) -> Option<&T> {
        match self {
            OnPath::Found(found) => Some(found),
            OnPath::Unchecked | OnPath::Missing => None,
        }
    }

    pub(super) fn present(name: &str) -> bool {
        env::var_os("PATH")
            .is_some_and(|path| env::split_paths(&path).any(|dir| dir.join(name).is_file()))
    }

    fn uninstalled<'a>(facts: &'a Facts, name: &str) -> Missing<'a> {
        facts
            .cargo_bin
            .as_deref()
            .filter(|cargo_bin| cargo_bin.join(name).is_file())
            .map_or(Missing::NotInstalled, Missing::OutsidePath)
    }
}

impl<T> Check for OnPath<T> {
    const LABEL: &str = "$PATH:";
    type Problem<'a> = Missing<'a>;

    fn assessments(facts: &Facts) -> impl Iterator<Item = Assessment<'_>> {
        PROGRAMS
            .into_iter()
            .filter(|(_, need)| *need != Need::InsideTmux || facts.inside_tmux)
            .map(|(name, need)| {
                let assessment = match (Self::present(name), need) {
                    (true, _) => Self::passed(""),
                    (false, Need::Optional(unchecked)) => {
                        Self::skipped(format!("not on PATH, so {unchecked} won't be checked"))
                    }
                    (false, Need::Installed) => {
                        Self::failed("not on PATH", Self::uninstalled(facts, name))
                    }
                    (false, Need::Shell) => Self::failed("not on PATH", Missing::Zsh),
                    (false, Need::InsideTmux) => Self::failed("not on PATH", Missing::Tmux),
                };
                assessment.about(name)
            })
    }

    fn suggested_fix(problem: Self::Problem<'_>) -> Fix<'_> {
        match problem {
            Missing::NotInstalled => Fix::Text("run `just install` from the play-by-play repo"),
            Missing::OutsidePath(cargo_bin) => Fix::AddToPath(cargo_bin),
            Missing::Zsh => Fix::Text("install zsh"),
            Missing::Tmux => Fix::Text("install tmux"),
        }
    }
}

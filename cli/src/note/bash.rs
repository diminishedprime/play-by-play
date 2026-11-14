#![allow(missing_docs)]

use std::borrow::Cow;
use std::fmt::{self, Formatter};
use std::io::Write;

use serde::Deserialize;

use crate::error::AppResult;
use crate::log::Screen;
use crate::note::{Body, Title};

#[derive(Deserialize)]
pub struct BashInput<'a> {
    #[serde(borrow)]
    pub command: Cow<'a, str>,
}

#[derive(Deserialize)]
pub struct BashResult<'a> {
    #[serde(borrow)]
    pub stdout: Cow<'a, str>,
    #[serde(borrow)]
    pub stderr: Cow<'a, str>,
    pub interrupted: bool,
}

impl Title for BashInput<'_> {
    fn title(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "$ {}", self.command)
    }
}

impl Body for BashResult<'_> {
    fn body(&self, out: &mut impl Write, _screen: &Screen) -> AppResult {
        [&self.stdout, &self.stderr]
            .into_iter()
            .filter(|text| !text.is_empty())
            .try_for_each(|text| writeln!(out, "{}", text.trim_end_matches('\n')))?;
        if self.interrupted {
            writeln!(out, "interrupted")?;
        }
        Ok(())
    }
}

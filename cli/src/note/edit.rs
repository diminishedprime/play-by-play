#![allow(missing_docs)]

use std::borrow::Cow;
use std::io::Write;

use bat::{Input, PrettyPrinter};
use serde::Deserialize;

use crate::error::AppResult;
use crate::log::Screen;
use crate::note::patch::{GitDiff, Hunk, Hunks};
use crate::note::{Body, Title};
use crate::text::Text;

#[derive(Deserialize)]
pub struct EditInput<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    #[serde(borrow)]
    pub old_string: Cow<'a, str>,
    #[serde(borrow)]
    pub new_string: Cow<'a, str>,
    pub replace_all: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditResult<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    #[serde(borrow)]
    pub old_string: Cow<'a, str>,
    #[serde(borrow)]
    pub new_string: Cow<'a, str>,
    #[serde(borrow)]
    pub original_file: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub structured_patch: Vec<Hunk<'a>>,
    pub user_modified: bool,
    pub replace_all: bool,
    #[serde(borrow)]
    pub git_diff: Option<GitDiff<'a>>,
    pub staged: Option<bool>,
}

impl Title for EditInput<'_> {
    fn title(&self, out: &mut impl Write) -> AppResult {
        write!(out, "edit: {}", self.file_path)?;
        Ok(())
    }
}

impl Body for EditResult<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        let mut printer = PrettyPrinter::new();
        printer
            .input(Input::from_reader(Hunks::new(&self.structured_patch)))
            .language("Diff")
            .colored_output(true);
        if let Some(theme) = &screen.theme {
            printer.theme(theme);
        }
        printer.print_with_writer(Some(Text(&mut *out)))?;
        Ok(())
    }
}

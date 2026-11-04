#![allow(missing_docs)]

use std::borrow::Cow;
use std::io::Write;

use serde::Deserialize;

use crate::error::AppResult;
use crate::log::Screen;
use crate::note::patch::{GitDiff, Hunk};
use crate::note::source_excerpt::SourceExcerpt;
use crate::note::{Body, Open, Title};

#[derive(Deserialize)]
pub struct WriteInput<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    #[serde(borrow)]
    pub content: Cow<'a, str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult<'a> {
    #[serde(rename = "type", borrow)]
    pub kind: Open<'a, WriteKind>,
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    #[serde(borrow)]
    pub content: Cow<'a, str>,
    #[serde(borrow)]
    pub structured_patch: Vec<Hunk<'a>>,
    #[serde(borrow)]
    pub original_file: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub git_diff: Option<GitDiff<'a>>,
    pub user_modified: Option<bool>,
    pub staged: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WriteKind {
    Create,
    Update,
}

impl Title for WriteInput<'_> {
    fn title(&self, out: &mut impl Write) -> AppResult {
        write!(out, "write: {}", self.file_path)?;
        Ok(())
    }
}

impl<'a> From<&'a WriteResult<'_>> for SourceExcerpt<'a> {
    fn from(written: &'a WriteResult<'_>) -> SourceExcerpt<'a> {
        SourceExcerpt {
            path: &written.file_path,
            content: &written.content,
            start: 1,
        }
    }
}

impl Body for WriteResult<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        SourceExcerpt::from(self).body(out, screen)
    }
}

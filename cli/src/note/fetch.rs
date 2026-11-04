#![allow(missing_docs)]

use std::borrow::Cow;
use std::io::Write;

use serde::Deserialize;

use crate::error::AppResult;
use crate::log::Screen;
use crate::markdown::Markdown;
use crate::note::{Body, Title};

#[derive(Deserialize)]
pub struct FetchInput<'a> {
    #[serde(borrow)]
    pub url: Cow<'a, str>,
    #[serde(borrow)]
    pub prompt: Cow<'a, str>,
    pub offset: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult<'a> {
    pub bytes: u64,
    pub code: u16,
    #[serde(borrow)]
    pub code_text: Cow<'a, str>,
    #[serde(borrow)]
    pub result: Cow<'a, str>,
    pub duration_ms: f64,
    #[serde(borrow)]
    pub url: Cow<'a, str>,
    #[serde(borrow)]
    pub artifact_read: Option<ArtifactRead<'a>>,
}

#[derive(Deserialize)]
pub struct ArtifactRead<'a> {
    #[serde(borrow)]
    pub slug: Cow<'a, str>,
    #[serde(borrow)]
    pub ver: Option<Cow<'a, str>>,
    pub seeded: Option<bool>,
}

impl Title for FetchInput<'_> {
    fn title(&self, out: &mut impl Write) -> AppResult {
        write!(out, "fetch: {}\nprompt: {}", self.url, self.prompt)?;
        Ok(())
    }
}

impl Body for FetchResult<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        Markdown::write(&self.result, &mut *out, screen)
    }
}

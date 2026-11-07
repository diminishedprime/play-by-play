#![allow(missing_docs)]

use std::borrow::Cow;
use std::fmt::{self, Formatter};
use std::io::Write;

use serde::Deserialize;

use crate::error::AppResult;
use crate::log::{Link, Screen};
use crate::markdown::Markdown;
use crate::note::{Body, Open, Title};

#[derive(Deserialize)]
pub struct SearchInput<'a> {
    #[serde(borrow)]
    pub query: Cow<'a, str>,
    #[serde(borrow)]
    pub allowed_domains: Option<Vec<Cow<'a, str>>>,
    #[serde(borrow)]
    pub blocked_domains: Option<Vec<Cow<'a, str>>>,
    #[serde(borrow)]
    pub mode: Open<'a, SearchMode>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Standard,
    Extended,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult<'a> {
    #[serde(borrow)]
    pub query: Cow<'a, str>,
    #[serde(borrow)]
    pub results: Vec<SearchPart<'a>>,
    pub duration_seconds: f64,
    pub search_count: Option<u64>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SearchPart<'a> {
    Hits(#[serde(borrow)] Hits<'a>),
    Summary(#[serde(borrow)] Cow<'a, str>),
}

#[derive(Deserialize)]
pub struct Hits<'a> {
    #[serde(borrow)]
    pub tool_use_id: Cow<'a, str>,
    #[serde(borrow)]
    pub content: Vec<Hit<'a>>,
}

#[derive(Deserialize)]
pub struct Hit<'a> {
    #[serde(borrow)]
    pub title: Cow<'a, str>,
    #[serde(borrow)]
    pub url: Cow<'a, str>,
    #[serde(borrow)]
    pub snippet: Option<Cow<'a, str>>,
}

impl Title for SearchInput<'_> {
    fn title(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "search: {}", self.query)
    }
}

impl Body for SearchResult<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        for part in &self.results {
            match part {
                SearchPart::Hits(hits) => {
                    for hit in &hits.content {
                        let link = Link {
                            url: &hit.url,
                            text: &hit.title,
                        };
                        writeln!(out, "- {link}")?;
                    }
                }
                SearchPart::Summary(summary) => Markdown::write(summary, &mut *out, screen)?,
            }
        }
        Ok(())
    }
}

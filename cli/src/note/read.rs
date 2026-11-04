#![allow(missing_docs)]

use std::borrow::Cow;
use std::io::Write;

use serde::Deserialize;
use serde_json::Value;

use crate::error::AppResult;
use crate::log::Screen;
use crate::note::source_excerpt::SourceExcerpt;
use crate::note::{Body, Open, Title};

#[derive(Deserialize)]
pub struct ReadInput<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    #[serde(borrow)]
    pub pages: Option<Cow<'a, str>>,
    pub allow_large: Option<bool>,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ReadResult<'a> {
    Text {
        #[serde(borrow)]
        file: TextFile<'a>,
        #[serde(borrow)]
        artifact_read: Option<ArtifactRead<'a>>,
    },
    Image {
        #[serde(borrow)]
        file: ImageFile<'a>,
    },
    Notebook {
        #[serde(borrow)]
        file: NotebookFile<'a>,
    },
    Pdf {
        #[serde(borrow)]
        file: PdfFile<'a>,
    },
    Parts {
        #[serde(borrow)]
        file: PartsFile<'a>,
        first_page: Option<u64>,
        total_pages: Option<u64>,
        #[serde(borrow)]
        pages: Option<Vec<PageImage<'a>>>,
    },
    FileUnchanged {
        #[serde(borrow)]
        file: UnchangedFile<'a>,
        #[serde(borrow)]
        source: Option<Open<'a, UnchangedSource>>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFile<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    #[serde(borrow)]
    pub content: Cow<'a, str>,
    pub num_lines: u64,
    pub start_line: u64,
    pub total_lines: u64,
    pub truncated_by_token_cap: Option<bool>,
}

#[derive(Deserialize)]
pub struct ArtifactRead<'a> {
    #[serde(borrow)]
    pub slug: Cow<'a, str>,
    #[serde(borrow)]
    pub ver: Cow<'a, str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageFile<'a> {
    #[serde(borrow)]
    pub base64: Cow<'a, str>,
    #[serde(rename = "type", borrow)]
    pub media_type: Open<'a, MediaType>,
    pub original_size: u64,
    pub dimensions: Option<Dimensions>,
}

#[derive(Deserialize)]
pub enum MediaType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimensions {
    pub original_width: Option<u64>,
    pub original_height: Option<u64>,
    pub display_width: Option<u64>,
    pub display_height: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotebookFile<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    pub cells: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfFile<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    #[serde(borrow)]
    pub base64: Cow<'a, str>,
    pub original_size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsFile<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
    pub original_size: u64,
    pub count: u64,
    #[serde(borrow)]
    pub output_dir: Cow<'a, str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageImage<'a> {
    #[serde(borrow)]
    pub base64: Cow<'a, str>,
    #[serde(borrow)]
    pub media_type: Open<'a, MediaType>,
    #[serde(borrow)]
    pub error: Option<Cow<'a, str>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnchangedFile<'a> {
    #[serde(borrow)]
    pub file_path: Cow<'a, str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnchangedSource {
    Seeded,
}

impl Title for ReadInput<'_> {
    fn title(&self, out: &mut impl Write) -> AppResult {
        write!(out, "read: {}", self.file_path)?;
        Ok(())
    }
}

impl<'a> From<&'a TextFile<'_>> for SourceExcerpt<'a> {
    fn from(file: &'a TextFile<'_>) -> SourceExcerpt<'a> {
        SourceExcerpt {
            path: &file.file_path,
            content: &file.content,
            start: file.start_line,
        }
    }
}

impl Body for TextFile<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        SourceExcerpt::from(self).body(out, screen)
    }
}

impl Body for ReadResult<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        match self {
            ReadResult::Text { file, .. } => file.body(out, screen)?,
            ReadResult::Image { file } => writeln!(out, "an image, {} bytes", file.original_size)?,
            ReadResult::Notebook { file } => {
                writeln!(out, "a notebook, {} cells", file.cells.len())?;
            }
            ReadResult::Pdf { file } => writeln!(out, "a PDF, {} bytes", file.original_size)?,
            ReadResult::Parts { file, .. } => {
                writeln!(
                    out,
                    "a PDF, as {} page images in {}",
                    file.count, file.output_dir
                )?;
            }
            ReadResult::FileUnchanged { .. } => writeln!(out, "unchanged since it was last read")?,
        }
        Ok(())
    }
}

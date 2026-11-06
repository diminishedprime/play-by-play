//! Markdown written for the viewer's terminal, wrapped to its width.

use std::cell::Ref;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::ops::Range;

use bat::PrettyPrinter;
use comrak::nodes::{Ast, AstNode, NodeCodeBlock, NodeValue};
use comrak::{Arena, Options};
use textwrap::core::{self, Fragment};
use textwrap::wrap_algorithms;

use crate::error::AppResult;
use crate::log::{BOLD, Color, RESET, Screen};
use crate::text::Text;

/// Markdown being written into `out`, wrapped at `columns`, its code
/// highlighted in `theme`.
pub struct Markdown<'s, W: Write> {
    out: W,
    columns: f64,
    theme: Option<&'s str>,
}

impl<'s, W: Write> Markdown<'s, W> {
    /// Writes `text` into `out` for `screen`: each paragraph wrapped to its
    /// width, and code highlighted in its theme.
    ///
    /// # Errors
    ///
    /// When `out` can't be written, or a code block can't be highlighted.
    pub fn write(text: &str, out: W, screen: &'s Screen) -> AppResult {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, text, &Options::default());
        let mut markdown = Markdown {
            out,
            columns: f64::from(screen.columns),
            theme: screen.theme.as_deref(),
        };
        for (at, block) in root.children().enumerate() {
            if at > 0 {
                writeln!(markdown.out)?;
            }
            match &block.data().value {
                NodeValue::CodeBlock(code) => markdown.code(code)?,
                NodeValue::List(_) => markdown.list(block)?,
                _ => markdown.paragraph(block, &Piece::PLAIN, &Piece::PLAIN)?,
            }
        }
        Ok(())
    }

    fn list<'a>(&mut self, node: &'a AstNode<'a>) -> AppResult {
        for item in node.children() {
            for block in item.children() {
                self.paragraph(block, &Piece::MARKER, &Piece::HANGING)?;
            }
        }
        Ok(())
    }

    fn code(&mut self, code: &NodeCodeBlock) -> AppResult {
        let mut printer = PrettyPrinter::new();
        printer
            .input_from_bytes(code.literal.as_bytes())
            .colored_output(true);
        if !code.info.is_empty() {
            printer.language(&code.info);
        }
        if let Some(theme) = self.theme {
            printer.theme(theme);
        }
        printer.print_with_writer(Some(Text(&mut self.out)))?;
        Ok(())
    }

    fn paragraph<'a>(&mut self, node: &'a AstNode<'a>, first: &Piece, rest: &Piece) -> AppResult {
        let inlines = Inline::collect(node);
        let paragraph = Paragraph::of(&inlines);
        let width = self.columns - f64::from(u16::try_from(first.text.len()).unwrap_or(u16::MAX));
        let lines = wrap_algorithms::wrap_first_fit(&paragraph.words, &[width]);
        for (at, line) in lines.into_iter().enumerate() {
            write!(self.out, "{}", if at == 0 { first } else { rest })?;
            self.line(&paragraph.pieces, line)?;
        }
        Ok(())
    }

    fn line(&mut self, pieces: &[Piece], words: &[Word]) -> AppResult {
        for (at, word) in words.iter().enumerate() {
            if at > 0 {
                write!(self.out, " ")?;
            }
            for piece in &pieces[word.pieces.clone()] {
                write!(self.out, "{piece}")?;
            }
        }
        writeln!(self.out)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Style {
    Plain,
    Code,
    Bold,
    Heading,
    List,
}

enum Inline<'a> {
    Text(Ref<'a, str>, Style),
    Space,
}

impl<'a> Inline<'a> {
    fn collect(node: &'a AstNode<'a>) -> Vec<Inline<'a>> {
        let in_item = node
            .parent()
            .is_some_and(|parent| matches!(parent.data().value, NodeValue::Item(_)));
        let style = match node.data().value {
            NodeValue::Heading(_) => Style::Heading,
            _ if in_item => Style::List,
            _ => Style::Plain,
        };
        let mut inlines = Vec::new();
        Inline::collect_into(node, style, &mut inlines);
        inlines
    }

    fn collect_into(node: &'a AstNode<'a>, style: Style, inlines: &mut Vec<Inline<'a>>) {
        for child in node.children() {
            let ast = match Ref::filter_map(child.data(), |ast| ast.value.text()) {
                Ok(text) => {
                    inlines.push(Inline::Text(text, style));
                    continue;
                }
                Err(ast) => ast,
            };
            let ast = match Ref::filter_map(ast, Inline::code) {
                Ok(code) => {
                    inlines.push(Inline::Text(code, Style::Code));
                    continue;
                }
                Err(ast) => ast,
            };
            match ast.value {
                NodeValue::SoftBreak | NodeValue::LineBreak => inlines.push(Inline::Space),
                NodeValue::Strong => {
                    drop(ast);
                    Inline::collect_into(child, Style::Bold, inlines);
                }
                _ => {
                    drop(ast);
                    Inline::collect_into(child, style, inlines);
                }
            }
        }
    }

    fn code(ast: &Ast) -> Option<&str> {
        if let NodeValue::Code(code) = &ast.value {
            Some(&code.literal)
        } else {
            None
        }
    }
}

struct Paragraph<'a> {
    pieces: Vec<Piece<'a>>,
    words: Vec<Word>,
    space: bool,
}

impl<'a> Paragraph<'a> {
    fn of(inlines: &'a [Inline<'a>]) -> Paragraph<'a> {
        let mut paragraph = Paragraph {
            pieces: Vec::new(),
            words: Vec::new(),
            space: true,
        };
        for inline in inlines {
            match inline {
                Inline::Text(text, style) => paragraph.push(text, *style),
                Inline::Space => paragraph.space = true,
            }
        }
        paragraph
    }

    fn push(&mut self, text: &'a str, style: Style) {
        self.space |= text.starts_with(char::is_whitespace);
        for piece in text.split_whitespace() {
            let width = core::display_width(piece);
            match self.words.last_mut() {
                Some(word) if !self.space => {
                    word.pieces.end += 1;
                    word.width += width;
                }
                _ => self.words.push(Word {
                    pieces: self.pieces.len()..self.pieces.len() + 1,
                    width,
                }),
            }
            self.pieces.push(Piece { text: piece, style });
            self.space = true;
        }
        self.space = text.ends_with(char::is_whitespace);
    }
}

struct Piece<'a> {
    text: &'a str,
    style: Style,
}

impl Piece<'static> {
    const PLAIN: Piece<'static> = Piece {
        text: "",
        style: Style::Plain,
    };
    const MARKER: Piece<'static> = Piece {
        text: "- ",
        style: Style::List,
    };
    const HANGING: Piece<'static> = Piece {
        text: "  ",
        style: Style::Plain,
    };
}

impl Display for Piece<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.style {
            Style::Plain => write!(f, "{}", self.text),
            Style::Code => write!(f, "{}{}{RESET}", Color::Green, self.text),
            Style::Bold => write!(f, "{BOLD}{}{RESET}", self.text),
            Style::Heading => write!(f, "{BOLD}{}{}{RESET}", Color::Blue, self.text),
            Style::List => write!(f, "{}{}{RESET}", Color::Cyan, self.text),
        }
    }
}

#[derive(Debug)]
struct Word {
    pieces: Range<usize>,
    width: usize,
}

impl Fragment for Word {
    fn width(&self) -> f64 {
        f64::from(u32::try_from(self.width).unwrap_or(u32::MAX))
    }

    fn whitespace_width(&self) -> f64 {
        1.0
    }

    fn penalty_width(&self) -> f64 {
        0.0
    }
}

//! Markdown written for the viewer's terminal, wrapped to its width.

use std::cell::Ref;
use std::fmt::{self, Display, Formatter, Write as _};
use std::io::Write;
use std::ops::Range;

use bat::PrettyPrinter;
use comrak::nodes::{AstNode, NodeCodeBlock, NodeValue};
use comrak::{Arena, Options};
use play_by_play::Painter;
use textwrap::core::{self, Fragment};
use textwrap::wrap_algorithms;

use crate::error::AppResult;
use crate::log::Screen;
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
                _ => markdown.paragraph(Node(block), Node(block))?,
            }
        }
        Ok(())
    }

    fn list<'a>(&mut self, node: &'a AstNode<'a>) -> AppResult {
        let loose = matches!(&node.data().value, NodeValue::List(list) if !list.tight);
        for item in node.children() {
            for (at, block) in item.children().enumerate() {
                if at > 0 && loose {
                    writeln!(self.out)?;
                }
                match &block.data().value {
                    NodeValue::List(_) => self.list(block)?,
                    _ => self.paragraph(Node(item), Node(block))?,
                }
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

    fn paragraph<'a>(&mut self, root: Node<'a>, block: Node<'a>) -> AppResult {
        let pieces = Piece::split(block);
        let words = Word::join(&pieces);
        let lines = wrap_algorithms::wrap_first_fit(&words, &[self.columns - root.indent()]);
        let leads = root
            .0
            .first_child()
            .is_some_and(|first| first.same_node(block.0));
        for (at, line) in lines.into_iter().enumerate() {
            let (Some(first), Some(last)) = (line.first(), line.last()) else {
                continue;
            };
            let line = Line {
                root,
                marked: leads && at == 0,
                pieces: &pieces[first.pieces.start..last.pieces.end],
            };
            writeln!(self.out, "{line}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Node<'a>(&'a AstNode<'a>);

impl<'a> Node<'a> {
    fn text(self) -> Ref<'a, str> {
        Ref::map(self.0.data(), |ast| match &ast.value {
            NodeValue::Text(text) => text.as_ref(),
            NodeValue::Code(code) => code.literal.as_str(),
            NodeValue::SoftBreak | NodeValue::LineBreak => " ",
            _ => "",
        })
    }

    fn is_leaf(self) -> bool {
        matches!(
            self.0.data().value,
            NodeValue::Text(_) | NodeValue::Code(_) | NodeValue::SoftBreak | NodeValue::LineBreak
        )
    }

    fn is_item(self) -> bool {
        matches!(self.0.data().value, NodeValue::Item(_))
    }

    fn is_code(self) -> bool {
        matches!(self.0.data().value, NodeValue::Code(_))
    }

    fn depth(self) -> usize {
        self.0
            .ancestors()
            .map(Node)
            .filter(|node| node.is_item())
            .count()
    }

    fn indent(self) -> f64 {
        f64::from(u32::try_from(2 * self.depth()).unwrap_or(u32::MAX))
    }

    fn toward(self, leaf: Node<'a>) -> Option<Node<'a>> {
        leaf.0
            .ancestors()
            .find(|node| node.parent().is_some_and(|parent| parent.same_node(self.0)))
            .map(Node)
    }

    fn paint(self, out: &mut Painter<'_, impl fmt::Write>, pieces: &[Piece<'a>]) -> fmt::Result {
        pieces
            .chunk_by(|a, b| {
                self.toward(a.node)
                    .is_some_and(|child| Some(child) == self.toward(b.node))
            })
            .enumerate()
            .try_for_each(|(at, run)| {
                if at > 0 && !run[0].glued {
                    out.write_str(" ")?;
                }
                match self.toward(run[0].node) {
                    Some(child) => Style::of(child).wrap(out, |out| child.paint(out, run)),
                    None => out.write_str(&run[0].text()),
                }
            })
    }

    fn gutter(self, out: &mut Painter<'_, impl fmt::Write>, marked: bool) -> fmt::Result {
        let Some(outer) = self.depth().checked_sub(1) else {
            return Ok(());
        };
        write!(out, "{:width$}", "", width = 2 * outer)?;
        out.write_str(if marked { "- " } else { "  " })
    }
}

impl PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.same_node(other.0)
    }
}

enum Style<'a> {
    Plain,
    Bold,
    Italic,
    Code,
    Heading,
    Item,
    Link(Ref<'a, str>),
}

impl<'a> Style<'a> {
    fn of(node: Node<'a>) -> Style<'a> {
        let ast = match Ref::filter_map(node.0.data(), |ast| match &ast.value {
            NodeValue::Link(link) => Some(link.url.as_str()),
            _ => None,
        }) {
            Ok(url) => return Style::Link(url),
            Err(ast) => ast,
        };
        match ast.value {
            NodeValue::Strong => Style::Bold,
            NodeValue::Emph => Style::Italic,
            NodeValue::Code(_) => Style::Code,
            NodeValue::Heading(_) => Style::Heading,
            NodeValue::Item(_) => Style::Item,
            _ => Style::Plain,
        }
    }

    fn wrap<'p, W: fmt::Write>(
        &self,
        out: &mut Painter<'p, W>,
        inner: impl FnOnce(&mut Painter<'p, W>) -> fmt::Result,
    ) -> fmt::Result {
        match self {
            Style::Plain => inner(out),
            Style::Bold => out.bold(inner),
            Style::Italic => out.italic(inner),
            Style::Code => out.green(inner),
            Style::Heading => out.bold(|out| out.blue(inner)),
            Style::Item => out.cyan(inner),
            Style::Link(url) => out.link(url, |out| out.blue(inner)),
        }
    }
}

#[derive(Clone)]
struct Piece<'a> {
    node: Node<'a>,
    range: Range<usize>,
    glued: bool,
}

impl<'a> Piece<'a> {
    fn split(block: Node<'a>) -> Vec<Piece<'a>> {
        let (pieces, _) = block
            .0
            .descendants()
            .map(Node)
            .filter(|node| node.is_leaf())
            .fold((Vec::new(), true), |(mut pieces, gap), node| {
                let text = node.text();
                let glued = !gap && !text.starts_with(char::is_whitespace);
                let breaks = !node.is_code();
                let words = text
                    .split(|c: char| breaks && c.is_whitespace())
                    .filter(|word| !word.is_empty());
                pieces.extend(words.enumerate().map(|(at, word)| {
                    let start = word.as_ptr().addr() - text.as_ptr().addr();
                    Piece {
                        node,
                        range: start..start + word.len(),
                        glued: glued && at == 0,
                    }
                }));
                let gap = text.ends_with(char::is_whitespace) || (text.is_empty() && gap);
                (pieces, gap)
            });
        pieces
    }

    fn text(&self) -> Ref<'a, str> {
        Ref::map(self.node.text(), |text| &text[self.range.clone()])
    }
}

#[derive(Debug)]
struct Word {
    pieces: Range<usize>,
    width: usize,
}

impl Word {
    fn join(pieces: &[Piece<'_>]) -> Vec<Word> {
        pieces
            .chunk_by(|_, next| next.glued)
            .scan(0, |start, run| {
                let pieces = *start..*start + run.len();
                *start = pieces.end;
                Some(Word {
                    pieces,
                    width: run
                        .iter()
                        .map(|piece| core::display_width(&piece.text()))
                        .sum(),
                })
            })
            .collect()
    }
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

struct Line<'l, 'a> {
    root: Node<'a>,
    marked: bool,
    pieces: &'l [Piece<'a>],
}

impl Display for Line<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut out = Painter::from(f);
        Style::of(self.root).wrap(&mut out, |out| {
            self.root.gutter(out, self.marked)?;
            self.root.paint(out, self.pieces)
        })
    }
}

use std::fmt;
use std::hash::Hash;

use indexmap::IndexMap;

use crate::board::Live;

use PingPong::{Ping, Pong};

/// The keyed lines a [`Broadcast`](crate::Broadcast) shows in drawing order.
///
/// Its `turn` just alternates every render. Each render starts by flipping it,
/// so every line from the render before already counts as unseen without being
/// reset, and the render only has to mark the lines it keeps.
pub(crate) struct Screen<K> {
    lines: IndexMap<K, Shown>,
    turn: PingPong,
}

/// One render in progress.
///
/// A later line with the same key overwrites an earlier one. Dropping the frame
/// cleans up, removing every line the program didn't yield this render.
struct Frame<'a, K> {
    screen: &'a mut Screen<K>,
    next: usize,
}

struct Shown {
    text: String,
    seen: PingPong,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PingPong {
    Ping,
    Pong,
}

impl PingPong {
    fn flip(self) -> PingPong {
        match self {
            Ping => Pong,
            Pong => Ping,
        }
    }
}

impl<K: Eq + Hash + Clone> Screen<K> {
    pub(crate) fn new() -> Screen<K> {
        Screen {
            lines: IndexMap::new(),
            turn: Ping,
        }
    }

    pub(crate) fn refresh(
        &mut self,
        lines: impl Iterator<Item = impl Live<Key = K>>,
        columns: u16,
    ) {
        let mut frame = self.frame();
        for line in lines {
            let drawn = frame.line(line.key());
            line.show(columns, &mut drawn.into())
                .expect("a line shows into a String");
        }
    }

    fn frame(&mut self) -> Frame<'_, K> {
        self.turn = self.turn.flip();
        Frame {
            screen: self,
            next: 0,
        }
    }

    pub(crate) fn block(&self) -> impl Iterator<Item = &str> {
        self.lines.values().map(|shown| shown.text.as_str())
    }
}

impl<K: Eq + Hash + Clone> Frame<'_, K> {
    fn line(&mut self, key: &K) -> &mut impl fmt::Write {
        let turn = self.screen.turn;
        let lines = &mut self.screen.lines;
        let (at, first) = match lines.get_index_of(key) {
            Some(at) => (at, lines[at].seen != turn),
            None => (lines.insert_full(key.clone(), Shown::new(turn)).0, true),
        };
        let at = if first {
            lines.move_index(at, self.next);
            self.next += 1;
            self.next - 1
        } else {
            at
        };
        let shown = &mut lines[at];
        shown.prepare(turn);
        &mut shown.text
    }
}

impl<K> Drop for Frame<'_, K> {
    fn drop(&mut self) {
        let turn = self.screen.turn;
        self.screen.lines.retain(|_, shown| shown.seen == turn);
    }
}

impl Shown {
    fn new(turn: PingPong) -> Shown {
        Shown {
            text: String::new(),
            seen: turn,
        }
    }

    fn prepare(&mut self, turn: PingPong) {
        self.seen = turn;
        self.text.clear();
    }
}

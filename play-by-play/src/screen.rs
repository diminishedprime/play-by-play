use std::fmt;
use std::hash::Hash;
use std::mem;

use indexmap::IndexMap;
use indexmap::map::RawEntryApiV1;

use PingPong::{Ping, Pong};

/// The keyed lines a [`LiveView`](crate::LiveView) shows in drawing order.
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
/// cleans up, removing every line the program didn't yield this render, and
/// settles which narrations are new.
pub(crate) struct Frame<'a, K> {
    screen: &'a mut Screen<K>,
}

struct Shown {
    text: String,
    narration: String,
    narrated: Option<String>,
    fresh: bool,
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

    pub(crate) fn frame(&mut self) -> Frame<'_, K> {
        self.turn = self.turn.flip();
        Frame { screen: self }
    }

    pub(crate) fn narrations(&self) -> impl Iterator<Item = &str> {
        self.lines
            .values()
            .filter(|shown| shown.fresh)
            .filter_map(|shown| shown.narrated.as_deref())
    }

    pub(crate) fn block(&self) -> impl Iterator<Item = &str> {
        self.lines.values().map(|shown| shown.text.as_str())
    }
}

impl<K: Eq + Hash + Clone> Frame<'_, K> {
    pub(crate) fn line(&mut self, key: &K) -> (&mut impl fmt::Write, &mut impl fmt::Write) {
        let turn = self.screen.turn;
        let (_, shown) = self
            .screen
            .lines
            .raw_entry_mut_v1()
            .from_key(key)
            .or_insert_with(|| (key.clone(), Shown::new(turn)));
        shown.prepare(turn);
        (&mut shown.text, &mut shown.narration)
    }
}

impl<K> Drop for Frame<'_, K> {
    fn drop(&mut self) {
        let turn = self.screen.turn;
        self.screen
            .lines
            .retain(|_, shown| (shown.seen == turn).then(|| shown.settle()).is_some());
    }
}

impl Shown {
    fn new(turn: PingPong) -> Shown {
        Shown {
            text: String::new(),
            narration: String::new(),
            narrated: None,
            fresh: false,
            seen: turn,
        }
    }

    fn prepare(&mut self, turn: PingPong) {
        self.seen = turn;
        self.text.clear();
        self.narration.clear();
    }

    fn settle(&mut self) {
        self.fresh = !self.narration.is_empty() && self.narrated.as_ref() != Some(&self.narration);
        if self.fresh {
            mem::swap(
                self.narrated.get_or_insert_with(String::new),
                &mut self.narration,
            );
        }
    }
}

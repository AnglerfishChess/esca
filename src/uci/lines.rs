//! The lines an engine has written that the client has not read yet.
//!
//! The queue is capped, so that an engine writing faster than it is read
//! cannot grow it without bound. Room is made by dropping the oldest line that
//! carries no part of the conversation, and every line dropped is counted.

use std::collections::VecDeque;

use super::protocol::{Message, parse};

/// How many unread lines are kept.
pub(super) const CAPACITY: usize = 4096;

/// Whether a line may be dropped to make room: a search report, or a line the
/// protocol reads as nothing. `bestmove`, `readyok`, `uciok`, `id`, `option`,
/// `copyprotection` and `registration` are kept whatever the queue costs.
fn droppable(line: &str) -> bool {
    matches!(parse(line), Message::Info(_) | Message::Raw(_))
}

/// Unread lines, oldest first.
pub(super) struct Queue {
    lines: VecDeque<String>,
    dropped: u64,
    closed: bool,
}

impl Queue {
    /// An empty queue of an engine that is still writing.
    pub(super) fn new() -> Queue {
        Queue {
            lines: VecDeque::new(),
            dropped: 0,
            closed: false,
        }
    }

    /// Adds one line, making room for it first when the queue is full.
    pub(super) fn push(&mut self, line: String) {
        if self.lines.len() >= CAPACITY {
            if let Some(at) = self.lines.iter().position(|line| droppable(line)) {
                self.lines.remove(at);
                self.dropped += 1;
            }
        }
        self.lines.push_back(line);
    }

    /// The oldest line, taken off the queue.
    pub(super) fn pop(&mut self) -> Option<String> {
        self.lines.pop_front()
    }

    /// Whether there is nothing to read.
    pub(super) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Records that the engine wrote its last line.
    pub(super) fn close(&mut self) {
        self.closed = true;
    }

    /// Whether the engine's output is over and the queue is drained.
    pub(super) fn is_done(&self) -> bool {
        self.closed && self.lines.is_empty()
    }

    /// How many lines were dropped to keep the queue within its cap.
    pub(super) fn dropped(&self) -> u64 {
        self.dropped
    }
}

use std::collections::{BTreeSet, VecDeque};

use crate::atom::AtomId;

#[derive(Debug, Default)]
pub struct AtomicScheduler {
    pending: VecDeque<AtomId>,
    queued: BTreeSet<AtomId>,
}

impl AtomicScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Duplicate invalidations collapse into one unit of work.
    pub fn schedule(&mut self, atom: AtomId) {
        if self.queued.insert(atom) {
            self.pending.push_back(atom);
        }
    }

    pub fn pop(&mut self) -> Option<AtomId> {
        let atom = self.pending.pop_front()?;
        self.queued.remove(&atom);
        Some(atom)
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }
}

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::atom::AtomId;
use crate::error::{AtomicError, AtomicResult};

#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// source -> direct dependents
    dependents: HashMap<AtomId, BTreeSet<AtomId>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dependency(&mut self, source: AtomId, dependent: AtomId) {
        self.dependents
            .entry(source)
            .or_default()
            .insert(dependent);
    }

    pub fn direct_dependents(&self, source: AtomId) -> impl Iterator<Item = AtomId> + '_ {
        self.dependents
            .get(&source)
            .into_iter()
            .flat_map(|items| items.iter().copied())
    }

    /// Returns only atoms affected by `source`.
    pub fn affected(&self, source: AtomId) -> AtomicResult<Vec<AtomId>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        queue.push_back(source);
        visited.insert(source);

        while let Some(current) = queue.pop_front() {
            for next in self.direct_dependents(current) {
                if !visited.insert(next) {
                    if next == source {
                        return Err(AtomicError::DependencyCycle);
                    }
                    continue;
                }
                result.push(next);
                queue.push_back(next);
            }
        }

        Ok(result)
    }
}

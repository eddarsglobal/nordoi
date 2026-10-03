use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AtomId(pub u64);

#[derive(Debug, Clone)]
pub struct Atom {
    pub id: AtomId,
    pub value: Value,
    pub version: u64,
    pub dirty: bool,
}

impl Atom {
    pub fn new(id: AtomId, value: Value) -> Self {
        Self {
            id,
            value,
            version: 0,
            dirty: false,
        }
    }

    /// Atomic rule: assigning an identical value performs zero state work.
    pub fn set(&mut self, next: Value) -> bool {
        if self.value == next {
            return false;
        }

        self.value = next;
        self.version = self.version.wrapping_add(1);
        self.dirty = true;
        true
    }

    pub fn clean(&mut self) {
        self.dirty = false;
    }
}

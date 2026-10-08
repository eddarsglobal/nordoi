use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    Network(String),
    FileRead(String),
    FileWrite(String),
    Camera,
    Microphone,
    Location,
    Gpu,
    Xr,
    Process,
    ConsoleWrite,
}

#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    allowed: HashSet<Capability>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow(&mut self, capability: Capability) {
        self.allowed.insert(capability);
    }

    pub fn revoke(&mut self, capability: &Capability) {
        self.allowed.remove(capability);
    }

    pub fn contains(&self, capability: &Capability) -> bool {
        self.allowed.contains(capability)
    }

    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty()
    }
}

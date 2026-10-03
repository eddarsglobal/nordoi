use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Effect {
    Pure,
    StateRead,
    StateWrite,

    Network(String),
    FileRead(String),
    FileWrite(String),

    Camera,
    Microphone,
    Location,
    Gpu,
    Xr,
    Process,
}

#[derive(Debug, Clone, Default)]
pub struct EffectSet {
    declared: BTreeSet<Effect>,
}

impl EffectSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pure() -> Self {
        let mut set = Self::new();
        set.declare(Effect::Pure);
        set
    }

    pub fn declare(&mut self, effect: Effect) {
        self.declared.insert(effect);
    }

    pub fn contains(&self, effect: &Effect) -> bool {
        self.declared.contains(effect)
    }

    pub fn is_pure(&self) -> bool {
        self.declared.len() == 1 && self.declared.contains(&Effect::Pure)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.declared.iter()
    }
}

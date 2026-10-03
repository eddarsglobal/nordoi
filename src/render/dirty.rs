use std::ops::{BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DirtyMask(u8);

impl DirtyMask {
    pub const NONE: Self = Self(0);
    pub const TRANSFORM: Self = Self(1 << 0);
    pub const APPEARANCE: Self = Self(1 << 1);
    pub const CONTENT: Self = Self(1 << 2);
    pub const VISIBILITY: Self = Self(1 << 3);
    pub const STRUCTURE: Self = Self(1 << 4);
    pub const ALL: Self = Self(
        Self::TRANSFORM.0
            | Self::APPEARANCE.0
            | Self::CONTENT.0
            | Self::VISIBILITY.0
            | Self::STRUCTURE.0,
    );

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::ALL.0 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }
}

impl BitOr for DirtyMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitOrAssign for DirtyMask {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

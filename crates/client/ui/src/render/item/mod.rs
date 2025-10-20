#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have new hands.
pub enum ItemDisplayHand {
    Left,
    Right,
}

impl ItemDisplayHand {
    /// Whether this hand is left.
    pub const fn is_left(&self) -> bool {
        matches!(self, Self::Left)
    }

    /// Whether this hand is right.
    pub const fn is_right(&self) -> bool {
        matches!(self, Self::Right)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have new perspectives.
pub enum ItemDisplayPerspective {
    ThirdPerson,
    FirstPerson,
}

impl ItemDisplayPerspective {
    /// Whether this perspective is first-person.
    pub const fn is_first_person(&self) -> bool {
        matches!(self, Self::FirstPerson)
    }

    /// Whether this perspective is third-person.
    pub const fn is_third_person(&self) -> bool {
        matches!(self, Self::ThirdPerson)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ItemDisplayMode {
    Perspective(ItemDisplayPerspective, ItemDisplayHand),
    Head,
    Gui,
    Ground,
    Fixed,
}

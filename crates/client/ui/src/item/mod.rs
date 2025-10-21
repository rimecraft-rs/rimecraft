//! Utilities for displaying items as UI elements.

/// The hand in which an item is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have new hands.
pub enum ItemDisplayHand {
    /// The left hand.
    Left,
    /// The right hand.
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

/// The perspective in which an item is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)] // We won't ever have new perspectives.
pub enum ItemDisplayPerspective {
    /// The first-person perspective.
    FirstPerson,
    /// The third-person perspective.
    ThirdPerson,
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

/// The mode in which an item is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ItemDisplayMode {
    /// The item is displayed in a specific perspective and hand.
    Perspective(ItemDisplayPerspective, ItemDisplayHand),
    /// The item is displayed on the player's head.
    Head,
    /// The item is displayed in the GUI.
    Gui,
    /// The item is displayed on the ground.
    Ground,
    /// The item is displayed as a fixed element.
    Fixed,
}

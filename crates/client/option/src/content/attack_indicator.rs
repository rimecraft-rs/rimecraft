//! Enum for attack indicator.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::Localize;

use super::ByUSizeId;

/// Represents the position of the attack indicator.
#[derive(Debug, Sequence, Localize)]
#[localize(prefix = [option, _])]
#[non_exhaustive]
pub enum AttackIndicator {
    /// Attack indicator off.
    Off,
    /// Below crosshair.
    Crosshair,
    /// Next to hotbar.
    Hotbar,
}

impl ByUSizeId for AttackIndicator {}

impl Display for AttackIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AttackIndicator::Off => "off",
                AttackIndicator::Crosshair => "crosshair",
                AttackIndicator::Hotbar => "hotbar",
            }
        )
    }
}

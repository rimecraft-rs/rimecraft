//! Enum for attack indicator.

use std::{borrow::Cow, fmt::Display};

use enum_iterator::Sequence;
use rimecraft_text::{Localize, format_localization_key};

use super::ByUSizeId;

/// Represents the position of the attack indicator.
#[derive(Debug, Sequence)]
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

impl Localize for AttackIndicator {
    fn localization_key(&self) -> Cow<'_, str> {
        Cow::Owned(format_localization_key![
            "options",
            match self {
                AttackIndicator::Off => "off".into(),
                _ => format_localization_key!["attack", format!("{}", self)],
            }
        ])
    }
}

//! Enum for narrator mode.

use std::{borrow::Cow, fmt::Display};

use enum_iterator::Sequence;
use rimecraft_text::{Localize, format_localization_key};

use super::ByUSizeId;

/// Represents the mode of narrator.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.NarratorMode` (yarn).
#[derive(Debug, Sequence)]
#[non_exhaustive]
pub enum NarratorMode {
    /// Narrator off.
    Off,
    /// Narrates all.
    All,
    /// Narrates only chat messages.
    Chat,
    /// Narrates only system messages.
    System,
}

impl ByUSizeId for NarratorMode {}

impl Display for NarratorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NarratorMode::Off => "off",
                NarratorMode::All => "all",
                NarratorMode::Chat => "chat",
                NarratorMode::System => "system",
            }
        )
    }
}

impl NarratorMode {
    /// Returns whether **chat messages** should be narrated.
    pub fn should_narrate_chat(&self) -> bool {
        matches!(self, NarratorMode::All | NarratorMode::Chat)
    }

    /// Returns whether **system messages** should be narrated.
    pub fn should_narrate_system(&self) -> bool {
        matches!(self, NarratorMode::All | NarratorMode::System)
    }

    /// Returns whether **any messages** should be narrated.
    pub fn should_narrate(&self) -> bool {
        !matches!(self, NarratorMode::Off)
    }
}

impl Localize for NarratorMode {
    fn localization_key(&self) -> Cow<'_, str> {
        Cow::Owned(format_localization_key!(
            "options",
            "narrator",
            &format!("{}", self)
        ))
    }
}

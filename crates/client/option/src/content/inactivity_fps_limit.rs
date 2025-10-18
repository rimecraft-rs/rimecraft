//! Inactivity FPS limit choices.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::Localize;

use super::ByUSizeId;

/// Represents FPS limit behavior when inactive or minimized.
#[derive(Debug, Sequence, Localize)]
#[localize(prefix = [option, _])]
#[non_exhaustive]
pub enum InactivityFpsLimit {
    /// When window is minimized.
    Minimized,
    /// When player is away-from-keyboard.
    Afk,
}

impl ByUSizeId for InactivityFpsLimit {}

impl Display for InactivityFpsLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InactivityFpsLimit::Minimized => "minimized",
                InactivityFpsLimit::Afk => "afk",
            }
        )
    }
}

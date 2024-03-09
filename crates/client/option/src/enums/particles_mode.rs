//! Enum for particles mode.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::{format_localization_key, Localizable};

use super::ByUSizeId;

/// Represents the rendering mode of particles.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.ParticlesMode` (yarn).
#[derive(Debug, Sequence, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum ParticlesMode {
    /// Renders all particles.
    All,
    /// Renders decreased particles.
    Decreased,
    /// Renders as less particles as possible.
    Minimal,
}

impl ByUSizeId for ParticlesMode {}

impl Display for ParticlesMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ParticlesMode::All => "all",
                ParticlesMode::Decreased => "decreased",
                ParticlesMode::Minimal => "minimal",
            }
        )
    }
}

impl Localizable for ParticlesMode {
    fn localization_key(&self) -> String {
        format_localization_key!("options", "particles", &format!("{}", self))
    }
}

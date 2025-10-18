//! Enum for graphics mode.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::Localize;

use super::ByUSizeId;

/// Represents the mode for graphics.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.GraphicsMode` (yarn).
#[derive(Debug, Sequence, Localize)]
#[localize(prefix = [option, graphics])]
#[non_exhaustive]
pub enum GraphicsMode {
    /// The fastest rendering speed with the worst picture.
    Fast,
    /// Not that fast but with a better picture.
    Fancy,
    /// Maybe slow, but with the best picture.
    Fabulous,
}

impl ByUSizeId for GraphicsMode {}

impl Display for GraphicsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GraphicsMode::Fast => "fast",
                GraphicsMode::Fancy => "fancy",
                GraphicsMode::Fabulous => "fabulous",
            }
        )
    }
}

//! Enum for cloud render mode.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::Localize;

use super::ByUSizeId;

/// Represents the rendering mode of clouds.
#[derive(Debug, Sequence, Localize)]
#[localize(prefix = [option, clouds])]
#[non_exhaustive]
pub enum CloudRenderMode {
    /// Doesn't render clouds.
    Off,
    /// Render clouds faster.
    Fast,
    /// Render clouds fancier.
    Fancy,
}

impl ByUSizeId for CloudRenderMode {}

impl Display for CloudRenderMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CloudRenderMode::Off => "off",
                CloudRenderMode::Fast => "fast",
                CloudRenderMode::Fancy => "fancy",
            }
        )
    }
}

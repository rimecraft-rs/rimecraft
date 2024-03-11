//! Enum for cloud render mode.

use std::fmt::Display;

use enum_iterator::Sequence;

use super::ByUSizeId;

/// Represents the rendering mode of clouds.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.CloudRenderMode` (yarn).
#[derive(Debug, Sequence)]
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

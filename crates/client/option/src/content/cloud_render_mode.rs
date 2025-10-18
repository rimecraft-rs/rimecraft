//! Enum for cloud render mode.

use std::{borrow::Cow, fmt::Display};

use enum_iterator::Sequence;
use rimecraft_text::{Localize, format_localization_key};

use super::ByUSizeId;

/// Represents the rendering mode of clouds.
#[derive(Debug, Sequence)]
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

impl Localize for CloudRenderMode {
    fn localization_key(&self) -> Cow<'_, str> {
        Cow::Owned(format_localization_key![
            "options",
            match self {
                CloudRenderMode::Off => "off".into(),
                _ => format_localization_key!["clouds", format!("{}", self)],
            }
        ])
    }
}

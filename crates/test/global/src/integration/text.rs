//! `rimecraft-text` integrations.

#![cfg(feature = "text")]

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use text::ProvideTextTy;

use crate::TestContext;

impl ProvideTextTy for TestContext {
    type Content = TextContent;

    type StyleExt = ();
}

/// Vanilla-style text content types for testing. Incomplete.
///
/// *Distinguishing optional type tags is unsupported as it only accelerates parsing,
/// which is not favorable in testing contexts.*
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextContent {
    /// Displays plain text.
    #[serde(rename = "text")]
    Plain {
        /// A string containing plain text to display directly.
        text: String,
    },
    /// Displays a translated piece of text from the currently selected language.
    #[serde(rename = "translatable")]
    Translated {
        /// A translation identifier, corresponding to the identifiers found in loaded language files.
        translate: String,
        /// If no corresponding translation can be found, this is used as the translated text.
        fallback: Option<String>,
    },
}

impl From<String> for TextContent {
    fn from(value: String) -> Self {
        Self::Plain { text: value }
    }
}

impl From<&str> for TextContent {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl Display for TextContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextContent::Plain { text } => write!(f, "{text}"),
            TextContent::Translated {
                translate,
                fallback,
            } => write!(f, "{}", fallback.as_ref().unwrap_or(translate)),
        }
    }
}

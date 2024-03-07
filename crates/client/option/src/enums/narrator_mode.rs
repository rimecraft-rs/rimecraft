//! Enum for narrator mode.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::{format_localization_key, Localizable};

use super::ByUSizeId;

/// Represents the mode of narrator.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.NarratorMode` (yarn).
#[derive(Debug, Sequence)]
pub enum NarratorMode {
	/// Narrator off.
	Off,
	/// Narrates all.
	All,
	/// Narrates only chat messages.
	Chat,
	/// Narrates only system messages.
	System
}

impl ByUSizeId for NarratorMode {}

impl Display for NarratorMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			NarratorMode::Off => "off",
			NarratorMode::All => "all",
			NarratorMode::Chat => "chat",
			NarratorMode::System => "system",
		})
	}
}

impl NarratorMode {
	pub fn should_narrate_chat(&self) -> bool {
		match self {
			NarratorMode::All | NarratorMode::Chat => true,
			_ => false
		}
	}

	pub fn should_narrate_system(&self) -> bool {
		match self {
			NarratorMode::All | NarratorMode::System => true,
			_ => false
		}
	}
}

impl Localizable for NarratorMode {
	fn localization_key(&self) -> String {
		format_localization_key!("options", "narrator", format!("{}", self))
	}
}
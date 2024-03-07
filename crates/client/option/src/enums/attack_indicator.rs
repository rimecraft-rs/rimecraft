//! Enum for attack indicator.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_identifier::format_localization_key;

use super::ByUSizeId;

/// Represents the position of the attack indicator.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.AttackIndicator` (yarn).
#[derive(Debug, Sequence)]
pub enum AttackIndicator {
	/// Attack indicator off.
	Off,
	/// Below crosshair.
	Crosshair,
	/// Next to hotbar.
	Hotbar
}

impl ByUSizeId for AttackIndicator {}

impl Display for AttackIndicator {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			AttackIndicator::Off => "off",
			AttackIndicator::Crosshair => "crosshair",
			AttackIndicator::Hotbar => "hotbar",
		})
	}
}

impl AttackIndicator {
	pub fn localization_key(&self) -> String {
		format_localization_key!("options", "attack", format!("{}", self))
	}
}
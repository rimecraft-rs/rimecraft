//! Enum for perspective.

use enum_iterator::Sequence;

use super::ByIntId;

/// Represents the perspective.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.Perspective` (yarn).
#[derive(Debug, Sequence)]
pub enum Perspective {
	/// 1st person perspective.
	FirstPerson,
	/// 3rd person perspective, camera behind player.
	ThirdPersonBack,
	/// 3rd person perspective, camera in front of player.
	ThirdPersonFront
}

impl ByIntId for Perspective {}

impl Perspective {
	fn is_first_person(&self) -> bool {
		match self {
			Perspective::FirstPerson => true,
			_ => false
		}
	}

	fn is_front_view(&self) -> bool {
		match self {
			Perspective::ThirdPersonBack => false,
			_ => true
		}
	}
}
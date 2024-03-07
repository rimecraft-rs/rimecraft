//! Enum for perspective.

use enum_iterator::Sequence;

use super::ByUIntId;

/// Represents the perspective.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.Perspective` (yarn).
#[derive(Debug, Sequence, PartialEq)]
pub enum Perspective {
	/// 1st person perspective.
	FirstPerson,
	/// 3rd person perspective, camera behind player.
	ThirdPersonBack,
	/// 3rd person perspective, camera in front of player.
	ThirdPersonFront
}

impl ByUIntId for Perspective {}

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
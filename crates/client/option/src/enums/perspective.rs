//! Enum for perspective.

use std::fmt::Display;

use enum_iterator::Sequence;

use super::ByUSizeId;

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

impl ByUSizeId for Perspective {}

impl Display for Perspective {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Perspective::FirstPerson => "first_person",
			Perspective::ThirdPersonBack => "third_person_back",
			Perspective::ThirdPersonFront => "third_person_front",
		})
	}
}

impl Perspective {
	pub fn is_first_person(&self) -> bool {
		match self {
			Perspective::FirstPerson => true,
			_ => false
		}
	}

	pub fn is_front_view(&self) -> bool {
		match self {
			Perspective::ThirdPersonBack => false,
			_ => true
		}
	}
}
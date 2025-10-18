//! Enum for perspective.

use std::fmt::Display;

use enum_iterator::Sequence;
use rimecraft_text::Localize;

use super::ByUSizeId;

/// Represents the perspective.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.Perspective` (yarn).
#[derive(Debug, Sequence, PartialEq, Localize)]
#[localize(prefix = [option, _])]
#[non_exhaustive]
pub enum Perspective {
    /// 1st person perspective.
    FirstPerson,
    /// 3rd person perspective, camera behind player.
    ThirdPersonBack,
    /// 3rd person perspective, camera in front of player.
    ThirdPersonFront,
}

impl ByUSizeId for Perspective {}

impl Display for Perspective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Perspective::FirstPerson => "first_person",
                Perspective::ThirdPersonBack => "third_person_back",
                Perspective::ThirdPersonFront => "third_person_front",
            }
        )
    }
}

impl Perspective {
    /// Returns whether the perspective is first person.
    pub fn is_first_person(&self) -> bool {
        matches!(self, Perspective::FirstPerson)
    }

    /// Returns whether the perspective is front view (first person or third person front).
    pub fn is_front_view(&self) -> bool {
        !matches!(self, Perspective::ThirdPersonBack)
    }
}

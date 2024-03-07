//! Enum for particles mode.

use enum_iterator::Sequence;

use super::ByUIntId;

/// Represents the rendering mode of particles.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.ParticlesMode` (yarn).
#[derive(Debug, Sequence, PartialEq)]
pub enum ParticlesMode {
	/// Renders all particles.
	All,
	/// Renders decreased particles.
	Decreased,
	/// Renders as less particles as possible.
	Minimal
}

impl ByUIntId for ParticlesMode {}

impl ParticlesMode {
	fn translation_key(&self) -> String {
		String::from("options.particles.") + match self {
			ParticlesMode::All => "all",
			ParticlesMode::Decreased => "decreased",
			ParticlesMode::Minimal => "minimal",
		}
	}
}
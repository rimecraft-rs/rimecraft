//! Enum for particles mode.

/// Represents the rendering mode of particles.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.ParticlesMode` (yarn).
#[derive(Debug)]
pub enum ParticlesMode {
	/// Renders all particles.
	All,
	/// Renders decreased particles.
	Decreased,
	/// Renders as less particles as possible.
	Minimal
}
//! Enum for narrator mode.

/// Represents the mode of narrator.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.NarratorMode` (yarn).
#[derive(Debug)]
pub enum NarratorMode {
	/// Narrator off.
	Off,
	/// Narrates all.
	All,
	/// Narrates only chat messages.
	Chat,
	/// Narrates only system messages.
	Sysyem
}
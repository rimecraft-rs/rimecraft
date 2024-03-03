//! Enum for graphics mode.

/// Represents the mode for graphics.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.GraphicsMode` (yarn).
#[derive(Debug)]
pub enum GraphicsMode {
	/// The fastest rendering speed with the worst picture.
	Fast,
	/// Not that fast but with a better picture.
	Fancy,
	/// Maybe slow, but with the best picture.
	Fabulous
}
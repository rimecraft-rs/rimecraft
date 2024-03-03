//! Enum for cloud render mode.

/// Represents the rendering mode of clouds.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.CloudRenderMode` (yarn).
#[derive(Debug)]
pub enum CloudRenderMode {
	/// Doesn't render clouds.
	Off,
	/// Render clouds faster.
	Fast,
	/// Render clouds fancier.
	Fancy
}
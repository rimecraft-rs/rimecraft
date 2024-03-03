//! Enum for attack indicator.

/// Represents the position of the attack indicator.
///
/// # MCJE Reference
///
/// This type represents `net.minecraft.client.option.AttackIndicator` (yarn).
#[derive(Debug)]
pub enum AttackIndicator {
	/// No attack indicator.
	None,
	/// Below crosshair.
	Crosshair,
	/// Next to hotbar.
	Hotbar
}
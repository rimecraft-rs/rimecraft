//! Minecraft client inputs.

use glam::Vec2;
use slow_down::SlowDownTickable;

pub mod controller_input;
pub mod cursor_movement;
pub mod slow_down;

/// An input state.
pub trait Input: SlowDownTickable {
    /// The forward movement.
    fn movement_forward(&self) -> f32;
    /// The sideways movement.
    fn movement_sideways(&self) -> f32;
    /// Whether the player is jumping.
    fn is_jumping(&self) -> bool;
    /// Whether the player is sneaking.
    fn is_sneaking(&self) -> bool;

    /// The movement input as a vector.
    fn movement_input(&self) -> Vec2 {
        Vec2 {
            x: self.movement_sideways(),
            y: self.movement_forward(),
        }
    }

    /// Whether the given value is valid for a movement.
    fn has_movement(value: f32) -> bool {
        value > 1.0e-5
    }

    /// Whether the player has a valid forward movement.
    fn has_movement_forward(&self) -> bool {
        Self::has_movement(self.movement_forward())
    }
}

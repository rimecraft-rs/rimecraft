//! Minecraft client inputs.

use glam::Vec2;
use slow_down::SlowDownTickable;

pub mod cursor_movement;
pub mod keyboard_input;
pub mod slow_down;

/// Represents an input state.
pub trait Input: SlowDownTickable {
    /// Returns the forward movement.
    fn movement_forward(&self) -> f32;
    /// Returns the sideways movement.
    fn movement_sideways(&self) -> f32;
    /// Returns whether the player is jumping.
    fn jumping(&self) -> bool;
    /// Returns whether the player is sneaking.
    fn sneaking(&self) -> bool;

    /// Returns the movement input as a vector.
    fn get_movement_input(&self) -> Vec2 {
        Vec2 {
            x: self.movement_sideways(),
            y: self.movement_forward(),
        }
    }

    /// Returns whether the given value is valid for a movement.
    fn has_movement(value: f32) -> bool {
        value > 1.0e-5
    }

    /// Returns whether the player has a valid forward movement.
    fn has_movement_forward(&self) -> bool {
        Self::has_movement(self.movement_forward())
    }
}

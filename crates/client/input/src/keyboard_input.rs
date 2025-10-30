//! Handles keyboard inputs.

use crate::{slow_down::SlowDown, SlowDownTickable};

use super::Input;

/// Represents keyboard input.
#[derive(Debug, Default)]
pub struct KeyboardInput {
    /// Whether the forward key is currently pressed.
    pub pressing_forward: bool,
    /// Whether the backward key is currently pressed.
    pub pressing_backward: bool,
    /// Whether the left key is currently pressed.
    pub pressing_left: bool,
    /// Whether the right key is currently pressed.
    pub pressing_right: bool,
    /// The movement modifier for forward direction.
    pub movement_forward: f32,
    /// The movement modifier for sideways direction.
    pub movement_sideways: f32,
    /// Whether the jump key is currently pressed.
    pub jumping: bool,
    /// Whether the sneak key is currently pressed.
    pub sneaking: bool,
}

impl KeyboardInput {
    /// Returns the movement modifier based on the positive and negative flags.
    pub fn get_movement_modifier(positive: bool, negative: bool) -> f32 {
        if positive == negative {
            0.0
        } else if positive {
            1.0
        } else {
            -1.0
        }
    }
}

impl SlowDownTickable for KeyboardInput {
    fn tick(&mut self, slow_down: SlowDown) {
        self.pressing_forward = false;
        self.pressing_backward = false;
        self.pressing_left = false;
        self.pressing_right = false;

        self.movement_forward =
            Self::get_movement_modifier(self.pressing_forward, self.pressing_backward);
        self.movement_sideways =
            Self::get_movement_modifier(self.pressing_left, self.pressing_right);

        self.jumping = false;
        self.sneaking = false;

        match slow_down {
            SlowDown::Yes(factor) => {
                self.movement_forward *= factor;
                self.movement_sideways *= factor;
            }
            SlowDown::No => (),
        }
    }
}

impl Input for KeyboardInput {
    fn movement_forward(&self) -> f32 {
        self.movement_forward
    }

    fn movement_sideways(&self) -> f32 {
        self.movement_sideways
    }

    fn jumping(&self) -> bool {
        self.jumping
    }

    fn sneaking(&self) -> bool {
        self.sneaking
    }
}

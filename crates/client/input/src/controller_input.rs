//! Handles keyboard inputs.

use crate::{SlowDownTickable, slow_down::SlowDown};

use super::Input;

/// Inputs from a controller.
#[derive(Debug, Default)]
pub struct ControllerInput {
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
    pub is_jumping: bool,
    /// Whether the sneak key is currently pressed.
    pub is_sneaking: bool,
}

impl ControllerInput {
    /// The movement modifier based on the positive and negative flags.
    pub fn movement_modifier(positive: bool, negative: bool) -> f32 {
        if positive == negative {
            0.0
        } else if positive {
            1.0
        } else {
            -1.0
        }
    }
}

impl SlowDownTickable for ControllerInput {
    fn tick(&mut self, slow_down: SlowDown) {
        self.pressing_forward = false;
        self.pressing_backward = false;
        self.pressing_left = false;
        self.pressing_right = false;

        self.movement_forward =
            Self::movement_modifier(self.pressing_forward, self.pressing_backward);
        self.movement_sideways = Self::movement_modifier(self.pressing_left, self.pressing_right);

        self.is_jumping = false;
        self.is_sneaking = false;

        match slow_down {
            SlowDown::Yes(factor) => {
                self.movement_forward *= factor;
                self.movement_sideways *= factor;
            }
            SlowDown::No => (),
        }
    }
}

impl Input for ControllerInput {
    fn movement_forward(&self) -> f32 {
        self.movement_forward
    }

    fn movement_sideways(&self) -> f32 {
        self.movement_sideways
    }

    fn is_jumping(&self) -> bool {
        self.is_jumping
    }

    fn is_sneaking(&self) -> bool {
        self.is_sneaking
    }
}

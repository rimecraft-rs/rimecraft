use super::{Input, SlowDown};

/// Represents keyboard input.
#[derive(Debug)]
pub struct KeyboardInput;

impl KeyboardInput {
    /// Creates a new instance of `KeyboardInput`.
    pub fn new_input() -> Input<KeyboardInput> {
        Input::new(Self {})
    }
}

impl Input<KeyboardInput> {
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

impl Input<KeyboardInput> {
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

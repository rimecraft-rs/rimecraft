///! Minecraft Input primitives.

use glam::Vec2;

/// Cursor movement handling.
pub mod cursor_movement;
/// Keyboard input handling.
pub mod keyboard_input;

/// Represents the input state.
#[derive(Debug)]
pub struct Input<T> {
    /// Represents the sideways movement.
    pub movement_sideways: f32,
    /// Represents the forward movement.
    pub movement_forward: f32,
    /// Represents if the forward key is pressed.
    pub pressing_forward: bool,
    /// Represents if the backward key is pressed.
    pub pressing_backward: bool,
    /// Represents if the left key is pressed.
    pub pressing_left: bool,
    /// Represents if the right key is pressed.
    pub pressing_right: bool,
    /// Represents if the jump key is pressed.
    pub jumping: bool,
    /// Represents if the sneak key is pressed.
    pub sneaking: bool,
    /// Represents the child.
    pub child: T,
}

/// Represents the tickable component.
pub trait SlowDownTickable {
    /// Performs a tick operation with the specified SlowDown.
    fn tick(&mut self, slow_down: SlowDown);
}

/// Represents the slowdown state.
#[derive(Debug)]
pub enum SlowDown {
    /// Slowdown with a factor.
    Yes(f32),
    /// No slowdown.
    No,
}

impl<T> SlowDownTickable for Input<T> {
    fn tick(&mut self, _slow_down: SlowDown) {

    }
}

impl<T> Input<T> {
    /// Creates a new `Input` instance with the specified child.
    pub fn new(child: T) -> Input<T> {
        Self {
            movement_sideways: 0.0,
            movement_forward: 0.0,
            pressing_forward: false,
            pressing_backward: false,
            pressing_left: false,
            pressing_right: false,
            jumping: false,
            sneaking: false,
            child,
        }
    }

    /// Returns the movement input as a `Vec2`.
    pub fn get_movement_input(&self) -> Vec2 {
        Vec2 {
            x: self.movement_sideways,
            y: self.movement_forward,
        }
    }

    /// Checks if there is movement forward.
    pub fn has_movement_forward(&self) -> bool {
        self.movement_forward > 1.0e-5
    }
}

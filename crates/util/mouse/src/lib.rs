//! Mouse input handling for Rimecraft.

pub mod button;

use rimecraft_global_cx::GlobalContext;

/// Provides associated types for mouse input.
pub trait ProvideMouseTy: GlobalContext {
    /// The button type used for mouse input.
    ///
    /// See: [`button` module](crate::button) for predefined button traits.
    type Button;
}

/// Represents the state of a button, useful for querying button states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ButtonState {
    /// The button is not pressed.
    Idle,
    /// The button is pressed down.
    Pressed,
}

impl ButtonState {
    /// Returns `true` if the button is currently idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns `true` if the button is currently pressed.
    pub fn is_pressed(&self) -> bool {
        matches!(self, Self::Pressed)
    }
}

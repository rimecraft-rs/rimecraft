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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ButtonState {
    /// The button is not pressed.
    Idle,
    /// The button is pressed down.
    Pressed,
    /// The button was double pressed. You should always receive a `Pressed` event before a `DoublePressed` event.
    DoublePressed,
    /// The button is being held down and is dragging.
    Dragging,
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

/// Position of the mouse cursor relative to the window coordinate system.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MousePos {
    /// The x coordinate of the mouse position.
    pub x: f32,
    /// The y coordinate of the mouse position.
    pub y: f32,
}

impl MousePos {
    /// Creates a new [`MousePos`] instance.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for MousePos {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<MousePos> for (f32, f32) {
    fn from(pos: MousePos) -> Self {
        (pos.x, pos.y)
    }
}

impl From<glam::Vec2> for MousePos {
    fn from(vec: glam::Vec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl From<MousePos> for glam::Vec2 {
    fn from(pos: MousePos) -> Self {
        Self::new(pos.x, pos.y)
    }
}

/// Represents a mouse scroll event.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseScroll {
    /// The horizontal scroll amount.
    pub delta_x: f32,
    /// The vertical scroll amount.
    pub delta_y: f32,
}

impl MouseScroll {
    /// Creates a new [`MouseScroll`] instance.
    pub fn new(delta_x: f32, delta_y: f32) -> Self {
        Self { delta_x, delta_y }
    }
}

impl From<(f32, f32)> for MouseScroll {
    fn from((delta_x, delta_y): (f32, f32)) -> Self {
        Self { delta_x, delta_y }
    }
}

impl From<MouseScroll> for (f32, f32) {
    fn from(scroll: MouseScroll) -> Self {
        (scroll.delta_x, scroll.delta_y)
    }
}

impl From<glam::Vec2> for MouseScroll {
    fn from(vec: glam::Vec2) -> Self {
        Self {
            delta_x: vec.x,
            delta_y: vec.y,
        }
    }
}

impl From<MouseScroll> for glam::Vec2 {
    fn from(scroll: MouseScroll) -> Self {
        Self::new(scroll.delta_x, scroll.delta_y)
    }
}

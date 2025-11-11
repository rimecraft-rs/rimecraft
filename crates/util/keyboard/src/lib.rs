//! Keyboard input handling for Rimecraft.

use std::fmt::Debug;

use rimecraft_global_cx::GlobalContext;

use crate::key::KeyModifier;

pub mod key;

/// Provides associated types for keyboard input.
pub trait ProvideKeyboardTy: GlobalContext {
    /// The key type used for keyboard input.
    ///
    /// See: [`key` module](crate::key) for predefined key traits.
    type Key: Copy + Eq;

    /// The modifier type used for keyboard input.
    type Modifier: KeyModifier + Copy + Eq;
}

/// Represents the state of a key, useful for querying key states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum KeyState {
    /// The key is not pressed.
    Idle,
    /// The key is pressed down.
    Pressed,
    /// The key is being held down in a repeating manner.
    Repeating,
}

impl KeyState {
    /// Returns `true` if the key is currently idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns `true` if the key is currently pressed.
    pub fn is_pressed(&self) -> bool {
        matches!(self, Self::Pressed)
    }

    /// Returns `true` if the key is currently repeating.
    pub fn is_repeating(&self) -> bool {
        matches!(self, Self::Repeating)
    }
}

/// Different keyboard input types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum InputType {
    /// The symbolic representation of a key, which may vary based on keyboard layout.
    KeySymbol,
    /// The physical key code, which is independent of keyboard layout.
    KeyCode,
    /// The scan code of the key, which is sent by the keyboard hardware.
    ScanCode,
}

/// Represents a keyboard input event.
pub struct Input<Cx>
where
    Cx: ProvideKeyboardTy,
{
    /// The [`InputType`] of the input.
    pub ty: InputType,
    /// The key represented by this input.
    pub key: Cx::Key,
}

impl<Cx> Debug for Input<Cx>
where
    Cx: ProvideKeyboardTy,
    <Cx as ProvideKeyboardTy>::Key: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Input")
            .field("ty", &self.ty)
            .field("key", &self.key)
            .finish()
    }
}

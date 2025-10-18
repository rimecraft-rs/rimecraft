//! Keyboard input handling for Rimecraft.

use std::fmt::Debug;

use rimecraft_global_cx::GlobalContext;

use key::*;

pub mod key;

/// Provides associated types for keyboard input.
pub trait ProvideKeyboardTy: GlobalContext {
    /// The key type used for keyboard input.
    type Key: KeyNum
        + KeyAlphabet
        + KeyFunction
        + KeyFunctionExt
        + KeyArrow
        + KeyNumpad
        + KeyNumpadExt
        + KeyModifier
        + KeySpecial
        + KeyExt;
}

/// Represents the state of a key, useful for querying key states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyState {
    /// The key is not pressed.
    Idle,
    /// The key is pressed down.
    Pressed,
}

impl KeyState {
    /// Returns `true` if the key is currently idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, KeyState::Idle)
    }

    /// Returns `true` if the key is currently pressed.
    pub fn is_pressed(&self) -> bool {
        matches!(self, KeyState::Pressed)
    }
}

/// Different keyboard input types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

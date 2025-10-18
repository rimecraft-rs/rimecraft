//! Keyboard input handling for Rimecraft.

use std::fmt::Debug;

use crate::key::ProvideKeyTy;

pub mod key;

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
    Cx: ProvideKeyTy,
{
    /// The [`InputType`] of the input.
    pub ty: InputType,
    /// The key represented by this input.
    pub key: Cx::Key,
}

impl<Cx> Debug for Input<Cx>
where
    Cx: ProvideKeyTy,
    <Cx as ProvideKeyTy>::Key: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Input")
            .field("ty", &self.ty)
            .field("key", &self.key)
            .finish()
    }
}

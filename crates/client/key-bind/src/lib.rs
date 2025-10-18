//! Minecraft client key binds.

use std::fmt::Debug;

use rimecraft_keyboard::{KeyState, ProvideKeyboardTy};
use rimecraft_mouse::ProvideMouseTy;

/// Represents a key of a [`KeyBind`]. Currently supporting:
///
/// - Keyboard keys via [`ProvideKeyboardTy::Key`].
/// - Mouse buttons via [`ProvideMouseTy::Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Key<Cx>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    /// The keyboard key associated with this key.
    KeyboardKey(Cx::Key),
    /// The mouse button associated with this key.
    MouseButton(Cx::Button),
}

pub struct KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    default_key: Key<Cx>,
    pub bound_key: Option<Key<Cx>>,
    state: KeyState,
    press_count: u32,
    pub ext: Ext,
}

impl<Cx, Ext> Debug for KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy + Debug,
    <Cx as ProvideKeyboardTy>::Key: Debug,
    <Cx as ProvideMouseTy>::Button: Debug,
    Ext: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyBind")
            .field("default_key", &self.default_key)
            .field("bound_key", &self.bound_key)
            .field("state", &self.state)
            .field("count", &self.press_count)
            .field("ext", &self.ext)
            .finish()
    }
}

impl<Cx, Ext> KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    /// Creates a new `KeyBind` with the specified default key and extension data.
    pub fn new(default_key: Key<Cx>, ext: Ext) -> Self {
        Self {
            default_key,
            bound_key: None,
            state: KeyState::Idle,
            press_count: 0,
            ext,
        }
    }

    /// Gets the effective key for this key bind.
    pub fn effective_key(&self) -> &Key<Cx> {
        self.bound_key.as_ref().unwrap_or(&self.default_key)
    }

    /// Marks the key as pressed and returns a [`KeyBindHandle`] that will release it when dropped.
    ///
    /// Calling this function will increment the press count.
    pub fn press<'a>(&'a mut self) -> KeyBindHandle<'a, Cx, Ext> {
        self.state = KeyState::Pressed;
        self.press_count += 1;
        KeyBindHandle(self)
    }

    /// Releases the key.
    fn release(&mut self) {
        self.state = KeyState::Idle;
    }
}

/// A handle that releases the [`KeyBind`] when dropped.
pub struct KeyBindHandle<'a, Cx, Ext>(&'a mut KeyBind<Cx, Ext>)
where
    Cx: ProvideKeyboardTy + ProvideMouseTy;

impl<Cx, Ext> Debug for KeyBindHandle<'_, Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy + Debug,
    <Cx as ProvideKeyboardTy>::Key: Debug,
    <Cx as ProvideMouseTy>::Button: Debug,
    Ext: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("KeyBindHandle").field(&self.0).finish()
    }
}

impl<Cx, Ext> Drop for KeyBindHandle<'_, Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    fn drop(&mut self) {
        self.0.release();
    }
}

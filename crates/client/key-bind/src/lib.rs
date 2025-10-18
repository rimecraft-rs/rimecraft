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

/// The mode of a [`KeyBind`], determining how it behaves when the key is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyBindMode {
    /// The key bind is active when the key is pressed.
    Hold,
    /// The key bind toggles its state each time the key is pressed.
    Toggle,
}

pub struct KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    mode_getter: Box<dyn Fn() -> KeyBindMode>,
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
    /// Creates a new [`KeyBind`] with the specified default key and extension data.
    pub fn new<F>(mode_getter: F, default_key: Key<Cx>, ext: Ext) -> Self
    where
        F: 'static + Fn() -> KeyBindMode,
    {
        Self {
            mode_getter: Box::new(mode_getter),
            default_key,
            bound_key: None,
            state: KeyState::Idle,
            press_count: 0,
            ext,
        }
    }

    /// Creates a new [`KeyBind`] that will be triggered on key hold.
    pub fn new_hold(default_key: Key<Cx>, ext: Ext) -> Self {
        Self::new(|| KeyBindMode::Hold, default_key, ext)
    }

    /// Creates a new [`KeyBind`] that will be triggered on key toggle (or sticky key).
    pub fn new_toggle(default_key: Key<Cx>, ext: Ext) -> Self {
        Self::new(|| KeyBindMode::Toggle, default_key, ext)
    }

    /// Gets the effective key for this key bind.
    pub fn effective_key(&self) -> &Key<Cx> {
        self.bound_key.as_ref().unwrap_or(&self.default_key)
    }

    /// Calling this function will increment the press count. Note that the [`KeyBindMode`] will be freezed at the time of pressing.
    ///
    /// # `Hold` Mode
    ///
    /// Marks the key as pressed and returns a [`KeyBindHandle`] that will release it when dropped.
    ///
    /// # `Toggle` Mode
    ///
    /// Toggles the key state between pressed and idle, returning a [`KeyBindHandle`] that will do nothing when dropped.
    pub fn press<'a>(&'a mut self) -> KeyBindHandle<'a, Cx, Ext> {
        let mode = (self.mode_getter)();
        self.press_count += 1;

        match mode {
            KeyBindMode::Toggle => {
                self.state = match self.state {
                    KeyState::Idle => KeyState::Pressed,
                    KeyState::Pressed => KeyState::Idle,
                    _ => self.state,
                }
            }
            KeyBindMode::Hold => {
                self.state = KeyState::Pressed;
            }
        }
        self.state = KeyState::Pressed;
        KeyBindHandle(self, mode)
    }

    /// Releases the key.
    fn release(&mut self, mode: KeyBindMode) {
        if mode == KeyBindMode::Hold {
            self.state = KeyState::Idle;
        }
    }
}

/// A handle that releases the [`KeyBind`] when dropped.
pub struct KeyBindHandle<'a, Cx, Ext>(&'a mut KeyBind<Cx, Ext>, KeyBindMode)
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
        self.0.release(self.1);
    }
}

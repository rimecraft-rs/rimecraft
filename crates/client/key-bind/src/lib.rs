//! Minecraft client key binds.

#[cfg(test)]
mod tests;

use std::{fmt::Debug, ops::Deref};

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

/// Operations for [`KeyBind`].
///
/// This trait is useful for abstracting over both [`KeyBind`] and [`KeyBindHandle`].
pub trait KeyBindOp<Cx>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    /// The extension data associated with the key bind.
    type Ext;

    /// Returns the default key of the key bind.
    fn default_key(&self) -> &Key<Cx>;

    /// Returns the currently bound key of the key bind, if any.
    fn bound_key(&self) -> Option<&Key<Cx>>;

    /// Returns the effective key of the key bind, which is the bound key if present, otherwise the default key.
    fn effective_key(&self) -> &Key<Cx> {
        self.bound_key().unwrap_or(self.default_key())
    }

    /// Returns the [`KeyBindMode`] of the key bind.
    fn mode(&self) -> KeyBindMode;

    /// Forcefully resets the key bind state to idle.
    fn reset_to_idle(&mut self);

    /// Returns the current [`KeyState`] of the key bind. The state should be frozen when a key is pressed.
    fn state(&self) -> KeyState;

    /// Returns the number of times the key bind has been pressed since initialization.
    fn press_count(&self) -> u32;

    /// Returns a reference to the extension data.
    fn ext(&self) -> &Self::Ext;

    /// Returns a mutable reference to the extension data.
    fn ext_mut(&mut self) -> &mut Self::Ext;

    /// Binds the key bind to the specified key.
    fn bind(&mut self, key: Key<Cx>);

    /// Reverting the bound key to [`None`]. This makes the key bind use the default key.
    fn unbind(&mut self);
}

/// A key bind that can be pressed and released, tracking its state and press count.
pub struct KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    mode_getter: Box<dyn Fn() -> KeyBindMode>,
    default_key: Key<Cx>,
    /// The currently bound key, if any.
    pub bound_key: Option<Key<Cx>>,
    state: KeyState,
    press_count: u32,
    /// Extension data associated with the key bind.
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

impl<Cx, Ext> PartialEq for KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy + PartialEq,
    <Cx as ProvideKeyboardTy>::Key: PartialEq,
    <Cx as ProvideMouseTy>::Button: PartialEq,
    Ext: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.default_key == other.default_key
            && self.bound_key == other.bound_key
            && self.state == other.state
            && self.press_count == other.press_count
            && self.ext == other.ext
    }
}

impl<Cx, Ext> Eq for KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy + Eq,
    <Cx as ProvideKeyboardTy>::Key: Eq,
    <Cx as ProvideMouseTy>::Button: Eq,
    Ext: Eq,
{
}

impl<Cx, Ext> KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    /// Creates a new [`KeyBind`] with the specified default key and extension data.
    pub fn new<F>(mode_getter: F, default_key: Key<Cx>, ext: Ext) -> Self
    where
        F: Fn() -> KeyBindMode + 'static,
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

    /// Calling this function will increment the press count. Note that the [`KeyBindMode`] will be freezed at the time of pressing.
    ///
    /// # `Hold` Mode
    ///
    /// Marks the key as pressed and returns a [`KeyBindHandle`] that will release it when dropped.
    ///
    /// # `Toggle` Mode
    ///
    /// Toggles the key state between pressed and idle, returning a [`KeyBindHandle`] that will do nothing when dropped.
    pub fn press(&mut self) -> KeyBindHandle<'_, Cx, Ext> {
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
        match mode {
            KeyBindMode::Toggle => match (self.mode_getter)() {
                KeyBindMode::Toggle => {
                    // Does nothing, the state is already toggled.
                }
                KeyBindMode::Hold => {
                    // Releases the key if the mode has changed to Hold.
                    self.state = KeyState::Idle;
                }
            },
            KeyBindMode::Hold => {
                self.state = KeyState::Idle;
            }
        }
    }
}

impl<Cx, Ext> KeyBindOp<Cx> for KeyBind<Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    type Ext = Ext;

    fn default_key(&self) -> &Key<Cx> {
        &self.default_key
    }

    fn bound_key(&self) -> Option<&Key<Cx>> {
        self.bound_key.as_ref()
    }

    fn mode(&self) -> KeyBindMode {
        (self.mode_getter)()
    }

    fn reset_to_idle(&mut self) {
        self.state = KeyState::Idle;
    }

    fn state(&self) -> KeyState {
        self.state
    }

    fn press_count(&self) -> u32 {
        self.press_count
    }

    fn ext(&self) -> &Self::Ext {
        &self.ext
    }

    fn ext_mut(&mut self) -> &mut Self::Ext {
        &mut self.ext
    }

    fn bind(&mut self, key: Key<Cx>) {
        self.bound_key = Some(key);
    }

    fn unbind(&mut self) {
        self.bound_key = None;
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

impl<Cx, Ext> Deref for KeyBindHandle<'_, Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    type Target = KeyBind<Cx, Ext>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<Cx, Ext> KeyBindOp<Cx> for KeyBindHandle<'_, Cx, Ext>
where
    Cx: ProvideKeyboardTy + ProvideMouseTy,
{
    type Ext = Ext;

    fn default_key(&self) -> &Key<Cx> {
        self.0.default_key()
    }

    fn bound_key(&self) -> Option<&Key<Cx>> {
        self.0.bound_key()
    }

    fn mode(&self) -> KeyBindMode {
        // Returns the frozen key bind mode at the time of pressing.
        self.1
    }

    fn reset_to_idle(&mut self) {
        self.0.reset_to_idle();
    }

    fn state(&self) -> KeyState {
        self.0.state()
    }

    fn press_count(&self) -> u32 {
        self.0.press_count()
    }

    fn ext(&self) -> &Self::Ext {
        self.0.ext()
    }

    fn ext_mut(&mut self) -> &mut Self::Ext {
        self.0.ext_mut()
    }

    fn bind(&mut self, key: Key<Cx>) {
        self.0.bind(key);
    }

    fn unbind(&mut self) {
        self.0.unbind();
    }
}

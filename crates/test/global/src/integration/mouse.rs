//! `rimecraft-client-mouse` integrations.

#![cfg(feature = "mouse")]

use mouse::{ProvideMouseTy, button::MouseButton};

use crate::TestContext;

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TestButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
    /// Other mouse button.
    Other(u8),
}

impl ProvideMouseTy for TestContext {
    type Button = TestButton;
}

impl MouseButton for TestButton {
    const BUTTON_PRIMARY: Self = Self::Left;
    const BUTTON_SECONDARY: Self = Self::Right;
    const BUTTON_SCROLL_WHEEL: Self = Self::Middle;
}

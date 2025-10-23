//! `rimecraft-client-mouse` integrations.

#![cfg(feature = "mouse")]

use mouse::{ProvideMouseTy, button::MouseButton};

use crate::TestContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TestButton {
    Left,
    Right,
    Middle,
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

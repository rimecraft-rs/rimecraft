//! `rimecraft-client-keyboard` integrations.

#![cfg(feature = "keyboard")]
#![allow(missing_docs)]

use keyboard::{
    ProvideKeyboardTy,
    key::{
        KeyAlphabet, KeyArrow, KeyFunction, KeyFunctionExt, KeyModifier, KeyNum, KeyNumpad,
        KeyNumpadExt, KeySpecial,
    },
};

use crate::TestContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TestKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadDecimal,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    NumpadAdd,
    NumpadEnter,
    LeftShift,
    LeftCtrl,
    LeftAlt,
    LeftMeta,
    RightShift,
    RightCtrl,
    RightAlt,
    RightMeta,
    Apostrophe,
    Backslash,
    Comma,
    Equal,
    GraveAccent,
    LeftBracket,
    Minus,
    Period,
    RightBracket,
    Semicolon,
    Slash,
    Space,
    Tab,
    Enter,
    Escape,
    Backspace,
    Delete,
    End,
    Home,
    Insert,
    PageDown,
    PageUp,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TestModifier {
    LeftShift,
    LeftCtrl,
    LeftAlt,
    LeftMeta,
    RightShift,
    RightCtrl,
    RightAlt,
    RightMeta,
}

impl ProvideKeyboardTy for TestContext {
    type Key = TestKey;

    type Modifier = TestModifier;
}

impl KeyNum for TestKey {
    const NUM_0: Self = Self::Num0;
    const NUM_1: Self = Self::Num1;
    const NUM_2: Self = Self::Num2;
    const NUM_3: Self = Self::Num3;
    const NUM_4: Self = Self::Num4;
    const NUM_5: Self = Self::Num5;
    const NUM_6: Self = Self::Num6;
    const NUM_7: Self = Self::Num7;
    const NUM_8: Self = Self::Num8;
    const NUM_9: Self = Self::Num9;
}

impl KeyAlphabet for TestKey {
    const A: Self = Self::A;
    const B: Self = Self::B;
    const C: Self = Self::C;
    const D: Self = Self::D;
    const E: Self = Self::E;
    const F: Self = Self::F;
    const G: Self = Self::G;
    const H: Self = Self::H;
    const I: Self = Self::I;
    const J: Self = Self::J;
    const K: Self = Self::K;
    const L: Self = Self::L;
    const M: Self = Self::M;
    const N: Self = Self::N;
    const O: Self = Self::O;
    const P: Self = Self::P;
    const Q: Self = Self::Q;
    const R: Self = Self::R;
    const S: Self = Self::S;
    const T: Self = Self::T;
    const U: Self = Self::U;
    const V: Self = Self::V;
    const W: Self = Self::W;
    const X: Self = Self::X;
    const Y: Self = Self::Y;
    const Z: Self = Self::Z;
}

impl KeyFunction for TestKey {
    const F1: Self = Self::F1;
    const F2: Self = Self::F2;
    const F3: Self = Self::F3;
    const F4: Self = Self::F4;
    const F5: Self = Self::F5;
    const F6: Self = Self::F6;
    const F7: Self = Self::F7;
    const F8: Self = Self::F8;
    const F9: Self = Self::F9;
    const F10: Self = Self::F10;
    const F11: Self = Self::F11;
    const F12: Self = Self::F12;
}

impl KeyFunctionExt for TestKey {
    const F13: Self = Self::F13;
    const F14: Self = Self::F14;
    const F15: Self = Self::F15;
    const F16: Self = Self::F16;
    const F17: Self = Self::F17;
    const F18: Self = Self::F18;
    const F19: Self = Self::F19;
    const F20: Self = Self::F20;
    const F21: Self = Self::F21;
    const F22: Self = Self::F22;
    const F23: Self = Self::F23;
    const F24: Self = Self::F24;
    const F25: Self = Self::F25;
}

impl KeyArrow for TestKey {
    const ARROW_UP: Self = Self::ArrowUp;
    const ARROW_DOWN: Self = Self::ArrowDown;
    const ARROW_LEFT: Self = Self::ArrowLeft;
    const ARROW_RIGHT: Self = Self::ArrowRight;
}

impl KeyNumpad for TestKey {
    const NUMPAD_0: Self = Self::Numpad0;
    const NUMPAD_1: Self = Self::Numpad1;
    const NUMPAD_2: Self = Self::Numpad2;
    const NUMPAD_3: Self = Self::Numpad3;
    const NUMPAD_4: Self = Self::Numpad4;
    const NUMPAD_5: Self = Self::Numpad5;
    const NUMPAD_6: Self = Self::Numpad6;
    const NUMPAD_7: Self = Self::Numpad7;
    const NUMPAD_8: Self = Self::Numpad8;
    const NUMPAD_9: Self = Self::Numpad9;
}

impl KeyNumpadExt for TestKey {
    const NUMPAD_DECIMAL: Self = Self::NumpadDecimal;
    const NUMPAD_DIVIDE: Self = Self::NumpadDivide;
    const NUMPAD_MULTIPLY: Self = Self::NumpadMultiply;
    const NUMPAD_SUBTRACT: Self = Self::NumpadSubtract;
    const NUMPAD_ADD: Self = Self::NumpadAdd;
    const NUMPAD_ENTER: Self = Self::NumpadEnter;
}

impl KeyModifier for TestKey {
    const LEFT_SHIFT: Self = Self::LeftShift;
    const LEFT_CTRL: Self = Self::LeftCtrl;
    const LEFT_ALT: Self = Self::LeftAlt;
    const LEFT_META: Self = Self::LeftMeta;
    const RIGHT_SHIFT: Self = Self::RightShift;
    const RIGHT_CTRL: Self = Self::RightCtrl;
    const RIGHT_ALT: Self = Self::RightAlt;
    const RIGHT_META: Self = Self::RightMeta;
}

impl KeySpecial for TestKey {
    const APOSTROPHE: Self = Self::Apostrophe;
    const BACKSLASH: Self = Self::Backslash;
    const COMMA: Self = Self::Comma;
    const EQUAL: Self = Self::Equal;
    const GRAVE_ACCENT: Self = Self::GraveAccent;
    const LEFT_BRACKET: Self = Self::LeftBracket;
    const MINUS: Self = Self::Minus;
    const PERIOD: Self = Self::Period;
    const RIGHT_BRACKET: Self = Self::RightBracket;
    const SEMICOLON: Self = Self::Semicolon;
    const SLASH: Self = Self::Slash;
    const SPACE: Self = Self::Space;
    const TAB: Self = Self::Tab;
    const ENTER: Self = Self::Enter;
    const ESCAPE: Self = Self::Escape;
    const BACKSPACE: Self = Self::Backspace;
    const DELETE: Self = Self::Delete;
    const END: Self = Self::End;
    const HOME: Self = Self::Home;
    const INSERT: Self = Self::Insert;
    const PAGE_DOWN: Self = Self::PageDown;
    const PAGE_UP: Self = Self::PageUp;
    const CAPS_LOCK: Self = Self::CapsLock;
    const SCROLL_LOCK: Self = Self::ScrollLock;
    const NUM_LOCK: Self = Self::NumLock;
    const PRINT_SCREEN: Self = Self::PrintScreen;
}

impl KeyModifier for TestModifier {
    const LEFT_SHIFT: Self = Self::LeftShift;
    const LEFT_CTRL: Self = Self::LeftCtrl;
    const LEFT_ALT: Self = Self::LeftAlt;
    const LEFT_META: Self = Self::LeftMeta;
    const RIGHT_SHIFT: Self = Self::RightShift;
    const RIGHT_CTRL: Self = Self::RightCtrl;
    const RIGHT_ALT: Self = Self::RightAlt;
    const RIGHT_META: Self = Self::RightMeta;
}

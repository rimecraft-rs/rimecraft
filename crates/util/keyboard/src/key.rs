use rimecraft_global_cx::GlobalContext;

/// Provides associated types for keyboard keys.
pub trait ProvideKeyTy: GlobalContext {
    /// Numeric keys `0-9`.
    type Num: KeyNum;
    /// Alphabet keys `A-Z`.
    type Alphabet: KeyAlphabet;
    /// Function keys `F1-F12`.
    type Function: KeyFunction;
    /// Extended function keys `F13-F25`.
    type FunctionExt: KeyFunctionExt;
    /// Arrow keys.
    type Arrow: KeyArrow;
    /// Numpad keys.
    type Numpad: KeyNumpad;
    /// Extended numpad keys.
    type NumpadExt: KeyNumpadExt;
    /// Modifier keys.
    type Modifier: KeyModifier;
    /// Special keys.
    type Special: KeySpecial;
}

macro_rules! define_key_trait {
    ($($($doc:expr)* ;)? $vis:vis $name:ident { $($variant:ident $(: $variant_doc:expr)?),* $(,)? }) => {
        $($(#[doc = $doc])*)?
        $vis trait $name {
            $(
                $(#[doc = $variant_doc])?
                const $variant: Self;
            )*
        }

        impl $name for () {
            $(const $variant: Self = ();)*
        }
    };
}

define_key_trait! {
    "Numeric keys `0-9`.";
    pub KeyNum {
        NUM_0: "Numeric key `0`.",
        NUM_1: "Numeric key `1`.",
        NUM_2: "Numeric key `2`.",
        NUM_3: "Numeric key `3`.",
        NUM_4: "Numeric key `4`.",
        NUM_5: "Numeric key `5`.",
        NUM_6: "Numeric key `6`.",
        NUM_7: "Numeric key `7`.",
        NUM_8: "Numeric key `8`.",
        NUM_9: "Numeric key `9`.",
    }
}

define_key_trait! {
    "Alphabet keys `A-Z`.";
    pub KeyAlphabet {
        A: "Alphabet key `A`.",
        B: "Alphabet key `B`.",
        C: "Alphabet key `C`.",
        D: "Alphabet key `D`.",
        E: "Alphabet key `E`.",
        F: "Alphabet key `F`.",
        G: "Alphabet key `G`.",
        H: "Alphabet key `H`.",
        I: "Alphabet key `I`.",
        J: "Alphabet key `J`.",
        K: "Alphabet key `K`.",
        L: "Alphabet key `L`.",
        M: "Alphabet key `M`.",
        N: "Alphabet key `N`.",
        O: "Alphabet key `O`.",
        P: "Alphabet key `P`.",
        Q: "Alphabet key `Q`.",
        R: "Alphabet key `R`.",
        S: "Alphabet key `S`.",
        T: "Alphabet key `T`.",
        U: "Alphabet key `U`.",
        V: "Alphabet key `V`.",
        W: "Alphabet key `W`.",
        X: "Alphabet key `X`.",
        Y: "Alphabet key `Y`.",
        Z: "Alphabet key `Z`.",
    }
}

define_key_trait! {
    "Function keys `F1-F12`.";
    pub KeyFunction {
        F1: "Function key `F1`.",
        F2: "Function key `F2`.",
        F3: "Function key `F3`.",
        F4: "Function key `F4`.",
        F5: "Function key `F5`.",
        F6: "Function key `F6`.",
        F7: "Function key `F7`.",
        F8: "Function key `F8`.",
        F9: "Function key `F9`.",
        F10: "Function key `F10`.",
        F11: "Function key `F11`.",
        F12: "Function key `F12`.",
    }
}

define_key_trait! {
    "Extended function keys `F13-F25`.";
    pub KeyFunctionExt {
        F13: "Function key `F13`.",
        F14: "Function key `F14`.",
        F15: "Function key `F15`.",
        F16: "Function key `F16`.",
        F17: "Function key `F17`.",
        F18: "Function key `F18`.",
        F19: "Function key `F19`.",
        F20: "Function key `F20`.",
        F21: "Function key `F21`.",
        F22: "Function key `F22`.",
        F23: "Function key `F23`.",
        F24: "Function key `F24`.",
        F25: "Function key `F25`.",
    }
}

define_key_trait! {
    "Arrow keys.";
    pub KeyArrow {
        UP: "Up arrow key.",
        DOWN: "Down arrow key.",
        LEFT: "Left arrow key.",
        RIGHT: "Right arrow key.",
    }
}

define_key_trait! {
    "Numpad keys.";
    pub KeyNumpad {
        NUMPAD_0: "Numpad key `0`.",
        NUMPAD_1: "Numpad key `1`.",
        NUMPAD_2: "Numpad key `2`.",
        NUMPAD_3: "Numpad key `3`.",
        NUMPAD_4: "Numpad key `4`.",
        NUMPAD_5: "Numpad key `5`.",
        NUMPAD_6: "Numpad key `6`.",
        NUMPAD_7: "Numpad key `7`.",
        NUMPAD_8: "Numpad key `8`.",
        NUMPAD_9: "Numpad key `9`.",
    }
}

define_key_trait! {
    "Extended numpad keys.";
    pub KeyNumpadExt {
        DECIMAL: "Numpad decimal key.",
        DIVIDE: "Numpad divide key.",
        MULTIPLY: "Numpad multiply key.",
        SUBTRACT: "Numpad subtract key.",
        ADD: "Numpad add key.",
        ENTER: "Numpad enter key.",
    }
}

define_key_trait! {
    "Modifier keys.";
    pub KeyModifier {
        LEFT_SHIFT: "Left shift key.",
        RIGHT_SHIFT: "Right shift key.",
        LEFT_CTRL: "Left control key.",
        RIGHT_CTRL: "Right control key.",
        LEFT_ALT: "Left alt key.",
        RIGHT_ALT: "Right alt key.",
        LEFT_META: "Left meta key.",
        RIGHT_META: "Right meta key.",
    }
}

define_key_trait! {
    "Special keys.";
    pub KeySpecial {
        APOSTROPHE: "Apostrophe key (`'`).",
        BACKSLASH: "Backslash key (`\\`).",
        COMMA: "Comma key (`,`).",
        EQUAL: "Equal key (`=`).",
        GRAVE_ACCENT: "Grave accent key (`` ` ``).",
        LEFT_BRACKET: "Left bracket key (`[`).",
        MINUS: "Minus key (`-`).",
        PERIOD: "Period key (`.`).",
        RIGHT_BRACKET: "Right bracket key (`]`).",
        SEMICOLON: "Semicolon key (`;`).",
        SLASH: "Slash key (`/`).",
        SPACE: "Space key.",
        TAB: "Tab key.",
        ENTER: "Enter key.",
        ESCAPE: "Escape key.",
        BACKSPACE: "Backspace key.",
        DELETE: "Delete key.",
        END: "End key.",
        HOME: "Home key.",
        INSERT: "Insert key.",
        PAGE_DOWN: "Page down key.",
        PAGE_UP: "Page up key.",
        CAPS_LOCK: "Caps lock key.",
        SCROLL_LOCK: "Scroll lock key.",
        NUM_LOCK: "Num lock key.",
        PRINT_SCREEN: "Print screen key.",
    }
}

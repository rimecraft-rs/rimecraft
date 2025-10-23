//! Defines traits for keyboard keys.
//!
//! All traits defined in this module are implemented for unit type `()`.

macro_rules! define_key_trait {
    ($(#[$outer:meta])* $vis:vis $name:ident { $($(#[$variant_outer:meta])* $variant:ident),* $(,)? }) => {
        $(#[$outer])*
        $vis trait $name {
            $(
                $(#[$variant_outer])*
                const $variant: Self;
            )*
        }

        impl $name for () {
            $(const $variant: Self = ();)*
        }
    };
}

define_key_trait! {
    /// Numeric keys `0-9`.
    pub KeyNum {
        /// Numeric key `0`.
        NUM_0,
        /// Numeric key `1`.
        NUM_1,
        /// Numeric key `2`.
        NUM_2,
        /// Numeric key `3`.
        NUM_3,
        /// Numeric key `4`.
        NUM_4,
        /// Numeric key `5`.
        NUM_5,
        /// Numeric key `6`.
        NUM_6,
        /// Numeric key `7`.
        NUM_7,
        /// Numeric key `8`.
        NUM_8,
        /// Numeric key `9`.
        NUM_9,
    }
}

define_key_trait! {
    /// Alphabet keys `A-Z`.
    pub KeyAlphabet {
        /// Alphabet key `A`.
        A,
        /// Alphabet key `B`.
        B,
        /// Alphabet key `C`.
        C,
        /// Alphabet key `D`.
        D,
        /// Alphabet key `E`.
        E,
        /// Alphabet key `F`.
        F,
        /// Alphabet key `G`.
        G,
        /// Alphabet key `H`.
        H,
        /// Alphabet key `I`.
        I,
        /// Alphabet key `J`.
        J,
        /// Alphabet key `K`.
        K,
        /// Alphabet key `L`.
        L,
        /// Alphabet key `M`.
        M,
        /// Alphabet key `N`.
        N,
        /// Alphabet key `O`.
        O,
        /// Alphabet key `P`.
        P,
        /// Alphabet key `Q`.
        Q,
        /// Alphabet key `R`.
        R,
        /// Alphabet key `S`.
        S,
        /// Alphabet key `T`.
        T,
        /// Alphabet key `U`.
        U,
        /// Alphabet key `V`.
        V,
        /// Alphabet key `W`.
        W,
        /// Alphabet key `X`.
        X,
        /// Alphabet key `Y`.
        Y,
        /// Alphabet key `Z`.
        Z,
    }
}

define_key_trait! {
    /// Function keys `F1-F12`.
    pub KeyFunction {
        /// Function key `F1`.
        F1,
        /// Function key `F2`.
        F2,
        /// Function key `F3`.
        F3,
        /// Function key `F4`.
        F4,
        /// Function key `F5`.
        F5,
        /// Function key `F6`.
        F6,
        /// Function key `F7`.
        F7,
        /// Function key `F8`.
        F8,
        /// Function key `F9`.
        F9,
        /// Function key `F10`.
        F10,
        /// Function key `F11`.
        F11,
        /// Function key `F12`.
        F12,
    }
}

define_key_trait! {
    /// Extended function keys `F13-F25`.
    pub KeyFunctionExt {
        /// Function key `F13`.
        F13,
        /// Function key `F14`.
        F14,
        /// Function key `F15`.
        F15,
        /// Function key `F16`.
        F16,
        /// Function key `F17`.
        F17,
        /// Function key `F18`.
        F18,
        /// Function key `F19`.
        F19,
        /// Function key `F20`.
        F20,
        /// Function key `F21`.
        F21,
        /// Function key `F22`.
        F22,
        /// Function key `F23`.
        F23,
        /// Function key `F24`.
        F24,
        /// Function key `F25`.
        F25,
    }
}

define_key_trait! {
    /// Arrow keys.
    pub KeyArrow {
        /// Up arrow key.
        ARROW_UP,
        /// Down arrow key.
        ARROW_DOWN,
        /// Left arrow key.
        ARROW_LEFT,
        /// Right arrow key.
        ARROW_RIGHT,
    }
}

define_key_trait! {
    /// Numpad keys.
    pub KeyNumpad {
        /// Numpad key `0`.
        NUMPAD_0,
        /// Numpad key `1`.
        NUMPAD_1,
        /// Numpad key `2`.
        NUMPAD_2,
        /// Numpad key `3`.
        NUMPAD_3,
        /// Numpad key `4`.
        NUMPAD_4,
        /// Numpad key `5`.
        NUMPAD_5,
        /// Numpad key `6`.
        NUMPAD_6,
        /// Numpad key `7`.
        NUMPAD_7,
        /// Numpad key `8`.
        NUMPAD_8,
        /// Numpad key `9`.
        NUMPAD_9,
    }
}

define_key_trait! {
    /// Extended numpad keys.
    pub KeyNumpadExt {
        /// Numpad decimal key.
        NUMPAD_DECIMAL,
        /// Numpad divide key.
        NUMPAD_DIVIDE,
        /// Numpad multiply key.
        NUMPAD_MULTIPLY,
        /// Numpad subtract key.
        NUMPAD_SUBTRACT,
        /// Numpad add key.
        NUMPAD_ADD,
        /// Numpad enter key.
        NUMPAD_ENTER,
    }
}

define_key_trait! {
    /// Modifier keys.
    pub KeyModifier {
        /// Left shift key.
        LEFT_SHIFT,
        /// Left control key.
        LEFT_CTRL,
        /// Left alt key.
        LEFT_ALT,
        /// Left meta key.
        LEFT_META,
        /// Right shift key.
        RIGHT_SHIFT,
        /// Right control key.
        RIGHT_CTRL,
        /// Right alt key.
        RIGHT_ALT,
        /// Right meta key.
        RIGHT_META,
    }
}

define_key_trait! {
    /// Special keys.
    pub KeySpecial {
        /// Apostrophe key (`'`).
        APOSTROPHE,
        /// Backslash key (`\`).
        BACKSLASH,
        /// Comma key (`,`).
        COMMA,
        /// Equal key (`=`).
        EQUAL,
        /// Grave accent key (`` ` ``).
        GRAVE_ACCENT,
        /// Left bracket key (`[`).
        LEFT_BRACKET,
        /// Minus key (`-`).
        MINUS,
        /// Period key (`.`).
        PERIOD,
        /// Right bracket key (`]`).
        RIGHT_BRACKET,
        /// Semicolon key (`;`).
        SEMICOLON,
        /// Slash key (`/`).
        SLASH,
        /// Space key.
        SPACE,
        /// Tab key.
        TAB,
        /// Enter key.
        ENTER,
        /// Escape key.
        ESCAPE,
        /// Backspace key.
        BACKSPACE,
        /// Delete key.
        DELETE,
        /// End key.
        END,
        /// Home key.
        HOME,
        /// Insert key.
        INSERT,
        /// Page down key.
        PAGE_DOWN,
        /// Page up key.
        PAGE_UP,
        /// Caps lock key.
        CAPS_LOCK,
        /// Scroll lock key.
        SCROLL_LOCK,
        /// Num lock key.
        NUM_LOCK,
        /// Print screen key.
        PRINT_SCREEN,
    }
}

//! Defines traits for mouse buttons.
//!
//! All traits defined in this module are implemented for unit type `()`.

macro_rules! define_button_trait {
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

define_button_trait! {
    "Mouse buttons.";
    pub MouseButton {
        BUTTON_PRIMARY: "The primary button, typically the left mouse button.",
        BUTTON_SECONDARY: "The secondary button, typically the right mouse button.",
        BUTTON_SCROLL_WHEEL: "The scroll wheel button, typically the middle mouse button.",
    }
}

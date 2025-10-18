//! Defines traits for mouse buttons.
//!
//! All traits defined in this module are implemented for unit type `()`,
//! allowing users to opt out of specifying concrete button types when they are not needed.

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
    "Extended buttons.";
    pub ButtonExt {}
}

define_button_trait! {
    "Mouse buttons.";
    pub Button {
        LEFT: "Left mouse button, or the primary button.",
        RIGHT: "Right mouse button, or the secondary button.",
        MIDDLE: "Middle mouse button, or the scroll wheel button.",
    }
}

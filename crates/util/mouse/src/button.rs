//! Defines traits for mouse buttons.
//!
//! All traits defined in this module are implemented for unit type `()`.

macro_rules! define_button_trait {
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

define_button_trait! {
    /// Mouse buttons.
    pub MouseButton {
        /// The primary button, typically the left mouse button.
        BUTTON_PRIMARY,
        /// The secondary button, typically the right mouse button.
        BUTTON_SECONDARY,
        /// The scroll wheel button, typically the middle mouse button.
        BUTTON_SCROLL_WHEEL,
    }
}

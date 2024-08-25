//! [`Update`] variant for [`erased_serde`].

#![allow(single_use_lifetimes)]

use crate::*;

// Part of the public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __internal_update_from_erased {
    () => {
        #[inline]
        fn update<D>(
            &mut self,
            deserializer: D,
        ) -> Result<(), <D as ::serde::Deserializer<'de>>::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            $crate::erased::ErasedUpdate::erased_update(
                self,
                &mut <dyn ::erased_serde::Deserializer<'de>>::erase(deserializer),
            )
            .map_err(::serde::de::Error::custom)
        }
    };
}

/// Implement [`Update`] for the target trait object types
/// that implements [`ErasedUpdate`].
#[macro_export]
macro_rules! update_trait_object {
    ($($t:tt)*) => {
        impl<'a, 'de> $crate::Update<'de> for dyn $($t)* + 'a {
            $crate::__internal_update_from_erased!();
        }

        impl<'a, 'de> $crate::Update<'de> for dyn $($t)* + ::core::marker::Send + 'a {
            $crate::__internal_update_from_erased!();
        }

        impl<'a, 'de> $crate::Update<'de> for dyn $($t)* + ::core::marker::Sync + 'a {
            $crate::__internal_update_from_erased!();
        }

        impl<'a, 'de> $crate::Update<'de> for dyn $($t)* + ::core::marker::Send + ::core::marker::Sync + 'a {
            $crate::__internal_update_from_erased!();
        }
    };
}

/// [`Update`] but type erased.
pub trait ErasedUpdate<'de> {
    /// Update this type from an erased deserializer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the type
    /// failed to deserialize in place.
    fn erased_update(
        &mut self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<(), erased_serde::Error>;
}

crate::update_trait_object!(ErasedUpdate<'de>);

impl<'de, T> ErasedUpdate<'de> for T
where
    T: ?Sized + Update<'de>,
{
    #[inline]
    fn erased_update(
        &mut self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<(), erased_serde::Error> {
        self.update(deserializer)
    }
}

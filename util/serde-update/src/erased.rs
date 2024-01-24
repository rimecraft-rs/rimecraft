use crate::*;

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_update_from_erased {
    () => {
        #[inline]
        fn update<D>(
            &mut self,
            deserializer: D,
        ) -> Result<(), <D as serde::Deserializer<'de>>::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;
            self.erased_update(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
                .map_err(D::Error::custom)
        }
    };
}

/// Implement [`Update`] for the target trait object types
/// that implements [`ErasedUpdate`].
#[macro_export]
macro_rules! update_trait_object {
    ($($t:tt)*) => {
        impl<'de> $crate::Update<'de> for dyn $($t)* {
            $crate::__internal_update_from_erased!();
        }

        impl<'de> $crate::Update<'de> for dyn $($t)* + Send {
            $crate::__internal_update_from_erased!();
        }

        impl<'de> $crate::Update<'de> for dyn $($t)* + Sync {
            $crate::__internal_update_from_erased!();
        }

        impl<'de> $crate::Update<'de> for dyn $($t)* + Send + Sync {
            $crate::__internal_update_from_erased!();
        }
    };
}

/// [`Update`] but type erased.
pub trait ErasedUpdate<'de>: erased_serde::Serialize {
    /// Update this type from an erased deserializer.
    fn erased_update(
        &mut self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<(), erased_serde::Error>;
}

erased_serde::serialize_trait_object!(<'de> ErasedUpdate<'de>);
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

pub struct ErasedWrapper<'a, 'de>(pub &'a mut dyn ErasedUpdate<'de>);

impl<'a, 'de> serde::Serialize for ErasedWrapper<'a, 'de> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a, 'de> Update<'de> for ErasedWrapper<'a, 'de> {
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        self.0
            .erased_update(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
            .map_err(D::Error::custom)
    }
}

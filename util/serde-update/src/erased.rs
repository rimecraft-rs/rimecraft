use crate::*;

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_update_from_erased {
    () => {
        #[inline]
        fn update<'de, D>(
            &'de mut self,
            deserializer: D,
        ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
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
    ($t:path) => {
        impl $crate::Update for dyn $t {
            $crate::__internal_update_from_erased!();
        }

        impl $crate::Update for dyn $t + Send {
            $crate::__internal_update_from_erased!();
        }

        impl $crate::Update for dyn $t + Sync {
            $crate::__internal_update_from_erased!();
        }

        impl $crate::Update for dyn $t + Send + Sync {
            $crate::__internal_update_from_erased!();
        }
    };
}

/// [`Update`] but type erased.
pub trait ErasedUpdate: erased_serde::Serialize {
    /// Update this type from an erased deserializer.
    fn erased_update<'de>(
        &'de mut self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<(), erased_serde::Error>;
}

erased_serde::serialize_trait_object!(ErasedUpdate);
crate::update_trait_object!(ErasedUpdate);

impl<T> ErasedUpdate for T
where
    T: ?Sized + Update,
{
    #[inline]
    fn erased_update<'de>(
        &'de mut self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<(), erased_serde::Error> {
        self.update(deserializer)
    }
}

#[derive(serde::Serialize)]
pub struct ErasedWrapper<'a>(pub &'a mut dyn ErasedUpdate);

impl Update for ErasedWrapper<'_> {
    #[inline]
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        self.0
            .erased_update(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
            .map_err(D::Error::custom)
    }
}

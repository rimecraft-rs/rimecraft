/// Represent types that are able to be updated
/// through serializing and deserializing.
pub trait Update: serde::Serialize {
    /// Update this type from a deserializer.
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>;
}

impl<T> Update for T
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    #[inline]
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self = Self::deserialize(deserializer)?;
        Ok(())
    }
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
    fn erased_update<'de>(
        &'de mut self,
        deserializer: &mut dyn erased_serde::Deserializer<'de>,
    ) -> Result<(), erased_serde::Error> {
        self.update(deserializer)
    }
}

/// Implement [`Update`] for the target trait object types
/// that implements [`ErasedUpdate`].
#[macro_export]
macro_rules! update_trait_object {
    ($t:path) => {
        impl $crate::serde_update::Update for dyn $t {
            $crate::__impl_update_from_erased!();
        }

        impl $crate::serde_update::Update for dyn $t + Send {
            $crate::__impl_update_from_erased!();
        }

        impl $crate::serde_update::Update for dyn $t + Sync {
            $crate::__impl_update_from_erased!();
        }

        impl $crate::serde_update::Update for dyn $t + Send + Sync {
            $crate::__impl_update_from_erased!();
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_update_from_erased {
    () => {
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

#[derive(serde::Serialize)]
struct ErasedWrapper<'a> {
    value: &'a mut dyn ErasedUpdate,
}

impl Update for ErasedWrapper<'_> {
    fn update<'de, D>(
        &'de mut self,
        deserializer: D,
    ) -> Result<(), <D as serde::Deserializer<'_>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        self.value
            .erased_update(&mut <dyn erased_serde::Deserializer>::erase(deserializer))
            .map_err(D::Error::custom)
    }
}

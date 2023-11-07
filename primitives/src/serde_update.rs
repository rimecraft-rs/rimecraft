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

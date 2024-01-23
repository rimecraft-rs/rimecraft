#[cfg(feature = "erased")]
pub mod erased;

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

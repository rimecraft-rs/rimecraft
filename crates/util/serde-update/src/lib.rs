//! Utilities for updating types through serialization and deserialization.

#[cfg(feature = "erased")]
pub mod erased;

/// Represent types that are able to be updated
/// through serializing and deserializing.
pub trait Update<'de> {
    /// Update this type from a deserializer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the type
    /// failed to deserialize in place.
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>;
}

impl<'de, T> Update<'de> for T
where
    T: serde::Deserialize<'de>,
{
    #[inline]
    fn update<D>(&mut self, deserializer: D) -> Result<(), <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self = Self::deserialize(deserializer)?;
        Ok(())
    }
}
